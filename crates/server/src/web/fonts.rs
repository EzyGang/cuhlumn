//! Library font files for blog stylesheets. Their names contain a hash of their content, so they never change.

use axum::{
    http::{HeaderValue, header},
    response::Response,
};

use crate::{
    fonts::{FontError, FontName, FontStore},
    web::stream,
};

/// Streams a library font file. Returns `None` when the file does not exist.
///
/// Every origin may read the file, because the sandboxed console preview loads it from an opaque origin.
///
/// # Errors
///
/// Returns [`FontError`] when the file exists but cannot be opened.
pub async fn serve(fonts: &FontStore, name: &FontName) -> Result<Option<Response>, FontError> {
    let Some(file) = fonts.open(name).await? else {
        return Ok(None);
    };
    let mut response = stream::immutable(file, name.content_type(), name).await;
    response.headers_mut().insert(
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        HeaderValue::from_static("*"),
    );
    Ok(Some(response))
}
