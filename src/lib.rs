use std::fmt::{Display, Formatter, Debug};
use std::error::Error;
use serde::Serialize;
pub use actix_error_derive::AsApiError;

/// Represents a structured error that can be easily serialized and sent as an HTTP response.
#[derive(Debug, Clone, Serialize)]
pub struct ApiError {
    /// A machine-readable error type or category.
    pub kind: String,
    /// The HTTP status code associated with this error. This field is not serialized.
    #[serde(skip_serializing)]
    pub code: u16, // Changed from StatusCode to u16
    /// A human-readable message describing the error.
    pub message: String,
}

impl ApiError {
    /// Creates a new `ApiError`.
    ///
    /// # Arguments
    ///
    /// * `code` - The HTTP status code for this error.
    /// * `kind` - A string slice representing the kind or category of the error.
    /// * `message` - A `String` containing the descriptive message for the error.
    pub fn new(code: u16, kind: &str, message: String) -> Self { // Changed code to u16
        Self {
            kind: kind.to_string(),
            message,
            code,
        }
    }
}

/// A trait for types that can be converted into an `ApiError`.
pub trait AsApiErrorTrait {
    /// Converts the type into an `ApiError`.
    fn as_api_error(&self) -> ApiError;
}

impl Display for ApiError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}: {}", self.kind, self.message)
    }
}

impl Error for ApiError {}

impl actix_web::ResponseError for ApiError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        actix_web::http::StatusCode::from_u16(self.code)
            .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR)
    }

    fn error_response(&self) -> actix_web::HttpResponse {
        actix_web::HttpResponse::build(self.status_code()).json(self)
    }
}
