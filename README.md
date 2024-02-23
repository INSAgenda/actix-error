# actix-error
This is a simple library to handle errors in a RESTful way. It uses a lightweight syntax to define errors and their codes.
  
## Usage
Example of usage in a endpoint:
```rust
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

#[get("/{name}/{age}")]
async fn get_info(req: HttpRequest) -> Result<HttpResponse> {
    let name = req.match_info().get("name").unwrap();
    let age: u32 = req.match_info().get("age").unwrap().parse().unwrap();
    
    if name.len() < 3 && age < 18 {
        return Err(Error::InfoError { name: name.to_string(), age });
    }
}

```
This will return a json with the following structure for the following request:
```http
GET /jo/17
```

```json
{
    "kind": "info_error",
    "message": "invalid name jo and age 17",
    
}
```
