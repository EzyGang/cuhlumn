//! Media handlers: upload, list, and delete images.

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode, header},
};
use cuhlumn_api::{
    site::host::Host,
    wire::{MediaItem, MediaSaved},
};

use crate::{
    api::{
        auth::Admin,
        error::ApiError,
        json::{Json, RawBody},
        path::{MediaPath, SitePath},
        sites::existing,
    },
    db::{media, unix_now},
    media::{format::ImageFormat, name::MediaName},
    state::AppState,
};

/// The largest upload, in bytes.
pub const UPLOAD_LIMIT: usize = 16 * 1024 * 1024;

/// Stores the raw image body. The `Content-Type` and the file signature must agree.
///
/// # Errors
///
/// Returns [`ApiError::UnsupportedMediaType`] for an unknown type or a signature mismatch,
/// and [`ApiError::NotFound`] when the site does not exist.
pub async fn upload(
    _admin: Admin,
    State(state): State<AppState>,
    SitePath(host): SitePath,
    headers: HeaderMap,
    RawBody(body): RawBody,
) -> Result<Json<MediaSaved>, ApiError> {
    let format = headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(ImageFormat::from_content_type)
        .ok_or_else(|| {
            ApiError::UnsupportedMediaType(
                "send a png, jpeg, webp, gif, or avif image with its Content-Type".to_owned(),
            )
        })?;
    if body.is_empty() {
        return Err(ApiError::BadRequest("the image is empty".to_owned()));
    }
    if !format.matches(&body) {
        return Err(ApiError::UnsupportedMediaType(format!(
            "the body is not a {} image",
            format.content_type()
        )));
    }
    let target = host.clone();
    state
        .db()
        .call(move |db| existing(db, &target).map(|_| ()))
        .await?;
    let name = state.media().save(&host, format, body.clone()).await?;
    let size = i64::try_from(body.len()).unwrap_or(i64::MAX);
    let record = name.clone();
    let target = host.clone();
    let inserted = state
        .db()
        .call(move |db| {
            existing(db, &target)?;
            media::insert(db, &target, &record, size, unix_now()).map_err(|err| {
                if err.is_foreign_key() {
                    ApiError::NotFound(format!("no site has the domain {target}"))
                } else {
                    ApiError::from(err)
                }
            })
        })
        .await;
    if let Err(err) = inserted {
        if matches!(err, ApiError::NotFound(_)) {
            remove_orphan(&state, &host, &name).await;
        }
        return Err(err);
    }
    state.cache().invalidate(&host);
    Ok(Json(MediaSaved {
        url: name.url(state.settings().scheme(), &host),
        name: name.into(),
    }))
}

/// Returns the images of a site, newest first.
///
/// # Errors
///
/// Returns [`ApiError::NotFound`] when the site does not exist.
pub async fn list(
    _admin: Admin,
    State(state): State<AppState>,
    SitePath(host): SitePath,
) -> Result<Json<Vec<MediaItem>>, ApiError> {
    let scheme = state.settings().scheme();
    let items = state
        .db()
        .call(move |db| {
            existing(db, &host)?;
            Ok::<_, ApiError>(media::list(db, scheme, &host)?)
        })
        .await?;
    Ok(Json(items))
}

/// Deletes one image record and its file.
///
/// # Errors
///
/// Returns [`ApiError::NotFound`] when the site or the image does not exist.
pub async fn delete(
    _admin: Admin,
    State(state): State<AppState>,
    MediaPath(host, name): MediaPath,
) -> Result<StatusCode, ApiError> {
    let (target, record) = (host.clone(), name.clone());
    state
        .db()
        .call(move |db| {
            existing(db, &target)?;
            if !media::delete(db, &target, &record)? {
                return Err(ApiError::NotFound(format!(
                    "the site {target} has no image {record}"
                )));
            }
            Ok(())
        })
        .await?;
    state.cache().invalidate(&host);
    state.media().remove(&host, &name).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Deletes a file that was saved for a site that a concurrent request deleted.
///
/// Other insert failures keep the file, because an earlier record can name the same content.
async fn remove_orphan(state: &AppState, host: &Host, name: &MediaName) {
    if let Err(err) = state.media().remove(host, name).await {
        tracing::warn!(%host, %name, error = %err, "orphan media file stays on disk");
    }
}
