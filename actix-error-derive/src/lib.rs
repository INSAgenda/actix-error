use darling::FromVariant;
use syn::{parse_macro_input, DeriveInput};

use proc_macro::TokenStream;
/// 
#[derive(FromVariant, Default)]
#[darling(default, attributes(error))]
struct Opts {
    code: Option<u16>,
    status: Option<String>,
    kind: Option<String>,
    msg: Option<String>,
    ignore: bool,
}


/// This derive macro is used to convert an enum into an ApiError.  
/// You can use it by adding the ```#[derive(AsApiError)]``` attribute to your enum.  
/// By default, the kind is ```snake case```.  
/// ```#[error(kind = "your_message_id")]``` attribute to the variant.  
/// You can also add a custom code to the error by adding the ```#[error(code = 400)]``` attribute to the variant.  
/// The following status are available and return the corresponding status code: 
/// ``` rust
/// match error_kind {
///     "BadRequest" => 400,
///     "Unauthorized" => 401,
///     "Forbidden" => 403,
///     "NotFound" => 404,
///     "MethodNotAllowed" => 405,
///     "Conflict" => 409,
///     "Gone" => 410,
///     "PayloadTooLarge" => 413,
///     "UnsupportedMediaType" => 415,
///     "UnprocessableEntity" => 422,
///     "TooManyRequests" => 429,
///     "InternalServerError" => 500,
///     "NotImplemented" => 501,
///     "BadGateway" => 502,
///     "ServiceUnavailable" => 503,
///     "GatewayTimeout" => 504,
///     _ => unreachable!(),
/// }
/// ```
#[proc_macro_derive(AsApiError, attributes(error))]
pub fn derive(input: TokenStream) -> TokenStream {
    use convert_case::{Case, Casing};

    // Parse the input tokens into a syntax tree
    let ast = parse_macro_input!(input as DeriveInput); 
    let ident_name = ast.ident;

    // Get the variants
    let enum_data = match ast.data {
        syn::Data::Enum(data) => data,
        _ => panic!("ApiError can only be derived for enums"),
    };
    let variants = enum_data.variants;

    // Generate the variant's code 
    let variants = variants.iter().map(|v| {
        let ident = &v.ident;
        let matching_wrapped = if let syn::Fields::Unnamed(u) = &v.fields {
            let mut fields = String::new();
            for (i, _) in u.unnamed.iter().enumerate() {
                fields.push_str(&format!("a{}", i));
                if i < u.unnamed.len() - 1 {
                    fields.push_str(", ");
                }
            }
            format!("({})", fields)
        } else if let syn::Fields::Named(u) = &v.fields {
            let mut fields = String::new();
            for (i, field) in u.named.iter().enumerate() {
                fields.push_str(field.ident.as_ref().unwrap().to_string().as_str());
                if i < u.named.len() - 1 {
                    fields.push_str(", ");
                }
            }
            format!("{{ {} }}", fields)
        } else {
            String::new()
        };
        // Get the tuple if it exists
        let tuple = match &v.fields {
            syn::Fields::Unnamed(u) => Some(u),
            _ => None,
        };
        let struc = if let syn::Fields::Named(n) = &v.fields {
            Some(n)
        } else {
            None
        };
            
        let opts = Opts::from_variant(&v).expect("Couldn't get the options for the variant");
        let code = if let Some(code) = opts.code {
            code
        } else {
            if let Some(ref error_kind) = opts.status {
                match error_kind.as_str() {
                    "BadRequest" => 400,
                    "Unauthorized" => 401,
                    "Forbidden" => 403,
                    "NotFound" => 404,
                    "MethodNotAllowed" => 405,
                    "Conflict" => 409,
                    "Gone" => 410,
                    "PayloadTooLarge" => 413,
                    "UnsupportedMediaType" => 415,
                    "UnprocessableEntity" => 422,
                    "TooManyRequests" => 429,
                    "InternalServerError" => 500,
                    "NotImplemented" => 501,
                    "BadGateway" => 502,
                    "ServiceUnavailable" => 503,
                    "GatewayTimeout" => 504,
                    _ => panic!("Invalid kind for variant {}: {}", ident, error_kind),
                }
            } else {
                500
            }
        };

        
        use actix_web::http::StatusCode;
        if let Err(e) = StatusCode::from_u16(code) {
            panic!("Invalid status code for variant {}: {}", ident, e);
        }
        let kind = opts.kind.unwrap_or_else(|| ident.to_string().to_case(Case::Snake));

        // Get the messages for the variant
        let mut message = "String::new()".to_owned();
        if let Some(msg) = opts.msg {
            message = if let Some(tuple) = tuple  {
                // genrate a string like "format!(\"message\", self.0, self.1)"
                // Where message is the msg attribute of the variant
                // and self.0, self.1 are the tuple fields
                let mut fields = String::new();
                for (i, _) in tuple.unnamed.iter().enumerate() {
                    fields.push_str(&format!("a{}", i));
                    if i < tuple.unnamed.len() - 1 {
                        fields.push_str(", ");
                    }
                }
                format!("format!(\"{}\", {})", msg, fields)
            } else if let Some(_) = struc {
                format!("format!(\"{}\")", msg)
            } else {
                format!("\"{}\".to_owned()", msg)
            };

            if opts.ignore {
                message = format!("\"{}\".to_owned()", msg);
            }
        }

        let mut list_vars = String::new();
        
        // Add the tuple syntax if it exists
        if list_vars.len() > 0 {
            if struc.is_some() {
                list_vars = format!("{{ {} }}", list_vars);
            } else {
                list_vars = format!("( {} )", list_vars);
            }
        }

        
        format!("
            {ident_name}::{ident} {matching_wrapped} {list_vars} => {{
                ApiError::new(
                    {code}, 
                    \"{kind}\",
                    {message}
                )
            }},
        ", )
    });

    // Implement the ApiError trait
    let mut code = String::new();
    code.push_str("use actix_error::{AsApiErrorTrait, ApiError};\n");
    code.push_str(&format!("impl AsApiErrorTrait for {ident_name} {{\n"));
    code.push_str(" fn as_api_error(&self) -> ApiError {\n");
    code.push_str("     match &self {\n");
    for v in variants {
        code.push_str(&v.to_string());
    }
    code.push_str("\n    }\n");
    code.push_str("   }\n");
    code.push_str("}\n");

    code.push_str(&format!(r#"
        use actix_web::http::StatusCode;
        use std::fmt::{{Display, Formatter, Debug}};

        impl Debug for {ident_name} {{
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {{ write!(f, "{{:?}}", self.as_api_error()) }}
        }}
    
        impl Display for {ident_name} {{
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {{ write!(f, "{{:?}}", self.as_api_error()) }}
        }}
    
        impl actix_web::ResponseError for {ident_name} {{
            fn status_code(&self) -> StatusCode {{
                let api_error = self.as_api_error();
                StatusCode::from_u16(api_error.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
            }}
        
            fn error_response(&self) -> actix_web::HttpResponse {{
                let api_error = self.as_api_error();
                actix_web::HttpResponse::build(self.status_code()).json(api_error)
            }}
        }}
    "#));
    code.parse().expect("Couldn't parse the code")
}
