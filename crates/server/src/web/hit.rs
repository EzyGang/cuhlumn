//! The analytics beacon `POST /_hit` that the page script sends once for each page view.

use std::net::SocketAddr;

use axum::{
    body::to_bytes,
    extract::{ConnectInfo, Request},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::Deserialize;

use crate::{
    analytics::{View, agent::Agent, referrer::referrer_host, visitor::client_ip},
    db::{analytics::check, unix_now},
    state::AppState,
    web::{
        plain,
        request::request_host,
        route::{Route, decode_path},
    },
};

/// The path of the beacon. Slugs have no `_`, so no post has this path.
pub const HIT_PATH: &str = "/_hit";

/// The body of a beacon.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Beacon {
    /// The page path, as `location.pathname` gives it.
    path: String,
    /// The referring address, as `document.referrer` gives it.
    referrer: String,
}

/// The largest beacon body, in bytes.
const BODY_LIMIT: usize = 2 * 1024;

/// Counts a beacon when it is a page view of a reader on a site page.
///
/// A well-formed beacon always gets 204, also when it does not count, so senders learn nothing.
/// A body above 2 KiB gets 413, and a body that is not a beacon gets 400.
pub async fn receive(state: &AppState, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let Ok(bytes) = to_bytes(body, BODY_LIMIT).await else {
        return plain(
            StatusCode::PAYLOAD_TOO_LARGE,
            "The beacon is too large.".to_owned(),
        );
    };
    let Ok(beacon) = serde_json::from_slice::<Beacon>(&bytes) else {
        return plain(
            StatusCode::BAD_REQUEST,
            "The beacon is not valid.".to_owned(),
        );
    };
    let peer = parts
        .extensions
        .get::<ConnectInfo<SocketAddr>>()
        .map(|info| info.0);
    let user_agent = parts
        .headers
        .get(header::USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    let (Some(client), Some(host)) = (
        Agent::classify(user_agent),
        request_host(&parts.headers, &parts.uri),
    ) else {
        return no_content();
    };
    let path = decode_path(&beacon.path).into_owned();
    let route = Route::parse(&path);
    if !matches!(route, Route::Home | Route::Entry(_) | Route::Tag(_)) {
        return no_content();
    }
    let name = host.clone();
    let checked = state
        .db()
        .call(move |db| check::counts(db, &name, &route))
        .await;
    match checked {
        Ok(true) => (),
        Ok(false) => return no_content(),
        Err(err) => {
            tracing::error!(%host, error = %err, "analytics page check failed");
            return no_content();
        }
    }
    let view = View {
        referrer: referrer_host(&beacon.referrer, &host),
        ip: client_ip(&parts.headers, peer),
        user_agent: user_agent.to_owned(),
        host,
        path,
        client,
    };
    state.analytics().record(view, unix_now());
    no_content()
}

/// Returns the empty answer of a well-formed beacon.
fn no_content() -> Response {
    let no_store = HeaderValue::from_static("no-store");
    (StatusCode::NO_CONTENT, [(header::CACHE_CONTROL, no_store)]).into_response()
}
