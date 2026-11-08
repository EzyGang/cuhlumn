//! Responses for files whose names come from their content, so they never change.

use std::fmt::Display;

use axum::{
    body::Body,
    http::{HeaderValue, header},
    response::{IntoResponse, Response},
};
use tokio::fs::File;
use tokio_util::io::ReaderStream;

/// Streams `file` with `content_type` and a one-year immutable cache header.
///
/// `name` names the file in the warning when its size is unknown.
pub async fn immutable(
    file: File,
    content_type: &'static str,
    name: &(dyn Display + Sync),
) -> Response {
    let length = match file.metadata().await {
        Ok(metadata) => Some(metadata.len()),
        Err(err) => {
            tracing::warn!(%name, error = %err, "file size is unknown, the response is chunked");
            None
        }
    };
    let mut response = Body::from_stream(ReaderStream::new(file)).into_response();
    let headers = response.headers_mut();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
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
    response
}
