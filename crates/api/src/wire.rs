//! Small request and response bodies of the HTTP API.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::post::Post;

/// The response of `PUT /api/sites/{host}/posts/{slug}`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct PostSaved {
    /// The public address of the post.
    pub url: String,
    pub post: Post,
}

/// The response of `POST /api/sites/{host}/media`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct MediaSaved {
    /// The file name, such as `0123456789abcdef0123456789abcdef.png`.
    pub name: String,
    /// The public address of the file.
    pub url: String,
}

/// One uploaded image in `GET /api/sites/{host}/media`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct MediaItem {
    pub name: String,
    pub url: String,
    /// The MIME type, such as `image/png`.
    pub content_type: String,
    /// The file size in bytes.
    #[cfg_attr(feature = "ts", ts(type = "number"))]
    pub size: u64,
    /// The upload time, in unix seconds.
    #[cfg_attr(feature = "ts", ts(type = "number"))]
    pub created_at: i64,
}

/// The body of `POST /api/sites/{host}/preview`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct PreviewInput {
    /// The Markdown text to render.
    pub body: String,
}

/// The response of `POST /api/sites/{host}/preview`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct Preview {
    /// The rendered HTML.
    pub html: String,
    /// The table of contents.
    pub toc: Vec<TocEntry>,
    pub word_count: u32,
}

/// One heading in a table of contents.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct TocEntry {
    /// The heading level, from 2 to 6.
    pub level: u8,
    /// The `id` attribute of the heading.
    pub id: String,
    /// The heading text.
    pub label: String,
}

/// The body of `POST /api/session`.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct SessionInput {
    /// The admin token.
    pub token: String,
}

/// A console session.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct Session {
    /// The expiry time, in unix seconds.
    #[cfg_attr(feature = "ts", ts(type = "number"))]
    pub expires_at: i64,
}

/// The response of `GET /api/health`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct Health {
    pub ok: bool,
    /// The server version.
    pub version: String,
}

/// The body of every failed API response.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct ErrorBody {
    pub error: ErrorDetail,
}

/// The description of a failure.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct ErrorDetail {
    pub code: ErrorCode,
    /// A message for people.
    pub message: String,
    /// The JSON path of the invalid field, such as `brand.accent`.
    pub field: Option<String>,
}

/// The kind of a failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", ts(rename_all = "snake_case"))]
pub enum ErrorCode {
    /// The request is malformed, for example the JSON syntax is wrong.
    BadRequest,
    /// A field breaks a validation rule.
    Validation,
    /// The token or the session is missing or wrong.
    Unauthorized,
    /// The request comes from a foreign origin.
    Forbidden,
    /// The resource does not exist.
    NotFound,
    /// The method is not available for the path.
    MethodNotAllowed,
    /// The request conflicts with stored data.
    Conflict,
    /// The body is too large.
    PayloadTooLarge,
    /// The content type is not supported or does not match the body.
    UnsupportedMediaType,
    /// The server has no admin token, so the API is off.
    ApiDisabled,
    /// The console build is missing.
    ConsoleUnavailable,
    /// The server failed.
    Internal,
}

impl ErrorCode {
    /// Returns the name used in JSON, such as `not_found`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BadRequest => "bad_request",
            Self::Validation => "validation",
            Self::Unauthorized => "unauthorized",
            Self::Forbidden => "forbidden",
            Self::NotFound => "not_found",
            Self::MethodNotAllowed => "method_not_allowed",
            Self::Conflict => "conflict",
            Self::PayloadTooLarge => "payload_too_large",
            Self::UnsupportedMediaType => "unsupported_media_type",
            Self::ApiDisabled => "api_disabled",
            Self::ConsoleUnavailable => "console_unavailable",
            Self::Internal => "internal",
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Debug for SessionInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SessionInput")
            .field("token", &"<redacted>")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::wire::ErrorCode;

    #[test]
    fn error_code_names_match_json() {
        let codes = [
            ErrorCode::BadRequest,
            ErrorCode::Validation,
            ErrorCode::Unauthorized,
            ErrorCode::Forbidden,
            ErrorCode::NotFound,
            ErrorCode::MethodNotAllowed,
            ErrorCode::Conflict,
            ErrorCode::PayloadTooLarge,
            ErrorCode::UnsupportedMediaType,
            ErrorCode::ApiDisabled,
            ErrorCode::ConsoleUnavailable,
            ErrorCode::Internal,
        ];
        for code in codes {
            let json = serde_json::to_value(code).unwrap();
            assert_eq!(json.as_str(), Some(code.as_str()));
        }
    }
}
