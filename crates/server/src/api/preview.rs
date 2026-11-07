//! The Markdown preview of the post editor.

use axum::extract::State;
use cuhlumn_api::wire::{Preview, PreviewInput};

use crate::{
    api::{auth::Admin, error::ApiError, json::Json, path::SitePath, sites::existing},
    render::{markdown, style::css::stylesheet},
    state::AppState,
};

/// Renders Markdown with the renderer of the public pages, and returns the stylesheet of the site.
///
/// # Errors
///
/// Returns [`ApiError::NotFound`] when the site does not exist.
pub async fn render(
    _admin: Admin,
    State(state): State<AppState>,
    SitePath(host): SitePath,
    Json(input): Json<PreviewInput>,
) -> Result<Json<Preview>, ApiError> {
    let site = state.db().call(move |db| existing(db, &host)).await?;
    let (doc, css) = tokio::task::spawn_blocking(move || {
        (markdown::render(&input.body), stylesheet(&site.brand))
    })
    .await?;
    Ok(Json(Preview {
        word_count: u32::try_from(doc.words).unwrap_or(u32::MAX),
        html: doc.html,
        toc: doc.toc,
        css,
    }))
}
