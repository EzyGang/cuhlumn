//! The embedded management console, served only on the console host.

use std::borrow::Cow;

use axum::{
    body::Body,
    http::{HeaderValue, Method, StatusCode, header},
    response::{IntoResponse, Response},
};
use bytes::Bytes;
use rust_embed::{Embed, EmbeddedFile};

use crate::{config::Scheme, state::AppState, web::plain};

/// Answers a request for a console path that is not under `/api`.
///
/// Paths that name no embedded file get `index.html`, so the console can route them.
/// Missing files under `/assets/` get 404.
#[must_use]
pub fn serve(state: &AppState, method: &Method, path: &str) -> Response {
    let mut response = respond(state, method, path);
    let csp = match state.config().scheme {
        Scheme::Https => CSP_HTTPS,
        Scheme::Http => CSP_HTTP,
    };
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(csp),
    );
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("same-origin"),
    );
    response
}

/// Returns `true` when the console build output is present.
#[must_use]
pub fn is_built() -> bool {
    Assets::get(INDEX).is_some()
}

#[derive(Embed)]
#[folder = "../../console/dist"]
#[allow_missing = true]
struct Assets;

const INDEX: &str = "index.html";
const CSP_HTTPS: &str = "default-src 'self'; img-src 'self' https: data:; style-src 'self' 'unsafe-inline'; frame-ancestors 'none'";
const CSP_HTTP: &str = "default-src 'self'; img-src 'self' http: https: data:; style-src 'self' 'unsafe-inline'; frame-ancestors 'none'";
const IMMUTABLE: &str = "public, max-age=31536000, immutable";
const NO_CACHE: &str = "no-cache";

fn respond(state: &AppState, method: &Method, path: &str) -> Response {
    if state.config().token().is_none() {
        return plain(
            StatusCode::SERVICE_UNAVAILABLE,
            "The console is off until the server has CUHLUMN_TOKEN.".to_owned(),
        );
    }
    if !matches!(*method, Method::GET | Method::HEAD) {
        let mut response = plain(
            StatusCode::METHOD_NOT_ALLOWED,
            "Use GET or HEAD.".to_owned(),
        );
        response
            .headers_mut()
            .insert(header::ALLOW, HeaderValue::from_static("GET, HEAD"));
        return response;
    }
    let Some(index) = Assets::get(INDEX) else {
        return plain(
            StatusCode::SERVICE_UNAVAILABLE,
            "console_unavailable: the console is not built. Run `pnpm build` in the repository root."
                .to_owned(),
        );
    };
    let path = path.trim_start_matches('/');
    if path.is_empty() || path == INDEX {
        return file(index, NO_CACHE);
    }
    let unsafe_path = path
        .split('/')
        .any(|segment| segment == ".." || segment.is_empty())
        || path.contains('\\');
    if unsafe_path {
        return no_such_file();
    }
    if let Some(asset) = Assets::get(path) {
        let cache = if path.starts_with("assets/") {
            IMMUTABLE
        } else {
            NO_CACHE
        };
        return file(asset, cache);
    }
    if path.starts_with("assets/") {
        return no_such_file();
    }
    file(index, NO_CACHE)
}

fn no_such_file() -> Response {
    plain(
        StatusCode::NOT_FOUND,
        "The console has no such file.".to_owned(),
    )
}

fn file(asset: EmbeddedFile, cache: &'static str) -> Response {
    let content_type = HeaderValue::from_str(asset.metadata.mimetype())
        .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream"));
    let body = match asset.data {
        Cow::Borrowed(bytes) => Bytes::from_static(bytes),
        Cow::Owned(bytes) => Bytes::from(bytes),
    };
    (
        [
            (header::CONTENT_TYPE, content_type),
            (header::CACHE_CONTROL, HeaderValue::from_static(cache)),
        ],
        Body::from(body),
    )
        .into_response()
}
