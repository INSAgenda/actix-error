use actix_web::http::StatusCode;
use std::fmt::{Display, Formatter, Debug};
use std::error::Error;
use serde::Serialize;
pub use actix_error_derive::AsApiError;

#[derive(Debug, Clone, Serialize)]
pub struct ApiError {
    pub kind: String,
    #[serde(skip_serializing)]
    pub code: u16, // Changed from StatusCode to u16
    pub message: String,
}

impl ApiError {
    pub fn new(code: u16, kind: &str, message: String) -> Self { // Changed code to u16
        Self {
            kind: kind.to_string(),
            message,
            code,
        }
    }
}

pub trait AsApiErrorTrait {
    fn as_api_error(&self) -> ApiError;
}

impl Display for ApiError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}: {}", self.kind, self.message)
    }
}

impl Error for ApiError {}

impl actix_web::ResponseError for ApiError {
    fn status_code(&self) -> actix_web::http::StatusCode { // Use fully qualified path
        actix_web::http::StatusCode::from_u16(self.code)
            .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR)
    }

    fn error_response(&self) -> actix_web::HttpResponse {
        actix_web::HttpResponse::build(actix_web::http::StatusCode::from_u16(self.code)
            .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR)).json(self)
    }
}

