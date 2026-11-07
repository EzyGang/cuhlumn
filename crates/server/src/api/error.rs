//! API failures and their JSON responses.

use std::fmt;

use axum::{
    Json,
    http::{StatusCode, header::InvalidHeaderValue},
    response::{IntoResponse, Response},
};
use cuhlumn_api::{
    site::indexnow_key::IndexNowKeyError,
    wire::{ErrorBody, ErrorCode, ErrorDetail},
};

use crate::{db::DbError, fonts::FontError, media::MediaError};

/// A failed API request. Every variant becomes an [`ErrorBody`] response.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    /// The request is malformed.
    #[error("{0}")]
    BadRequest(String),
    /// A value breaks a validation rule.
    #[error("{message}")]
    Validation {
        /// The JSON path or the path parameter of the invalid value.
        field: Option<String>,
        message: String,
    },
    /// The token or the session is missing or wrong.
    #[error("send the admin token as a bearer token, or log in to the console")]
    Unauthorized,
    /// The request comes from a foreign origin.
    #[error("the request origin is not the console host")]
    Forbidden,
    /// The resource does not exist.
    #[error("{0}")]
    NotFound(String),
    /// The method is not available for the path.
    #[error("this method is not available for this path")]
    MethodNotAllowed,
    /// The request conflicts with stored data.
    #[error("{0}")]
    Conflict(String),
    /// The value of `field` conflicts with stored data.
    #[error("{message}")]
    FieldConflict {
        /// The JSON path of the field.
        field: String,
        message: String,
    },
    /// The body is larger than the limit of the route.
    #[error("the request body is too large")]
    PayloadTooLarge,
    /// The content type is not supported or does not match the body.
    #[error("{0}")]
    UnsupportedMediaType(String),
    /// The server has no admin token yet.
    #[error("the server has no admin token yet")]
    NeedsSetup,
    /// An environment variable sets the value of `field`.
    #[error("an environment variable sets this value")]
    Locked {
        /// The JSON path of the locked field.
        field: String,
    },
    /// The database failed.
    #[error(transparent)]
    Database(#[from] DbError),
    /// The font library failed.
    #[error(transparent)]
    Fonts(FontError),
    /// The media storage failed.
    #[error(transparent)]
    Media(#[from] MediaError),
    /// The operating system random source failed.
    #[error("the random source failed: {0}")]
    Random(getrandom::Error),
    /// A response header value is not valid.
    #[error("a response header is not valid: {0}")]
    Header(#[from] InvalidHeaderValue),
    /// A generated IndexNow key is not valid.
    #[error("a generated IndexNow key is not valid: {0}")]
    Key(#[from] IndexNowKeyError),
    /// A blocking task did not finish.
    #[error("a blocking task failed: {0}")]
    Task(#[from] tokio::task::JoinError),
}

const INTERNAL_MESSAGE: &str = "the server failed; try again later";

impl ApiError {
    /// Returns a validation failure for `field`.
    #[must_use]
    pub fn validation<E>(field: &str, err: E) -> Self
    where
        E: fmt::Display,
    {
        Self::Validation {
            field: Some(field.to_owned()),
            message: format!("{field}: {err}"),
        }
    }

    fn status_and_code(&self) -> (StatusCode, ErrorCode) {
        match self {
            Self::BadRequest(_) => (StatusCode::BAD_REQUEST, ErrorCode::BadRequest),
            Self::Validation { .. } => (StatusCode::BAD_REQUEST, ErrorCode::Validation),
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, ErrorCode::Unauthorized),
            Self::Forbidden => (StatusCode::FORBIDDEN, ErrorCode::Forbidden),
            Self::NotFound(_) => (StatusCode::NOT_FOUND, ErrorCode::NotFound),
            Self::MethodNotAllowed => (StatusCode::METHOD_NOT_ALLOWED, ErrorCode::MethodNotAllowed),
            Self::Conflict(_) | Self::FieldConflict { .. } => {
                (StatusCode::CONFLICT, ErrorCode::Conflict)
            }
            Self::PayloadTooLarge => (StatusCode::PAYLOAD_TOO_LARGE, ErrorCode::PayloadTooLarge),
            Self::UnsupportedMediaType(_) => (
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                ErrorCode::UnsupportedMediaType,
            ),
            Self::NeedsSetup => (StatusCode::SERVICE_UNAVAILABLE, ErrorCode::NeedsSetup),
            Self::Locked { .. } => (StatusCode::CONFLICT, ErrorCode::Locked),
            Self::Database(_)
            | Self::Fonts(_)
            | Self::Media(_)
            | Self::Random(_)
            | Self::Header(_)
            | Self::Key(_)
            | Self::Task(_) => (StatusCode::INTERNAL_SERVER_ERROR, ErrorCode::Internal),
        }
    }
}

impl From<FontError> for ApiError {
    fn from(err: FontError) -> Self {
        match err {
            FontError::NotAFont(_) => Self::validation("file", err),
            FontError::TooLarge => Self::PayloadTooLarge,
            FontError::Url(_) | FontError::Upstream(_) | FontError::NoFiles => {
                Self::validation("url", err)
            }
            FontError::Http(_) | FontError::Io { .. } | FontError::Task(_) => Self::Fonts(err),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code) = self.status_and_code();
        let message = match code {
            ErrorCode::Internal => {
                tracing::error!(error = %self, "API request failed");
                INTERNAL_MESSAGE.to_owned()
            }
            _ => self.to_string(),
        };
        let field = match self {
            Self::Validation { field, .. } => field,
            Self::Locked { field } | Self::FieldConflict { field, .. } => Some(field),
            _ => None,
        };
        let body = ErrorBody {
            error: ErrorDetail {
                code,
                message,
                field,
            },
        };
        (status, Json(body)).into_response()
    }
}
