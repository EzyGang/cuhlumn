//! The admin HTTP API under `/api`.

pub mod auth;
pub mod error;
pub mod json;
pub mod media;
pub mod path;
pub mod posts;
pub mod preview;
pub mod session;
pub mod sites;

use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{delete, get, post},
};
use cuhlumn_api::wire::Health;

use crate::{
    api::{error::ApiError, json::Json},
    state::AppState,
};

/// Returns the API routes. Nest them under `/api`.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
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
        .fallback(not_found)
        .method_not_allowed_fallback(method_not_allowed)
}

async fn health() -> Json<Health> {
    Json(Health {
        ok: true,
        version: env!("CARGO_PKG_VERSION").to_owned(),
    })
}

async fn not_found() -> ApiError {
    ApiError::NotFound("no API route has this path".to_owned())
}

async fn method_not_allowed() -> ApiError {
    ApiError::MethodNotAllowed
}
