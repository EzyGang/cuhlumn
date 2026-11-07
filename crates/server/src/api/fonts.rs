//! Font library handlers: list, upload, Google Fonts import, and delete.

use axum::{
    extract::{FromRequestParts, Path, State},
    http::{StatusCode, request::Parts},
};
use bytes::Bytes;
use cuhlumn_api::fonts::{FontItem, FontSource, GoogleFontInput};

use crate::{
    api::{
        auth::Admin,
        error::ApiError,
        json::{Json, RawBody},
    },
    db::fonts,
    fonts::FontName,
    state::AppState,
};

/// The `{name}` parameter of a font route.
#[derive(Clone, Debug)]
pub struct FontPath(pub FontName);

/// Returns every library font, ordered by name.
///
/// # Errors
///
/// Returns [`ApiError`] when the database fails.
pub async fn list(
    _admin: Admin,
    State(state): State<AppState>,
) -> Result<Json<Vec<FontItem>>, ApiError> {
    let items = state.db().call(|db| fonts::list(db)).await?;
    Ok(Json(items))
}

/// Stores the raw body as a library font.
///
/// # Errors
///
/// Returns [`ApiError::Validation`] when the body is not a TTF, OTF, or TTC font.
pub async fn upload(
    _admin: Admin,
    State(state): State<AppState>,
    RawBody(body): RawBody,
) -> Result<Json<FontItem>, ApiError> {
    if body.is_empty() {
        return Err(ApiError::validation("file", "the font file is empty"));
    }
    let item = store(&state, body, FontSource::Upload).await?;
    state.fonts_changed().await?;
    Ok(Json(item))
}

/// Imports every font file of a Google Fonts specimen or stylesheet address.
///
/// # Errors
///
/// Returns [`ApiError::Validation`] for an address that is not Google Fonts, or for files that are not fonts.
pub async fn import_google(
    _admin: Admin,
    State(state): State<AppState>,
    Json(input): Json<GoogleFontInput>,
) -> Result<Json<Vec<FontItem>>, ApiError> {
    let files = state.fonts().download_google(input.url.as_str()).await?;
    let mut items = Vec::with_capacity(files.len());
    let mut failure = None;
    for file in files {
        match store(&state, file, FontSource::Google).await {
            Ok(item) => items.push(item),
            Err(err) => {
                failure = Some(err);
                break;
            }
        }
    }
    if !items.is_empty() {
        state.fonts_changed().await?;
    }
    match failure {
        Some(err) => Err(err),
        None => Ok(Json(items)),
    }
}

/// Deletes a library font and its file.
///
/// # Errors
///
/// Returns [`ApiError::NotFound`] when the library has no font with this name.
pub async fn delete(
    _admin: Admin,
    State(state): State<AppState>,
    FontPath(name): FontPath,
) -> Result<StatusCode, ApiError> {
    let record = name.as_str().to_owned();
    let deleted = state
        .db()
        .call(move |db| fonts::delete(db, &record))
        .await?;
    if !deleted {
        return Err(ApiError::NotFound(format!(
            "the font library has no font {}",
            name.as_str()
        )));
    }
    state.fonts().remove(&name).await?;
    state.fonts_changed().await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Writes a font file and records it. A failed record removes the file again.
async fn store(state: &AppState, bytes: Bytes, source: FontSource) -> Result<FontItem, ApiError> {
    let item = state.fonts().save(bytes, source).await?;
    let record = item.clone();
    let inserted = state.db().call(move |db| fonts::insert(db, &record)).await;
    if let Err(err) = inserted {
        if let Ok(name) = FontName::new(&item.name)
            && let Err(cleanup) = state.fonts().remove(&name).await
        {
            tracing::warn!(name = %item.name, error = %cleanup, "unrecorded font file stays on disk");
        }
        return Err(ApiError::from(err));
    }
    Ok(item)
}

impl<S> FromRequestParts<S> for FontPath
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Path(name) = Path::<String>::from_request_parts(parts, state)
            .await
            .map_err(|rejection| ApiError::BadRequest(rejection.body_text()))?;
        let name = FontName::new(&name).map_err(|err| ApiError::validation("name", err))?;
        Ok(Self(name))
    }
}
