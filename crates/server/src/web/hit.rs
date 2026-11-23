//! The analytics beacon `POST /_hit` that the page script sends for each page view and each counted link click.

use std::net::SocketAddr;

use axum::{
    body::to_bytes,
    extract::{ConnectInfo, Request},
    http::{HeaderValue, StatusCode, header, request::Parts},
    response::{IntoResponse, Response},
};
use cuhlumn_api::{
    analytics::ClickKind,
    site::{host::Host, kind::SiteKind},
};
use serde::Deserialize;

use crate::{
    analytics::{
        Click, View,
        agent::{Agent, Client},
        referrer::referrer_host,
        unverified::{Target, page_path},
        visitor::client_ip,
    },
    db::{DbError, analytics::check, unix_now},
    state::AppState,
    web::{
        plain,
        request::{origin_host, request_host},
        route::{Route, decode_path},
    },
};

/// The path of the beacon. Slugs have no `_`, so no post has this path.
pub const HIT_PATH: &str = "/_hit";

/// The body of a beacon.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Beacon {
    View(ViewBeacon),
    Click(ClickBeacon),
}

/// The body of a page view.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ViewBeacon {
    /// The host name of the page, as `location.hostname` gives it. Only a page of an external site sends it.
    host: Option<String>,
    /// The page path, as `location.pathname` gives it.
    path: String,
    /// The referring address, as `document.referrer` gives it.
    referrer: String,
}

/// The body of a link click.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ClickBeacon {
    /// The host name of the page, as `location.hostname` gives it. Only a page of an external site sends it.
    host: Option<String>,
    click: ClickKind,
    /// The host name and the path of the link.
    target: String,
}

/// The site that a beacon belongs to, and the kind that the stored site must have.
struct Sender {
    host: Host,
    site: SiteKind,
}

/// The largest beacon body, in bytes.
const BODY_LIMIT: usize = 2 * 1024;

/// Counts a beacon when it is a page view or a link click of a reader on a site page.
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
    let user_agent = parts
        .headers
        .get(header::USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    if let Some(client) = Agent::classify(user_agent) {
        match beacon {
            Beacon::View(beacon) => view(state, &parts, beacon, client, user_agent).await,
            Beacon::Click(beacon) => click(state, &parts, beacon).await,
        }
    }
    let no_store = HeaderValue::from_static("no-store");
    (StatusCode::NO_CONTENT, [(header::CACHE_CONTROL, no_store)]).into_response()
}

/// Records a page view of a blog page that exists, or of a page of an external site.
async fn view(state: &AppState, parts: &Parts, beacon: ViewBeacon, client: Client, agent: &str) {
    let Some(Sender { host, site }) = sender(parts, beacon.host.as_deref()) else {
        return;
    };
    let (path, route) = match site {
        SiteKind::Blog => {
            let path = decode_path(&beacon.path).into_owned();
            let route = Route::parse(&path);
            if !matches!(route, Route::Home | Route::Entry(_) | Route::Tag(_)) {
                return;
            }
            (path, Some(route))
        }
        SiteKind::External => match page_path(&beacon.path) {
            Some(path) => (path, None),
            None => return,
        },
    };
    let name = host.clone();
    let checked = state
        .db()
        .call(move |db| match route {
            Some(route) => check::counts(db, &name, &route),
            None => check::is_site(db, &name, SiteKind::External),
        })
        .await;
    if !passed(checked, &host) {
        return;
    }
    let peer = parts
        .extensions
        .get::<ConnectInfo<SocketAddr>>()
        .map(|info| info.0);
    let view = View {
        referrer: referrer_host(&beacon.referrer, &host),
        ip: client_ip(&parts.headers, peer),
        user_agent: agent.to_owned(),
        host,
        site,
        path,
        client,
    };
    state.analytics().record(view, unix_now());
}

/// Records a click on an outbound link or on a download link.
///
/// An outbound link to the host of the site itself does not count.
async fn click(state: &AppState, parts: &Parts, beacon: ClickBeacon) {
    let Some(Sender { host, site }) = sender(parts, beacon.host.as_deref()) else {
        return;
    };
    let Some(target) = Target::parse(&beacon.target) else {
        return;
    };
    if beacon.click == ClickKind::Outbound && *target.host() == host {
        return;
    }
    let name = host.clone();
    let checked = state
        .db()
        .call(move |db| check::is_site(db, &name, site))
        .await;
    if !passed(checked, &host) {
        return;
    }
    let click = Click {
        host,
        kind: beacon.click,
        target: target.into_text(),
    };
    state.analytics().record_click(click, unix_now());
}

/// Returns the site that a beacon belongs to.
///
/// A beacon without a host name belongs to the blog on the request host.
/// A beacon with a host name belongs to the external site of that name, but only when the `Origin` header names the same host.
/// The browser sets `Origin`, so a page on a different website cannot count for the external site.
fn sender(parts: &Parts, named: Option<&str>) -> Option<Sender> {
    let Some(named) = named else {
        let host = request_host(&parts.headers, &parts.uri)?;
        return Some(Sender {
            host,
            site: SiteKind::Blog,
        });
    };
    let origin = parts.headers.get(header::ORIGIN)?.to_str().ok()?;
    let host = origin_host(origin)?;
    (Host::new(named).ok()? == host).then_some(Sender {
        host,
        site: SiteKind::External,
    })
}

/// Returns the result of a site or page check. A failed check logs the error and counts as `false`.
fn passed(checked: Result<bool, DbError>, host: &Host) -> bool {
    checked.unwrap_or_else(|err| {
        tracing::error!(%host, error = %err, "analytics beacon check failed");
        false
    })
}
