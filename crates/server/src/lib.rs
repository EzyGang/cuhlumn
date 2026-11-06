//! Server that hosts many blogs, each on its own domain, with an admin HTTP API and a management console.

pub mod api;
pub mod config;
pub mod console;
pub mod db;
pub mod indexnow;
pub mod media;
pub mod og;
pub mod render;
pub mod state;
pub mod web;

use axum::{
    Router,
    extract::{Request, State},
    response::Response,
};
use tower_http::trace::TraceLayer;

use crate::{state::AppState, web::request::request_host};

/// Builds the router for every host.
///
/// All hosts get `/api`. The console host gets the console for other paths; other hosts get the blogs.
pub fn router(state: AppState) -> Router {
    Router::new()
        .nest("/api", api::routes())
        .fallback(site_or_console)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn site_or_console(State(state): State<AppState>, request: Request) -> Response {
    let console = request_host(request.headers(), request.uri())
        .is_some_and(|host| state.config().is_console_host(&host));
    if console {
        return console::serve(&state, request.method(), request.uri().path());
    }
    web::public(state, request).await
}
