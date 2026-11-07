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
    web::request::{origin_host, request_authority, request_host},
};

/// Proof that the request has admin access.
///
/// A cookie request with an unsafe method must also send an `Origin` header of its own console.
#[derive(Clone, Copy, Debug)]
pub struct Admin;

/// The console that a request addresses.
///
/// The `Origin` header of a cookie request must name it.
#[derive(Clone, Debug)]
pub enum Console {
    /// A console host on the single listener. The `Origin` host must equal it, with any port.
    Host(Host),
    /// The admin listener, on any host. The `Origin` authority must equal the `Host` header, port included.
    AdminListener(String),
}

/// The console of the request, or `None` when the request does not address the console.
#[derive(Clone, Debug)]
pub struct ConsoleRequest(pub Option<Console>);

/// The name of the console session cookie.
pub const SESSION_COOKIE: &str = "cuhlumn_session";

/// Returns the console that the request addresses.
///
/// With an admin listener, every request with a `Host` header does, because only the admin listener serves `/api`.
///
/// # Errors
///
/// Returns [`ApiError::Database`] when the site lookup fails.
pub async fn console(
    headers: &HeaderMap,
    uri: &Uri,
    state: &AppState,
) -> Result<Option<Console>, ApiError> {
    if state.settings().admin_listener() {
        let authority = request_authority(headers, uri).map(str::to_ascii_lowercase);
        return Ok(authority.map(Console::AdminListener));
    }
    let Some(host) = request_host(headers, uri) else {
        return Ok(None);
    };
    Ok(state
        .serves_console(&host)
        .await?
        .then_some(Console::Host(host)))
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

impl Console {
    /// Checks that the `Origin` header names this console.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::Forbidden`] when the header is missing or names another host.
    pub fn check_origin(&self, headers: &HeaderMap) -> Result<(), ApiError> {
        let origin = headers
            .get(header::ORIGIN)
            .and_then(|value| value.to_str().ok());
        let matches = match self {
            Self::Host(console) => origin
                .and_then(origin_host)
                .is_some_and(|host| host == *console),
            Self::AdminListener(authority) => origin
                .and_then(|origin| origin.split_once("://"))
                .is_some_and(|(_, sent)| sent.eq_ignore_ascii_case(authority)),
        };
        if !matches {
            return Err(ApiError::Forbidden);
        }
        Ok(())
    }

    /// Checks the `Origin` header like [`Console::check_origin`], but only when the request sends one.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::Forbidden`] when the header names another host.
    pub fn check_origin_if_sent(&self, headers: &HeaderMap) -> Result<(), ApiError> {
        match headers.contains_key(header::ORIGIN) {
            true => self.check_origin(headers),
            false => Ok(()),
        }
    }
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
        let Some(console) = console(&parts.headers, &parts.uri, state).await? else {
            return Err(ApiError::Unauthorized);
        };
        if !parts.method.is_safe() {
            console.check_origin(&parts.headers)?;
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
        Ok(Self(console(&parts.headers, &parts.uri, state).await?))
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

    use crate::api::auth::{Authorization, Console, authorization, session_cookie};

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

    #[test]
    fn admin_listener_origin_must_match_the_host_header() {
        let console = Console::AdminListener("100.92.3.4:8081".to_owned());
        let origin = |value: &'static str| {
            let mut headers = HeaderMap::new();
            headers.insert(header::ORIGIN, HeaderValue::from_static(value));
            headers
        };
        assert!(
            console
                .check_origin(&origin("http://100.92.3.4:8081"))
                .is_ok()
        );
        assert!(
            console
                .check_origin(&origin("HTTPS://100.92.3.4:8081"))
                .is_ok()
        );
        assert!(
            console
                .check_origin(&origin("http://100.92.3.4:9000"))
                .is_err()
        );
        assert!(console.check_origin(&origin("null")).is_err());
        assert!(console.check_origin(&HeaderMap::new()).is_err());
        assert!(console.check_origin_if_sent(&HeaderMap::new()).is_ok());
    }
}
