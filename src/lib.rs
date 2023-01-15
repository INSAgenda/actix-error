use serde::Serialize;
pub use resterror_derive::AsApiError;

#[derive(Debug, Clone, Serialize)]
pub struct ApiError {
    pub kind: &'static str,
    pub code: u16,
    pub messages: Vec<(String, String)>,
}

impl ApiError {
    /// Returns a new `ApiError` with the given kind and message.
    pub fn to_json(&self) -> serde_json::error::Result<String>  {
        serde_json::to_string(&self)
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

#[cfg(feature = "actix")]
use actix_web::http::StatusCode;
#[cfg(feature = "actix")]
impl actix_web::ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        StatusCode::from_u16(self.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    }

    fn error_response(&self) -> actix_web::HttpResponse {
        actix_web::HttpResponse::build(self.status_code()).json(self.messages.clone())
    }
}

#[derive(AsApiError)]
#[cfg_attr(feature = "po", po_directory = "locales/")]
pub enum ErrorEn {
    #[error(status = "BadRequest", msg_id = "invalid_password")]
    InvalidPassword,
    #[error(code = 404, msg_id = "invalid_id")]
    InvalidId(u32),
}

#[test]
fn default() {
    let e = ErrorEn::InvalidPassword;
    println!("Error::as_api_error() = {:?}", e.as_api_error());
    let e = ErrorEn::InvalidId(42);
    println!("Error::as_api_error() = {:?}", e.as_api_error());
    
}