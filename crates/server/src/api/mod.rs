//! The admin HTTP API under `/api`.

pub mod analytics;
pub mod auth;
pub mod error;
pub mod fonts;
pub mod json;
pub mod media;
pub mod path;
pub mod posts;
pub mod preview;
pub mod session;
pub mod settings;
pub mod sites;

use axum::{
    Router,
    extract::{DefaultBodyLimit, State},
    routing::{delete, get, post, put},
};
use cuhlumn_api::wire::Health;

use crate::{
    api::{error::ApiError, json::Json},
    fonts::UPLOAD_LIMIT as FONT_LIMIT,
    state::AppState,
};

/// Returns the API routes. Nest them under `/api`.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/setup", post(settings::setup))
        .route("/settings", get(settings::get).put(settings::put))
        .route("/settings/token", put(settings::put_token))
        .route(
            "/fonts",
            post(fonts::upload)
                .layer(DefaultBodyLimit::max(FONT_LIMIT))
                .get(fonts::list),
        )
        .route("/fonts/google", post(fonts::import_google))
        .route("/fonts/{name}", delete(fonts::delete))
        .route(
            "/session",
            post(session::login)
                .get(session::check)
                .delete(session::logout),
        )
        .route("/sites", get(sites::list))
        .route(
            "/sites/{host}",
            get(sites::get).put(sites::put).delete(sites::delete),
        )
        .route("/sites/{host}/posts", get(posts::list))
        .route(
            "/sites/{host}/posts/{slug}",
            get(posts::get).put(posts::put).delete(posts::delete),
        )
        .route(
            "/sites/{host}/media",
            post(media::upload)
                .layer(DefaultBodyLimit::max(media::UPLOAD_LIMIT))
                .get(media::list),
        )
        .route("/sites/{host}/media/{name}", delete(media::delete))
        .route("/sites/{host}/preview", post(preview::render))
        .route("/sites/{host}/analytics", get(analytics::get))
        .fallback(not_found)
        .method_not_allowed_fallback(method_not_allowed)
}

/// Returns the API routes of the public listener when an admin listener is on. Nest them under `/api`.
///
/// Only the health check is there. Every other path gets the JSON 404 response.
pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .fallback(not_found)
        .method_not_allowed_fallback(method_not_allowed)
}

async fn health(State(state): State<AppState>) -> Json<Health> {
    Json(Health {
        ok: true,
        version: env!("CARGO_PKG_VERSION").to_owned(),
        needs_setup: state.settings().needs_setup(),
    })
}

async fn not_found() -> ApiError {
    ApiError::NotFound("no API route has this path".to_owned())
}

async fn method_not_allowed() -> ApiError {
    ApiError::MethodNotAllowed
}
