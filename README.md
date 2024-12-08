# actix-error
## Introduction

The `derive(AsApiError)` library provides a powerful, derive macro for Rust developers working with Actix-Web to easily convert enum variants into structured API errors. This library simplifies error handling in web applications by enabling automatic mapping of custom error types to HTTP response errors.

## Features
 - Automatic Conversion: Automatically convert enum variants to ApiError instances, including status codes, error kinds, and messages.
 - Customizable Error Responses: Customize error messages and HTTP status codes directly in enum definitions.
 - Support for Structured Errors: Handle errors with additional context, supporting both unnamed and named fields in enum variants.
 - Group Error Handling: Aggregate related errors into groups for streamlined error management.
 - Integration with Actix-Web: Seamlessly integrates with Actix-Web's error handling mechanisms.

## Installation
```toml
[dependencies]
actix-error = "0.2.5"
```

## Usage
### Defining Errors
Use the `#[derive(AsApiError)]` macro on enums to define your error types. Customize each variant with #[error] attributes to specify HTTP status codes, error messages, and more.


### Handling Errors in Actix-Web
Implement your Actix-Web handlers to return your custom errors. The `AsApiErrorTrait` ensures they are automatically converted into appropriate HTTP responses.

```rust
async fn my_handler() -> Result<HttpResponse, MyError> {
    // Your handler logic here...
    Err(MyError::NotFound)
}
```

### Advanced Error Handling
For errors requiring additional context, use named or unnamed fields directly in your enum variants.

```rust
#[derive(AsApiError)]
pub enum DetailedError {
    #[error(code = 500, msg = "Unexpected error occurred: {0}")]
    SystemError(String),
    #[error(status = "BadRequest", msg = "Invalid input: {field}")]
    ValidationError { field: DataField },
}

#[derive(AsApiError)]
pub enum Error {
    #[error(group)]
    Detailed(DetailedError), // Group errors together
    #[error(code = 500, msg = "Database error occurred", ignore)]
    DatabaseError(PostgresError), // Ignore the unnamed field
}

```
### Response format
```json
{
    "kind": "system_error",
    "message": "Unexpected error occurred: Internal Server Error"
    
}
```
