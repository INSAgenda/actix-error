use actix_api_error_derive::ApiError;

pub trait ApiError {
    fn to_code(&self) -> u16;
}


#[derive(ApiError)]
#[po_path = "*.po"]
pub enum ErrorEn {
    #[error(code = "404")]
    InvalidPassword,
}

#[test]
fn default() {
    let e = ErrorEn::InvalidPassword;
    println!("Error::to_code() = {}", e.to_code());
}