use std::collections::HashMap;

pub use resterror_derive::AsApiError;
use serde::Serialize;

pub mod translate;

#[derive(Debug, Clone, Serialize)]
pub struct ApiError {
    pub kind: String,
    #[serde(skip_serializing)]
    pub code: u16,
    #[serde(skip_serializing)]
    pub messages: HashMap<String, String>,
    message_fr: String,
    message_en: String,
    origin: String,
}

impl ApiError {
    pub fn new(code: u16, kind: &str, messages: HashMap<String, String>) -> Self {
        let message_en = messages.get("en").unwrap_or(&String::new()).to_string();
        let message_fr = messages.get("fr").unwrap_or(&String::new()).to_string();
        Self {
            kind: kind.to_string(),
            code,
            messages,
            message_fr,
            message_en,
            origin: String::new(),
        }
    }
}

pub trait AsApiError {
    fn as_api_error(&self) -> ApiError;
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} : {:?}", self.kind, self.messages)
    }
}
#[cfg(feature = "po")]
use translate::Translation;

#[cfg(feature = "actix")]
use actix_web::http::StatusCode;

#[cfg(feature = "actix")]
impl actix_web::ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        StatusCode::from_u16(self.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    }

    fn error_response(&self) -> actix_web::HttpResponse {
        actix_web::HttpResponse::build(self.status_code()).json(self)
    }
}
#[cfg(feature = "po")]
#[derive(AsApiError)]
#[cfg_attr(all(feature = "json", feature = "po"), msg_path = "locales/")]
#[cfg_attr(all(not(feature = "json"), feature = "po"), po_directory = "locales/")]
pub enum ErrorEn {
    #[error(status = "BadRequest", msg_id = "invalid_password")]
    InvalidPassword,
    #[error(code = 404, msg_id = "invalid_id")]
    InvalidId(u32),
    #[error(code = 500, msg_id = "named_error")]
    NamedError { name: String, age: u32 },
    #[error(code = 500)]
    NamedError2(Translation),
}

#[test]
#[cfg(all(feature = "po", feature = "actix"))]
fn default() {
    let e = ErrorEn::InvalidPassword;
    println!(
        "Error::as_api_error() = {:?}",
        serde_json::to_string(&e.as_api_error()).unwrap()
    );
    let e = ErrorEn::InvalidId(42);
    println!(
        "Error::as_api_error() = {:?}",
        serde_json::to_string(&e.as_api_error()).unwrap()
    );
    let e = ErrorEn::NamedError {
        name: "John".to_string(),
        age: 42,
    };
    println!(
        "Error::as_api_error() = {:?}",
        serde_json::to_string(&e.as_api_error()).unwrap()
    );
    let e = ErrorEn::NamedError2(tr! {
        "en" => "Hello",
        "fr" => "Bonjour",
    });
    println!(
        "Error::as_api_error() = {:?}",
        serde_json::to_string(&e.as_api_error()).unwrap()
    );
}
