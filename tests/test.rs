use actix_error::*;

#[derive(AsApiError)]
pub enum GrpError {
    #[error(status = "InternalServerError", msg = "Firewall fail")]
    FirewallFail,
    #[error(status = "BadRequest", msg = "Invalid token")]
    InvalidToken,
}

#[derive(AsApiError)]
pub enum ErrorEn {
    #[error(status = "BadRequest", msg = "Invalid password")]
    InvalidPassword,
    #[error(code = 404, msg = "invalid id {}")]
    InvalidId(u32),
    #[error(code = 500, msg = "invalid name {name} and age {age}")]
    NamedError { name: String, age: u32 },
    #[error(status = "InternalServerError", msg = "Internal database error", ignore)]
    PostgresError(String),
    #[error(group)]
    GroupError(GrpError),
}

#[actix_web::test]
async fn test_error() {
    let error = ErrorEn::InvalidPassword;
    let api_error = error.as_api_error();
    assert_eq!(api_error.code, 400);
    assert_eq!(api_error.kind, "invalid_password");
    assert_eq!(api_error.message, "Invalid password");

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

    let error = ErrorEn::PostgresError("test".to_string());
    let api_error = error.as_api_error();
    assert_eq!(api_error.code, 500);
    assert_eq!(api_error.kind, "postgres_error");
    assert_eq!(api_error.message, "Internal database error");

    // Group error
    let error = ErrorEn::GroupError(GrpError::FirewallFail);
    let api_error = error.as_api_error();
    assert_eq!(api_error.code, 500);
    assert_eq!(api_error.kind, "firewall_fail");
}
