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
    #[error(status = "BadRequest", msg = "Error with details", ignore)]
    ErrorWithDetails(Option<serde_json::Value>),
    #[error(status = "UnprocessableEntity", msg = "Error with direct details", ignore)]
    ErrorWithDirectDetails(serde_json::Value),
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

    // Test error with Option<serde_json::Value> details
    let details_json = serde_json::json!({ "field": "test_field", "issue": "is missing" });
    let error_with_details = ErrorEn::ErrorWithDetails(Some(details_json.clone()));
    let api_error_details = error_with_details.as_api_error();
    assert_eq!(api_error_details.code, 400);
    assert_eq!(api_error_details.kind, "error_with_details");
    assert_eq!(api_error_details.message, "Error with details");
    assert_eq!(api_error_details.details, Some(details_json));

    // Test error with direct serde_json::Value details
    let direct_details_json = serde_json::json!({ "error_code": 123, "description": "something went wrong" });
    let error_with_direct_details = ErrorEn::ErrorWithDirectDetails(direct_details_json.clone());
    let api_error_direct_details = error_with_direct_details.as_api_error();
    assert_eq!(api_error_direct_details.code, 422); // UnprocessableEntity
    assert_eq!(api_error_direct_details.kind, "error_with_direct_details");
    assert_eq!(api_error_direct_details.message, "Error with direct details");
    assert_eq!(api_error_direct_details.details, Some(direct_details_json));

    // Test error with None details
    let error_no_details = ErrorEn::ErrorWithDetails(None);
    let api_error_no_details = error_no_details.as_api_error();
    assert_eq!(api_error_no_details.code, 400);
    assert_eq!(api_error_no_details.kind, "error_with_details");
    assert_eq!(api_error_no_details.message, "Error with details");
    assert_eq!(api_error_no_details.details, None);
}
