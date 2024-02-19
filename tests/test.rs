use actix_error::AsApiError;

#[derive(AsApiError)]
pub enum ErrorEn {
    #[error(status = "BadRequest", msg = "invalid_password")]
    InvalidPassword,
    #[error(code = 404, msg = "invalid id {}")]
    InvalidId(u32),
    #[error(code = 500, msg = "invalid name {name} and age {age}")]
    NamedError { name: String, age: u32 },
}

#[actix_web::test]
async fn test_error() {
    let error = ErrorEn::InvalidPassword;
    let api_error = error.as_api_error();
    assert_eq!(api_error.code, 400);
    assert_eq!(api_error.kind, "invalid_password");
    assert_eq!(api_error.message, "invalid_password");

    let error = ErrorEn::InvalidId(100);
    let api_error = error.as_api_error();
    assert_eq!(api_error.code, 404);
    assert_eq!(api_error.kind, "invalid_id");
    assert_eq!(api_error.message, "invalid id 100");

    let error = ErrorEn::NamedError {
        name: "test".to_string(),
        age: 100,
    };
    let api_error = error.as_api_error();
    assert_eq!(api_error.code, 500);
    assert_eq!(api_error.kind, "named_error");
    assert_eq!(api_error.message, "invalid name test and age 100");
}
