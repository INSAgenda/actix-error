use actix_web::http::StatusCode;
use serde::Serialize;
pub use actix_error_derive::AsApiError;

#[derive(Debug, Clone, Serialize)]
pub struct ApiError {
    pub kind: String,
    #[serde(skip_serializing)]
    pub code: u16,
    pub message: String,
}

impl ApiError {
    pub fn new(code: u16, kind: &str, message: String) -> Self {
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

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} : {}", self.kind, self.message)
    }
}

impl actix_web::ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        StatusCode::from_u16(self.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    }

    fn error_response(&self) -> actix_web::HttpResponse {
        actix_web::HttpResponse::build(self.status_code()).json(self)
    }
}
