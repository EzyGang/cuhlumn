//! Console login, session check, and logout.

use axum::{
    extract::{FromRequestParts, State},
    http::{
        HeaderMap, HeaderValue, StatusCode, header, header::InvalidHeaderValue, request::Parts,
    },
    response::{IntoResponse, Response},
};
use cuhlumn_api::{
    settings::Scheme,
    wire::{Session, SessionInput},
};

use crate::{
    api::{
        auth::{Console, SESSION_COOKIE, console, session_cookie},
        error::ApiError,
        json::Json,
    },
    config::Token,
    db::{sessions, unix_now},
    state::AppState,
};

/// Proof that the server has a token and that the request addresses the console.
#[derive(Clone, Debug)]
pub struct ConsoleHost {
    token: Token,
    console: Console,
}

/// Logs in with the admin token and sets the session cookie.
///
/// # Errors
///
/// Returns [`ApiError::Unauthorized`] for a wrong token and [`ApiError::Forbidden`] for a foreign `Origin`.
pub async fn login(
    console: ConsoleHost,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<SessionInput>,
) -> Result<Response, ApiError> {
    console.console.check_origin_if_sent(&headers)?;
    if !console.token.matches(&input.token) {
        return Err(ApiError::Unauthorized);
    }
    start(&state, &console.token, Some(&headers)).await
}

/// Returns the session of the cookie.
///
/// # Errors
///
/// Returns [`ApiError::Unauthorized`] when the cookie is missing, unknown, or expired.
pub async fn check(
    console: ConsoleHost,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Session>, ApiError> {
    let id_hash = session_cookie(&headers)
        .map(|id| console.token.session_hash(id))
        .ok_or(ApiError::Unauthorized)?;
    let expires_at = state
        .db()
        .call(move |db| sessions::find(db, &id_hash, unix_now()))
        .await?
        .ok_or(ApiError::Unauthorized)?;
    Ok(Json(Session { expires_at }))
}

/// Deletes the session of the cookie and clears the cookie.
///
/// # Errors
///
/// Returns [`ApiError::Forbidden`] when a cookie request has a foreign or no `Origin`.
pub async fn logout(
    console: ConsoleHost,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    if let Some(id) = session_cookie(&headers) {
        console.console.check_origin(&headers)?;
        let id_hash = console.token.session_hash(id);
        state
            .db()
            .call(move |db| sessions::delete(db, &id_hash))
            .await?;
    }
    let cookie = session_cookie_header(secure_cookie(&state, &headers), "", 0)?;
    Ok((StatusCode::NO_CONTENT, [(header::SET_COOKIE, cookie)]).into_response())
}

/// Creates a session for `token` and returns its [`Session`] body.
///
/// The response sets the session cookie only when `cookie` has the request headers, which callers give for console requests.
/// The headers decide the `Secure` flag on the admin listener.
///
/// # Errors
///
/// Returns [`ApiError`] when the random source or the database fails.
pub async fn start(
    state: &AppState,
    token: &Token,
    cookie: Option<&HeaderMap>,
) -> Result<Response, ApiError> {
    let mut id = [0_u8; ID_BYTES];
    getrandom::fill(&mut id).map_err(ApiError::Random)?;
    let id = hex::encode(id);
    let id_hash = token.session_hash(&id);
    let now = unix_now();
    let expires_at = now + SESSION_SECONDS;
    state
        .db()
        .call(move |db| {
            sessions::delete_expired(db, now)?;
            sessions::create(db, &id_hash, expires_at)
        })
        .await?;
    let body = Json(Session { expires_at });
    let Some(headers) = cookie else {
        return Ok(body.into_response());
    };
    let secure = secure_cookie(state, headers);
    let cookie = session_cookie_header(secure, &id, SESSION_SECONDS)?;
    Ok(([(header::SET_COOKIE, cookie)], body).into_response())
}

const ID_BYTES: usize = 32;
const SESSION_SECONDS: i64 = 7 * 24 * 60 * 60;

impl FromRequestParts<AppState> for ConsoleHost {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = state
            .settings()
            .token()
            .cloned()
            .ok_or(ApiError::NeedsSetup)?;
        let Some(console) = console(&parts.headers, &parts.uri, state).await? else {
            return Err(ApiError::NotFound(
                "sessions are available only on the console host".to_owned(),
            ));
        };
        Ok(Self { token, console })
    }
}

/// Returns `true` when the session cookie needs the `Secure` flag.
///
/// On the admin listener, plain HTTP is permitted, so only `X-Forwarded-Proto: https` from a TLS proxy sets the flag.
/// On the single listener, the scheme setting decides.
fn secure_cookie(state: &AppState, headers: &HeaderMap) -> bool {
    let settings = state.settings();
    if !settings.admin_listener() {
        return settings.scheme() == Scheme::Https;
    }
    headers
        .get(FORWARDED_PROTO)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .is_some_and(|proto| proto.trim().eq_ignore_ascii_case("https"))
}

const FORWARDED_PROTO: &str = "x-forwarded-proto";

fn session_cookie_header(
    secure: bool,
    value: &str,
    max_age: i64,
) -> Result<HeaderValue, InvalidHeaderValue> {
    let secure = match secure {
        true => "; Secure",
        false => "",
    };
    HeaderValue::try_from(format!(
        "{SESSION_COOKIE}={value}; HttpOnly; SameSite=Strict; Path=/; Max-Age={max_age}{secure}"
    ))
}
