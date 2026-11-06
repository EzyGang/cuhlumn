//! Uploaded images. Their names come from their content, so they never change.

use axum::{
    body::Body,
    http::{HeaderValue, header},
    response::{IntoResponse, Response},
};
use cuhlumn_api::site::host::Host;
use tokio_util::io::ReaderStream;

use crate::media::{MediaError, MediaStore, name::MediaName};

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
    let length = match file.metadata().await {
        Ok(metadata) => Some(metadata.len()),
        Err(err) => {
            tracing::warn!(%host, %name, error = %err, "media file size is unknown, the response is chunked");
            None
        }
    };
    let mut response = Body::from_stream(ReaderStream::new(file)).into_response();
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static(name.format().content_type()),
    );
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=31536000, immutable"),
    );
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    if let Some(length) = length {
        headers.insert(header::CONTENT_LENGTH, HeaderValue::from(length));
    }
    Ok(Some(response))
}
