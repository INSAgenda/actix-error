# actix-error
## Introduction

The `actix-error` crate, along with `actix-error-derive`, provides a powerful derive macro for Rust developers working with Actix-Web to easily convert enum variants into structured API errors. This library simplifies error handling in web applications by enabling automatic mapping of custom error types to HTTP responses, now with support for detailed, structured error information.

## Features
 - Automatic Conversion: Automatically convert enum variants to `ApiError` instances, including status codes, error kinds, and messages.
 - Customizable Error Responses: Customize error messages and HTTP status codes directly in enum definitions.
 - Support for Structured Errors: Handle errors with additional context, supporting both unnamed and named fields in enum variants. The `ApiError` struct can include a `details` field of type `Option<serde_json::Value>` for complex error information.
 - Group Error Handling: Aggregate related errors into groups for streamlined error management.
 - Integration with Actix-Web: Seamlessly integrates with Actix-Web's error handling mechanisms.

## Installation
To use `actix-error` in your project, add the following to your `Cargo.toml`:
```toml
[dependencies]
actix-error = "0.2.9"
```
It's recommended to use the same version for both crates if you are using the derive macro.

## Usage
### Defining Errors
Use the `#[derive(AsApiError)]` macro on enums to define your error types. Customize each variant with #[error] attributes to specify HTTP status codes, error messages, and more.

```rust
use actix_error::AsApiError;
use serde_json::json;

#[derive(Debug, AsApiError)]
pub enum MyError {
    #[error(status = "NotFound", msg = "The requested resource was not found.")]
    NotFound,

    #[error(code = 401, msg = "Authentication required.")]
    Unauthorized,

    #[error(code = 500, msg = "An internal server error occurred.")]
    InternalError,
    
    #[error(status = "BadRequest", msg = "Invalid input provided.")]
    WithDetails(Option<serde_json::Value>), // This variant can carry details
}

// Example of creating an error with details
fn create_detailed_error() -> MyError {
    MyError::WithDetails(Some(json!({
        "field": "username",
        "issue": "Username cannot be empty"
    })))
}
```

### Handling Errors in Actix-Web
Implement your Actix-Web handlers to return your custom errors. The `AsApiErrorTrait` (which is automatically implemented by the derive macro) ensures they are converted into appropriate HTTP responses.

```rust
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
# use actix_error::AsApiError;
# use serde_json::json;

# #[derive(Debug, AsApiError)]
# pub enum MyError {
#     #[error(status = "NotFound", msg = "The requested resource was not found.")]
#     NotFound,
#     #[error(status = "BadRequest", msg = "Invalid input provided.")]
#     WithDetails(Option<serde_json::Value>),
# }

async fn my_handler() -> Result<HttpResponse, MyError> {
    // Your handler logic here...
    // For example, returning a NotFound error:
    Err(MyError::NotFound)
}

async fn another_handler(fail: bool) -> Result<HttpResponse, MyError> {
    if fail {
        Err(MyError::WithDetails(Some(json!({
            "reason": "Validation failed",
            "missing_fields": ["email", "password"]
        }))))
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

### Advanced Error Handling
For errors requiring additional context, use named or unnamed fields directly in your enum variants. The derive macro will attempt to format these into the message if specified (e.g., `msg = "Error with {field}"`). For more complex structured data, consider passing `serde_json::Value` directly as a field in your enum variant and ensuring your `ApiError::new` call (if manually constructing) or derive macro usage populates the `details` field.

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
    #[error(code = 500, msg = "Unexpected error occurred: {0}")]
    SystemError(String), // Simple string interpolation

    #[error(status = "BadRequest", msg = "Invalid input for field.")] // Details can be passed separately
    ValidationError { field_name: String, error_detail: Value },
}

#[derive(Debug, AsApiError)]
pub enum Error {
    #[error(group)]
    Detailed(DetailedError), // Group errors together

    #[error(code = 500, msg = "Database error occurred", ignore)]
    DatabaseError(PostgresError), // Ignore the unnamed field in the main message, but it could be part of details if handled by a custom AsApiError impl.
    
    #[error(status = "UnprocessableEntity")] // No msg here, details will be the primary source
    ComplexIssue(Value), // Pass serde_json::Value directly for the details field
}

// Example of how the derive macro might populate details if a variant contains a single serde_json::Value
// (This part is conceptual for the derive macro's behavior with a single Value field)
// If Error::ComplexIssue(json!({"complex": "data"})) is returned,
// the ApiError generated might have that json! value in its `details` field.
```
The derive macro passes `None` for the `details` argument to `ApiError::new` by default. If an enum variant has a single field of type `Option<serde_json::Value>` or `serde_json::Value`, you might need to manually implement `AsApiErrorTrait` for that specific enum if you want that field to be automatically used as the `details` in the resulting `ApiError`. Alternatively, the message formatting can include fields from the variant.

### Response Format
The `ApiError` struct serializes to JSON. The `code` field is used for the HTTP status code and is not part of the JSON body by default.

**Basic Error:**
```json
{
    "kind": "not_found",
    "message": "The requested resource was not found."
}
```

**Error with Details:**
If the `details` field in `ApiError` is `Some(value)`, it will be included in the JSON response:
```json
{
    "kind": "with_details",
    "message": "Invalid input provided.",
    "details": {
        "reason": "Validation failed",
        "missing_fields": ["email", "password"]
    }
}
```
