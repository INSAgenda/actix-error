# actix-error
## Introduction

The `actix-error` crate, along with `actix-error-derive`, provides a powerful derive macro for Rust developers working with Actix-Web to easily convert enum variants into structured API errors. This library simplifies error handling in web applications by enabling automatic mapping of custom error types to HTTP responses, now with enhanced support for detailed, structured error information using the `#[api_error(...)]` attribute.

## Features
 - Automatic Conversion: Automatically convert enum variants to `ApiError` instances, including status codes, error kinds, and messages, using the `#[api_error(...)]` attribute.
 - Customizable Error Responses: Customize error messages, HTTP status codes (via `code` or `status`), and error `kind` directly in enum definitions.
 - Support for Structured Errors: Handle errors with additional context. The `ApiError` struct includes a `details` field (`Option<serde_json::Value>`). The derive macro automatically populates this field if an enum variant (not marked as `group`) has a single field of type `serde_json::Value` or `Option<serde_json::Value>`.
 - Message Formatting: Use the `msg` attribute to define custom messages, with support for interpolating values from variant fields (e.g., `msg = "Error with {field_name}"`). Use `ignore` on a field if it should not be part of the default message generation when `msg` is not provided, or if you handle its display manually.
 - Conditional `Display` Implementation: A `std::fmt::Display` implementation is automatically generated for your enum if any variant uses the `msg` attribute. If no `msg` attribute is used, you should provide your own `Display` implementation (e.g., using `thiserror`).
 - Group Error Handling: Aggregate related errors into groups for streamlined error management using the `group` attribute.
 - Integration with Actix-Web: Seamlessly integrates with Actix-Web's error handling mechanisms.

## Installation
To use `actix-error` in your project, add the following to your `Cargo.toml`:
```toml
[dependencies]
actix-error = "0.x.y" # Replace with the desired version
# actix-error-derive is re-exported by actix-error, so you usually don't need to add it separately.
```
It's recommended to use the same version for both crates if you are using the derive macro.

## Usage
### Defining Errors
Use the `#[derive(AsApiError)]` macro on enums to define your error types. Customize each variant with `#[api_error(...)]` attributes to specify HTTP status codes, error messages, and more.

