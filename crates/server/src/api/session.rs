//! Console login, session check, and logout.

use axum::{
    extract::{FromRequestParts, State},
    http::{
        HeaderMap, HeaderValue, StatusCode, header, header::InvalidHeaderValue, request::Parts,
    },
    response::{IntoResponse, Response},
};
use cuhlumn_api::wire::{Session, SessionInput};

use crate::{
    api::{
        auth::{SESSION_COOKIE, check_origin, is_console_request, session_cookie},
        error::ApiError,
        json::Json,
    },
    config::{Scheme, Token},
    db::{sessions, unix_now},
    state::AppState,
};

/// Proof that the API is on and that the request addresses the console host.
#[derive(Clone, Debug)]
pub struct ConsoleHost {
    token: Token,
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
    if headers.contains_key(header::ORIGIN) {
        check_origin(&headers, &state)?;
    }
    if !console.token.matches(&input.token) {
        return Err(ApiError::Unauthorized);
    }
    let mut id = [0_u8; ID_BYTES];
    getrandom::fill(&mut id).map_err(ApiError::Random)?;
    let id = hex::encode(id);
    let id_hash = console.token.session_hash(&id);
    let now = unix_now();
    let expires_at = now + SESSION_SECONDS;
    state
        .db()
        .call(move |db| {
            sessions::delete_expired(db, now)?;
            sessions::create(db, &id_hash, expires_at)
        })
        .await?;
    let cookie = session_cookie_header(state.config().scheme, &id, SESSION_SECONDS)?;
    Ok(([(header::SET_COOKIE, cookie)], Json(Session { expires_at })).into_response())
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
        check_origin(&headers, &state)?;
        let id_hash = console.token.session_hash(id);
        state
            .db()
            .call(move |db| sessions::delete(db, &id_hash))
            .await?;
    }
    let cookie = session_cookie_header(state.config().scheme, "", 0)?;
    Ok((StatusCode::NO_CONTENT, [(header::SET_COOKIE, cookie)]).into_response())
}

const ID_BYTES: usize = 32;
const SESSION_SECONDS: i64 = 7 * 24 * 60 * 60;

impl FromRequestParts<AppState> for ConsoleHost {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = state.config().token().ok_or(ApiError::ApiDisabled)?;
        if !is_console_request(parts, state) {
            return Err(ApiError::NotFound(
                "sessions are available only on the console host".to_owned(),
            ));
        }
        Ok(Self {
            token: token.clone(),
        })
    }
}

fn session_cookie_header(
    scheme: Scheme,
    value: &str,
    max_age: i64,
) -> Result<HeaderValue, InvalidHeaderValue> {
    let secure = match scheme {
        Scheme::Https => "; Secure",
        Scheme::Http => "",
    };
    HeaderValue::try_from(format!(
        "{SESSION_COOKIE}={value}; HttpOnly; SameSite=Strict; Path=/; Max-Age={max_age}{secure}"
    ))
}
