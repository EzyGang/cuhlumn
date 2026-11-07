//! The public side: each request gets the blog that its host names.

pub mod cache;
mod media;
mod pages;
pub mod request;
pub mod route;

use std::sync::Arc;

use axum::{
    extract::Request,
    http::{HeaderValue, Method, StatusCode, header},
    response::{IntoResponse, Response},
};
use cuhlumn_api::{
    post::Post,
    settings::Scheme,
    site::{Site, host::Host},
};
use rusqlite::Connection;

use crate::{
    db::{
        DbError,
        posts::{self, Visibility},
        sites, unix_now,
    },
    media::name::MediaName,
    render::{blog::Blog, not_found},
    state::AppState,
    web::{
        pages::Outcome,
        request::request_host,
        route::{Route, decode_path},
    },
};

/// Answers a GET or HEAD request to a blog host or to an unknown host.
///
/// Successful pages come from the page cache, or are built once and stored there.
/// An image request loads only the site, not its posts.
pub async fn public(state: AppState, request: Request) -> Response {
    let (parts, _body) = request.into_parts();
    if !matches!(parts.method, Method::GET | Method::HEAD) {
        let mut response = plain(
            StatusCode::METHOD_NOT_ALLOWED,
            "Use GET or HEAD.".to_owned(),
        );
        response
            .headers_mut()
            .insert(header::ALLOW, HeaderValue::from_static("GET, HEAD"));
        return response;
    }
    let Some(host) = request_host(&parts.headers, &parts.uri) else {
        return no_blog();
    };
    let path = decode_path(parts.uri.path()).into_owned();
    if let Some(hit) = state.cache().get(&host, &path, unix_now()) {
        return hit.respond(&parts.headers);
    }
    let route = Route::parse(&path);
    let generation = state.cache().generation(&host);
    let settings = state.settings();
    let scheme = settings.scheme();
    let with_posts = !matches!(route, Route::Media(_));
    let name = host.clone();
    let load = move |db: &mut Connection| lookup(db, scheme, &name, with_posts);
    let (site, posts) = match state.db().call(load).await {
        Ok(Some((site, _))) if site.host != host => {
            let target = parts
                .uri
                .path_and_query()
                .map_or("/", |value| value.as_str());
            return redirect(&format!("{}{target}", settings.origin(&site.host)));
        }
        Ok(Some(found)) => found,
        Ok(None) => return no_blog(),
        Err(err) => {
            tracing::error!(%host, error = %err, "blog lookup failed");
            return server_error();
        }
    };
    if let Route::Media(name) = &route {
        return send_media(&state, &host, name).await;
    }
    let blog = Arc::new(Blog::new(site, settings.origin(&host), posts));
    let query = parts.uri.query().map(str::to_owned);
    match pages::build(&state, Arc::clone(&blog), route, query).await {
        Ok(Outcome::Page(cached)) => {
            let cached = Arc::new(cached);
            let response = cached.respond(&parts.headers);
            state.cache().insert(&host, path, generation, cached);
            response
        }
        Ok(Outcome::Redirect(location)) => redirect(&location),
        Ok(Outcome::Missing) => missing(blog).await,
        Err(err) => {
            tracing::error!(%host, %path, error = %err, "page build failed");
            server_error()
        }
    }
}

/// Returns a plain text response.
#[must_use]
pub fn plain(status: StatusCode, text: String) -> Response {
    let content_type = HeaderValue::from_static("text/plain; charset=utf-8");
    (status, [(header::CONTENT_TYPE, content_type)], text).into_response()
}

/// Loads the site that `host` names, as its canonical host or an alias, in one database call.
///
/// The published posts come only when `with_posts` is set and `host` is the canonical host.
/// Otherwise the list is empty.
fn lookup(
    db: &Connection,
    scheme: Scheme,
    host: &Host,
    with_posts: bool,
) -> Result<Option<(Site, Vec<Post>)>, DbError> {
    let Some(site) = sites::resolve(db, host)? else {
        return Ok(None);
    };
    let posts = match with_posts && site.host == *host {
        true => posts::list(db, scheme, host, Visibility::Published)?,
        false => Vec::new(),
    };
    Ok(Some((site, posts)))
}

/// Streams an uploaded image, or answers 404 in plain text when it does not exist.
async fn send_media(state: &AppState, host: &Host, name: &MediaName) -> Response {
    match media::serve(state.media(), host, name).await {
        Ok(Some(response)) => response,
        Ok(None) => plain(StatusCode::NOT_FOUND, "No such file.".to_owned()),
        Err(err) => {
            tracing::error!(%host, error = %err, "media file cannot be opened");
            server_error()
        }
    }
}

/// Returns a 301 response. Callers give an ASCII location, so the header is always present.
fn redirect(location: &str) -> Response {
    match HeaderValue::from_str(location) {
        Ok(value) => (StatusCode::MOVED_PERMANENTLY, [(header::LOCATION, value)]).into_response(),
        Err(err) => {
            tracing::error!(location, error = %err, "redirect location is not a valid header");
            server_error()
        }
    }
}

/// Returns the HTML "Page not found" page, rendered on the blocking thread pool.
async fn missing(blog: Arc<Blog>) -> Response {
    let body = match tokio::task::spawn_blocking(move || not_found::render(&blog)).await {
        Ok(body) => body,
        Err(err) => {
            tracing::error!(error = %err, "not found page build failed");
            return server_error();
        }
    };
    let content_type = HeaderValue::from_static("text/html; charset=utf-8");
    (
        StatusCode::NOT_FOUND,
        [(header::CONTENT_TYPE, content_type)],
        body,
    )
        .into_response()
}

fn no_blog() -> Response {
    plain(
        StatusCode::NOT_FOUND,
        "No blog lives at this address.".to_owned(),
    )
}

fn server_error() -> Response {
    plain(
        StatusCode::INTERNAL_SERVER_ERROR,
        "The server failed. Try again later.".to_owned(),
    )
}