*   `code = <u16>`: Directly sets the HTTP status code (e.g., `code = 404`).
*   `status = "<StatusCodeString>"`: Sets the HTTP status code based on a predefined string (e.g., `status = "NotFound"` which maps to 404). If both `code` and `status` are provided, `code` takes precedence. If neither is provided, it defaults to 500.
*   `kind = "<string>"`: Sets a machine-readable error type. Defaults to the snake_case version of the variant name.
*   `msg = "<string>"`: Sets a human-readable message. Can interpolate variant fields using `{field_name}` for named fields or `{index}` for unnamed fields (e.g., `{0}`). If a variant has fields and `msg` is not provided, the macro attempts to generate a message from the fields unless `ignore` is used.
*   `ignore`: If a variant has fields and no `msg` is specified, adding `ignore` to a field (or to the variant if it's a unit-like variant whose message should be suppressed from auto-generation) prevents it from being automatically included in the message. For fields, this is useful if they are only meant for the `details` field.
*   `group`: Used on a variant that wraps another error type that itself implements `AsApiErrorTrait`. The `as_api_error()` method will be called on the wrapped error.

```rust
use actix_error::AsApiError;
use serde_json::{json, Value};

#[derive(Debug, AsApiError)]
pub enum MyError {
    #[api_error(status = "NotFound", msg = "The requested resource was not found.")]
    NotFound,

    #[api_error(code = 401, msg = "Authentication required.")]
    Unauthorized,

    #[api_error(code = 500, msg = "An internal server error occurred.")]
    InternalError,
    
    // This variant's single field of type Option<Value> will be used for `ApiError.details`
    #[api_error(status = "BadRequest", msg = "Invalid input provided.")]
    WithDetails(Option<Value>),

    #[api_error(status = "UnprocessableEntity", msg = "Validation failed for field {field_name}. Issue: {issue}")]
    ValidationError { field_name: String, issue: String },

    // This variant's single field of type Value will be used for `ApiError.details`
    // No msg is provided, so if Display is not implemented manually, it might be empty or a default.
    // However, the details field will be populated.
    #[api_error(status = "PaymentRequired")]
    PaymentData(Value),

    #[api_error(code = 400, msg = "User ID {0} is invalid.")]
    InvalidUserId(u32),

    // Example with ignore: field_to_ignore won't be in the auto-generated message if msg wasn't specified.
    // If msg is specified like below, ignore on the field itself is not needed for message generation.
    #[api_error(status = "BadRequest", msg = "Configuration error with setting: {setting_key}")]
    ConfigError { setting_key: String, #[api_error(ignore)] _internal_code: u32 },
}

// Example of creating an error with details
fn create_detailed_error() -> MyError {
    MyError::WithDetails(Some(json!({
        "field": "username",
        "issue": "Username cannot be empty"
    })))
}

fn create_payment_error() -> MyError {
    MyError::PaymentData(json!({
        "transaction_id": "12345",
        "reason": "Insufficient funds"
    }))
}
```

### Handling Errors in Actix-Web
Implement your Actix-Web handlers to return your custom errors. The `AsApiErrorTrait` (which is automatically implemented by the derive macro) ensures they are converted into appropriate HTTP responses.

```rust
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
# use actix_error::AsApiError;
# use serde_json::{json, Value};

# #[derive(Debug, AsApiError)]
# pub enum MyError {
#     #[api_error(status = "NotFound", msg = "The requested resource was not found.")]
#     NotFound,
#     #[api_error(status = "BadRequest", msg = "Invalid input provided.")]
#     WithDetails(Option<Value>),
# }

async fn my_handler() -> Result<HttpResponse, MyError> {
    // Your handler logic here...
    // For example, returning a NotFound error:
    Err(MyError::NotFound)
}

async fn another_handler(fail: bool) -> Result<HttpResponse, MyError> {
    if fail {
        Err(MyError::WithDetails(Some(json!({"debug_info": "Failed due to unmet condition"}))))
    } else {
        Ok(HttpResponse::Ok().json({"status": "success"}))
    }
}

// main function for example purposes
# #[actix_web::main]
# async fn main() -> std::io::Result<()> {
#     HttpServer::new(|| {
#         App::new()
#             .route("/test", web::get().to(my_handler))
#             .route("/test_details", web::get().to(|| another_handler(true)))
#     })
#     .bind("127.0.0.1:8080")?
#     .run()
#     .await
# }
```

### Advanced Error Handling & `details` Field
The derive macro automatically populates the `ApiError.details` field if an enum variant meets these conditions:
1. It is **not** marked with `#[api_error(group)]`.
2. It has exactly **one** field.
3. The type of that single field is `serde_json::Value` or `Option<serde_json::Value>`.

If these conditions are met, the value of this field is moved into the `details` field of the generated `ApiError`.

```rust
# use actix_error::AsApiError;
# use serde_json::Value; // Required for serde_json::Value

// Assuming PostgresError is some error type, and DataField is some struct/enum
// For simplicity, let's define them minimally for the example to compile
#[derive(Debug)]
pub struct PostgresError;
impl std::fmt::Display for PostgresError { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "PostgresError") } }
impl std::error::Error for PostgresError {}


#[derive(Debug)]
pub struct DataField(String);
impl std::fmt::Display for DataField { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.0) } }


#[derive(Debug, AsApiError)]
pub enum DetailedError {
    #[api_error(code = 500, msg = "Unexpected error occurred: {0}")]
    SystemError(String), // Simple string interpolation

    // Here, `error_detail` is not automatically put into `ApiError.details` because there are two fields.
    // You would typically format this into the message or handle it manually if you need it in `details`.
    // For automatic `details` population, the variant must have a *single* field of type Value or Option<Value>.
    #[api_error(status = "BadRequest", msg = "Invalid input for field {field_name}: {error_detail}")]
    ValidationError { field_name: String, error_detail: String }, // Changed error_detail to String for this example

    // This variant's single field `data` will be used for `ApiError.details`.
    #[api_error(status = "UnprocessableEntity", msg = "Complex issue encountered.")]
    ComplexIssueWithDetails(Value),
}

#[derive(Debug, AsApiError)]
pub enum ErrorGroupExample {
    #[api_error(group)]
    Detailed(DetailedError), // Group errors together

    // The `PostgresError` field is not `Value` or `Option<Value>`, so it won't go to `details` automatically.
    // If `msg` was not provided, and `Display` for `PostgresError` exists, it might be used.
    // `ignore` here means `PostgresError` won't be part of an auto-generated message if `msg` is absent.
    #[api_error(code = 500, msg = "Database error occurred", ignore)]
    DatabaseError(#[api_error(ignore)] PostgresError),
    
    // This variant's single field of type Value will be used for `ApiError.details`.
    // The message "Raw JSON error" will be used for `ApiError.message`.
    #[api_error(status = "UnprocessableEntity", msg = "Raw JSON error")]
    RawJsonError(Value),
}

// Example of how the derive macro populates details:
// If ErrorGroupExample::RawJsonError(json!({"complex": "data"})) is returned,
// the ApiError generated will have `message: "Raw JSON error"` and
// `details: Some(json!({"complex": "data"}))`.
//
// If DetailedError::ComplexIssueWithDetails(json!({"issue_code": 123})) is returned,
// the ApiError generated will have `message: "Complex issue encountered."` and
// `details: Some(json!({"issue_code": 123}))`.
```

### Conditional `Display` Trait Implementation
The `actix-error-derive` macro will generate a `std::fmt::Display` implementation for your enum if **any** of its variants use the `#[api_error(msg = "...")]` attribute.
*   If `msg` is used, the `Display` implementation will use the provided message string, performing field interpolation if specified (e.g., `msg = "Error: {field}"`).
*   If a variant does **not** have a `msg` attribute:
    *   And it has fields that are not `ignore`d, the `Display` implementation will attempt to create a string from these fields.
    *   And it's a unit variant (no fields), its `Display` output will be the variant name.
*   If **no** variants in the enum use the `msg` attribute, the derive macro **will not** generate a `Display` implementation. In this scenario, you are responsible for providing one, for example, by also deriving `thiserror::Error` which provides a `Display` impl based on its own attributes. This is to avoid conflicts and give you more control when `msg` is not the primary way you define error messages.

### Response Format
The `ApiError` struct serializes to JSON. The `code` field (HTTP status code) is used by Actix-Web to set the response status and is not part of the JSON body by default (due to `#[serde(skip_serializing)]` on `ApiError.code`).

**Basic Error:**
```json
{
    "kind": "not_found",
    "message": "The requested resource was not found."
}
```

**Error with Details:**
If the `details` field in `ApiError` is `Some(value)`, it will be included in the JSON response (due to `#[serde(skip_serializing_if = "Option::is_none")]` on `ApiError.details`).
```json
{
    "kind": "with_details",
    "message": "Invalid input provided.",
    "details": {"field": "username", "issue": "Username cannot be empty"}
}
```
