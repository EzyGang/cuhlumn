//! Server that hosts many blogs, each on its own domain, with an admin HTTP API and a management console.

pub mod analytics;
pub mod api;
pub mod config;
pub mod console;
pub mod db;
pub mod fonts;
pub mod healthcheck;
pub mod indexnow;
pub mod media;
pub mod og;
pub mod render;
pub mod state;
pub mod web;

use axum::{
    Router,
    extract::{Request, State},
    http::{Method, StatusCode},
    response::Response,
};
use tower_http::trace::TraceLayer;

use crate::{
    fonts::FontName,
    render::style::font::FONTS_PATH,
    state::AppState,
    web::{request::request_host, route::decode_path},
};

/// Builds the router of the single listener, for every host.
///
/// All hosts get `/api`. The console hosts get the console for other paths, and the other hosts get the blogs.
pub fn router(state: AppState) -> Router {
    Router::new()
        .nest("/api", api::routes())
        .fallback(site_or_console)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// Builds the router of the public listener when an admin listener is on.
///
/// It serves the blogs and `GET /api/health`, and never the console or the admin API.
pub fn public_router(state: AppState) -> Router {
    Router::new()
        .nest("/api", api::public_routes())
        .fallback(site)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// Builds the router of the admin listener.
///
/// Every host gets `/api` and the console for other paths, without host routing.
pub fn admin_router(state: AppState) -> Router {
    Router::new()
        .nest("/api", api::routes())
        .fallback(console_only)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn site(State(state): State<AppState>, request: Request) -> Response {
    web::public(state, request).await
}

async fn console_only(State(state): State<AppState>, request: Request) -> Response {
    console_or_font(&state, request.method(), request.uri().path()).await
}

/// Answers a console request. GET and HEAD under `/_fonts/` get library font files, so the post preview can load blog fonts.
async fn console_or_font(state: &AppState, method: &Method, path: &str) -> Response {
    if let Some(name) = decode_path(path).strip_prefix(FONTS_PATH)
        && matches!(*method, Method::GET | Method::HEAD)
    {
        return match FontName::new(name) {
            Ok(name) => web::send_font(state, &name).await,
            Err(_) => web::plain(StatusCode::NOT_FOUND, "No such file.".to_owned()),
        };
    }
    console::serve(state, method, path)
}

async fn site_or_console(State(state): State<AppState>, request: Request) -> Response {
    let Some(host) = request_host(request.headers(), request.uri()) else {
        return web::public(state, request).await;
    };
    match state.serves_console(&host).await {
        Ok(true) => console_or_font(&state, request.method(), request.uri().path()).await,
        Ok(false) => web::public(state, request).await,
        Err(err) => {
            tracing::error!(%host, error = %err, "console host check failed");
            web::plain(
                StatusCode::INTERNAL_SERVER_ERROR,
                "The server failed. Try again later.".to_owned(),
            )
        }
    }
}
