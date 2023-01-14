use actix_api_error_derive::AsApiError;
use std::collections::HashMap;

#[derive(Debug)]
pub struct ApiError {
    pub code: u16,
    pub messages: HashMap<String, String>,
}

pub trait AsApiError {
    fn as_api_error(&self) -> ApiError;
}

#[derive(AsApiError)]
//#[po_directory = ".po"]
pub enum ErrorEn {
    #[error(code = 404, msg_id = "invalid_password")]
    InvalidPassword,
    #[error(code = 404, msg_id = "invalid_id")]
    InvalidId(u32),
}


#[test]
fn default() {
    let e = ErrorEn::InvalidPassword;
    println!("Error::to_code() = {:?}", e.as_api_error());
    let e = ErrorEn::InvalidId(42);
    println!("Error::to_code() = {:?}", e.as_api_error());
}