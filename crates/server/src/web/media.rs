//! Uploaded images. Their names come from their content, so they never change.

use axum::response::Response;
use cuhlumn_api::site::host::Host;

use crate::{
    media::{MediaError, MediaStore, name::MediaName},
    web::stream,
};

/// Streams an uploaded image of the site at `host`. Returns `None` when the file does not exist.
///
/// # Errors
///
/// Returns [`MediaError`] when the file exists but cannot be opened.
pub async fn serve(
    media: &MediaStore,
    host: &Host,
    name: &MediaName,
) -> Result<Option<Response>, MediaError> {
    let Some(file) = media.open(host, name).await? else {
        return Ok(None);
    };
    Ok(Some(
        stream::immutable(file, name.format().content_type(), name).await,
    ))
}
