use std::collections::HashMap;

use serde::Serialize;
pub use resterror_derive::AsApiError;

pub mod translate;

#[derive(Debug, Clone, Serialize)]
pub struct ApiError {
    pub kind: &'static str,
    #[serde(skip_serializing, skip_deserializing)]
    pub code: u16,
    pub messages: HashMap<String, String>,
}

pub trait AsApiError {
    fn as_api_error(&self) -> ApiError;
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} : {:?}", self.kind, self.messages)
    }
}

#[cfg(feature = "actix")]
use actix_web::http::StatusCode;
use translate::Translation;

#[cfg(feature = "actix")]
impl actix_web::ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        StatusCode::from_u16(self.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    }

    fn error_response(&self) -> actix_web::HttpResponse {
        actix_web::HttpResponse::build(self.status_code()).json(self)
    }
}

#[derive(AsApiError)]
#[cfg_attr(feature = "po", po_directory = "locales/")]
pub enum ErrorEn {
    #[error(status = "BadRequest", msg_id = "invalid_password")]
    InvalidPassword,
    #[error(code = 404, msg_id = "invalid_id")]
    InvalidId(u32),
    #[error(code = 500, msg_id = "named_error")]
    NamedError { name: String, age: u32 },
    #[error(code = 500)]
    NamedError2(Translation)
}

#[test]
fn default() {
    let e = ErrorEn::InvalidPassword;
    println!("Error::as_api_error() = {:?}", e.as_api_error());
    let e = ErrorEn::InvalidId(42);
    println!("Error::as_api_error() = {:?}", e.as_api_error());
    let e = ErrorEn::NamedError { name: "John".to_string(), age: 42 };
    println!("Error::as_api_error() = {:?}", e.as_api_error());
    let e = ErrorEn::NamedError2(trad!{
        "en" => "Hello",
        "fr" => "Bonjour",
    });
    println!("Error::as_api_error() = {:?}", e.as_api_error());
}

