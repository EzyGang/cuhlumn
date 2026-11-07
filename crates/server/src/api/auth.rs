//! Admin authentication: a bearer token on any host, or a console session cookie on a console host.

use axum::{
    extract::FromRequestParts,
    http::{HeaderMap, HeaderValue, Uri, header, request::Parts},
};
use cuhlumn_api::site::host::Host;

use crate::{
    api::error::ApiError,
    db::{sessions, unix_now},
    state::AppState,
    web::request::{origin_host, request_host},
};

/// Proof that the request has admin access.
///
/// A cookie request with an unsafe method must also send an `Origin` header of its own console host.
#[derive(Clone, Copy, Debug)]
pub struct Admin;

/// The request host when it serves the console, else `None`.
#[derive(Clone, Debug)]
pub struct ConsoleRequest(pub Option<Host>);

/// The name of the console session cookie.
pub const SESSION_COOKIE: &str = "cuhlumn_session";

/// Returns the request host when it serves the console.
///
/// # Errors
///
/// Returns [`ApiError::Database`] when the site lookup fails.
pub async fn console_host(
    headers: &HeaderMap,
    uri: &Uri,
    state: &AppState,
) -> Result<Option<Host>, ApiError> {
    let Some(host) = request_host(headers, uri) else {
        return Ok(None);
    };
    Ok(state.serves_console(&host).await?.then_some(host))
}

/// Returns the value of the session cookie.
#[must_use]
pub fn session_cookie(headers: &HeaderMap) -> Option<&str> {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .find_map(|pair| {
            let (name, value) = pair.trim().split_once('=')?;
            (name == SESSION_COOKIE && !value.is_empty()).then_some(value)
        })
}

/// Checks that the `Origin` header names `console`, the console host of the request.
///
/// # Errors
///
/// Returns [`ApiError::Forbidden`] when the header is missing or names another host.
pub fn check_origin(headers: &HeaderMap, console: &Host) -> Result<(), ApiError> {
    let matches = headers
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok())
        .and_then(origin_host)
        .is_some_and(|host| host == *console);
    if !matches {
        return Err(ApiError::Forbidden);
    }
    Ok(())
}

/// The credentials in an `Authorization` header.
#[derive(Debug, PartialEq, Eq)]
enum Authorization<'a> {
    /// A Bearer token. A malformed Bearer value gives an empty token, which never matches.
    Bearer(&'a str),
    /// Another scheme, such as `Basic` from a reverse proxy. The cookie path decides.
    Other,
}

impl FromRequestParts<AppState> for Admin {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let settings = state.settings();
        let token = settings.token().ok_or(ApiError::NeedsSetup)?;
        if let Some(Authorization::Bearer(candidate)) =
            parts.headers.get(header::AUTHORIZATION).map(authorization)
        {
            if !token.matches(candidate) {
                return Err(ApiError::Unauthorized);
            }
            return Ok(Self);
        }
        let Some(id) = session_cookie(&parts.headers) else {
            return Err(ApiError::Unauthorized);
        };
        let Some(console) = console_host(&parts.headers, &parts.uri, state).await? else {
            return Err(ApiError::Unauthorized);
        };
        if !parts.method.is_safe() {
            check_origin(&parts.headers, &console)?;
        }
        let id_hash = token.session_hash(id);
        let session = state
            .db()
            .call(move |db| sessions::find(db, &id_hash, unix_now()))
            .await?;
        session.map(|_| Self).ok_or(ApiError::Unauthorized)
    }
}

impl FromRequestParts<AppState> for ConsoleRequest {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self(console_host(&parts.headers, &parts.uri, state).await?))
    }
}

fn authorization(value: &HeaderValue) -> Authorization<'_> {
    let Ok(value) = value.to_str() else {
        return Authorization::Bearer("");
    };
    let (scheme, rest) = value.split_once(' ').unwrap_or((value, ""));
    if !scheme.eq_ignore_ascii_case("bearer") {
        return Authorization::Other;
    }
    Authorization::Bearer(rest.trim())
}

#[cfg(test)]
mod tests {
    use axum::http::{HeaderMap, HeaderValue, header};

    use crate::api::auth::{Authorization, authorization, session_cookie};

    #[test]
    fn reads_authorization_schemes() {
        let cases = [
            ("Bearer abc", Authorization::Bearer("abc")),
            ("bearer abc", Authorization::Bearer("abc")),
            ("Bearer", Authorization::Bearer("")),
            ("Basic abc", Authorization::Other),
        ];
        for (value, expected) in cases {
            assert_eq!(authorization(&HeaderValue::from_static(value)), expected);
        }
        let binary = HeaderValue::from_bytes(b"Bearer \xff").unwrap();
        assert_eq!(authorization(&binary), Authorization::Bearer(""));
    }

    #[test]
    fn reads_the_session_cookie() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_static("a=1; cuhlumn_session=xyz; b=2"),
        );
        assert_eq!(session_cookie(&headers), Some("xyz"));
        headers.insert(header::COOKIE, HeaderValue::from_static("cuhlumn_session="));
        assert_eq!(session_cookie(&headers), None);
    }
}
