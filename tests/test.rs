use actix_error::*;

#[derive(AsApiError, Debug, thiserror::Error)]
pub enum GrpError {
    /// Firewall fail
    #[api_error(status = "InternalServerError")] 
    #[error("Firewall fail")] // thiserror attribute
    FirewallFail,
    /// Invalid token
    #[api_error(status = "BadRequest")]
    #[error("Invalid token")] // thiserror attribute
    InvalidToken,
}

#[derive(AsApiError, Debug)] // Removed thiserror::Error for this enum so as_api_error with msg will implement Display
pub enum ErrorEn {
    /// Invalid password
    #[api_error(status = "BadRequest", msg = "Invalid password")]
    InvalidPassword,
    /// invalid id {0}
    #[api_error(code = 404, msg = "invalid id {}")]
    InvalidId(u32),
    /// invalid name {name} and age {age}
    #[api_error(code = 500, msg = "invalid name {name} and age {age}")]
    NamedError { name: String, age: u32 },
    /// Internal database error: {0}
    #[api_error(status = "InternalServerError", msg = "Internal database error {0}", ignore)]
    PostgresError(String),
    /// {0}
    #[api_error(group)]
    GroupError(GrpError), // GrpError still uses thiserror, this is fine for testing group functionality
    /// Error with details: {:?}
    #[api_error(status = "BadRequest", msg = "Error with details", ignore)]
    ErrorWithDetails(Option<serde_json::Value>),
    /// Error with direct details: {}
    #[api_error(status = "UnprocessableEntity", msg = "Error with direct details", ignore)]
    ErrorWithDirectDetails(serde_json::Value),
    /// Variant without a specific msg in api_error, and no thiserror Display
    #[api_error(code = 402)] // Use numeric code for PaymentRequired
    MissingMessageVariant,
}

// Enum that uses thiserror for Display, and AsApiError without its own msg attribute
#[derive(AsApiError, Debug, thiserror::Error)]
pub enum ErrorWithThiserrorDisplay {
    /// Item not found via thiserror: id {0}
    #[api_error(status = "NotFound")] // Changed from error to api_error
    #[error("Item not found via thiserror: id {0}")] // thiserror attribute
    ItemNotFound(String),

    /// Authentication failed (user: {username}) - from thiserror
    #[api_error(status = "Unauthorized")] // Changed from error to api_error
    #[error("Authentication failed (user: {username}) - from thiserror")] // thiserror attribute
    AuthFailure { username: String },

    /// Just a simple error from thiserror
    #[api_error(status = "InternalServerError")] // Changed from error to api_error
    #[error("Just a simple error from thiserror")] // thiserror attribute
    SimpleError,
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

    let error = ErrorEn::PostgresError("test_pg_error".to_string());
    let api_error = error.as_api_error();
    assert_eq!(api_error.code, 500);
    assert_eq!(api_error.kind, "postgres_error");
    assert_eq!(api_error.message, "Internal database error test_pg_error");

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

    // Test for variant without specific msg and no thiserror Display
    let error_missing_msg = ErrorEn::MissingMessageVariant;
    let api_error_missing_msg = error_missing_msg.as_api_error();
    assert_eq!(api_error_missing_msg.code, 402); // PaymentRequired
    assert_eq!(api_error_missing_msg.kind, "missing_message_variant");
    assert_eq!(api_error_missing_msg.message, "MissingMessageVariant"); // Should default to variant name
}

#[actix_web::test]
async fn test_thiserror_display_integration() {
    // Test case 1: Variant with a field
    let error1 = ErrorWithThiserrorDisplay::ItemNotFound("test_id_123".to_string());
    let api_error1 = error1.as_api_error();
    assert_eq!(api_error1.code, 404); // From NotFound status
    assert_eq!(api_error1.kind, "item_not_found"); // Snake case of variant
    assert_eq!(api_error1.message, "Item not found via thiserror: id test_id_123"); // From thiserror's Display

    // Test case 2: Variant with named fields
    let error2 = ErrorWithThiserrorDisplay::AuthFailure { username: "copilot".to_string() };
    let api_error2 = error2.as_api_error();
    assert_eq!(api_error2.code, 401); // From Unauthorized status
    assert_eq!(api_error2.kind, "auth_failure");
    assert_eq!(api_error2.message, "Authentication failed (user: copilot) - from thiserror"); // From thiserror's Display

    // Test case 3: Simple unit variant
    let error3 = ErrorWithThiserrorDisplay::SimpleError;
    let api_error3 = error3.as_api_error();
    assert_eq!(api_error3.code, 500); // From InternalServerError status
    assert_eq!(api_error3.kind, "simple_error");
    assert_eq!(api_error3.message, "Just a simple error from thiserror"); // From thiserror's Display
}
