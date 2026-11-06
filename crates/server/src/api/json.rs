//! Request body extractors that report failures as [`ApiError`].

use axum::{
    extract::{FromRequest, Request, rejection::BytesRejection},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
};
use bytes::Bytes;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::error::Category;

use crate::api::error::ApiError;

/// A JSON request or response body.
///
/// As an extractor, it needs `Content-Type: application/json` and reports the JSON path of an invalid field.
#[derive(Clone, Copy, Debug, Default)]
pub struct Json<T>(pub T);

/// The raw request body.
#[derive(Clone, Debug)]
pub struct RawBody(pub Bytes);

impl<T, S> FromRequest<S> for Json<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        if !is_json(request.headers()) {
            return Err(ApiError::UnsupportedMediaType(
                "send the body as JSON with Content-Type: application/json".to_owned(),
            ));
        }
        let RawBody(bytes) = RawBody::from_request(request, state).await?;
        let mut deserializer = serde_json::Deserializer::from_slice(&bytes);
        let value = serde_path_to_error::deserialize(&mut deserializer).map_err(json_error)?;
        deserializer
            .end()
            .map_err(|err| ApiError::BadRequest(format!("the body is not valid JSON: {err}")))?;
        Ok(Self(value))
    }
}

impl<T> IntoResponse for Json<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        axum::Json(self.0).into_response()
    }
}

impl<S> FromRequest<S> for RawBody
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        Bytes::from_request(request, state)
            .await
            .map(Self)
            .map_err(body_error)
    }
}

fn is_json(headers: &HeaderMap) -> bool {
    let Some(value) = headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
    else {
        return false;
    };
    let essence = value.split(';').next().unwrap_or_default().trim();
    essence.eq_ignore_ascii_case("application/json")
        || essence.to_ascii_lowercase().ends_with("+json")
}

fn body_error(rejection: BytesRejection) -> ApiError {
    if rejection.status() == StatusCode::PAYLOAD_TOO_LARGE {
        return ApiError::PayloadTooLarge;
    }
    ApiError::BadRequest(rejection.body_text())
}

fn json_error(err: serde_path_to_error::Error<serde_json::Error>) -> ApiError {
    let path = err.path().to_string();
    let inner = err.into_inner();
    if inner.classify() != Category::Data {
        return ApiError::BadRequest(format!("the body is not valid JSON: {inner}"));
    }
    let text = inner.to_string();
    let message = text
        .rfind(" at line ")
        .and_then(|end| text.get(..end))
        .unwrap_or(&text);
    let field = match missing_field(message) {
        Some(name) if path == "." => Some(name.to_owned()),
        Some(name) => Some(format!("{path}.{name}")),
        None if path == "." => None,
        None => Some(path),
    };
    let message = match &field {
        Some(field) => format!("{field}: {message}"),
        None => message.to_owned(),
    };
    ApiError::Validation { field, message }
}

/// Returns the field name of a missing-field message.
///
/// For a missing field, the path names the parent object, so the field name must be added.
fn missing_field(message: &str) -> Option<&str> {
    let rest = message.strip_prefix("missing field `")?;
    rest.split_once('`').map(|(name, _)| name)
}

#[cfg(test)]
mod tests {
    use crate::api::json::missing_field;

    #[test]
    fn missing_fields_come_from_serde_messages() {
        assert_eq!(missing_field("missing field `title`"), Some("title"));
        assert_eq!(
            missing_field("unknown field `accent`, expected one of `title`"),
            None
        );
        assert_eq!(missing_field("invalid type: integer"), None);
    }
}
