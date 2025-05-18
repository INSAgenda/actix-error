use darling::FromVariant;
use syn::{parse_macro_input, DeriveInput};
use proc_macro::TokenStream;
use quote::{quote, format_ident};
use convert_case::{Case, Casing};

#[derive(FromVariant, Default)] 
#[darling(default, attributes(error))]
struct Opts {
    code: Option<u16>,
    status: Option<String>,
    kind: Option<String>,
    msg: Option<String>,
    ignore: bool,
    group: bool,
}


/// This derive macro is used to convert an enum into an ApiError.  
/// You can use it by adding the ```#[derive(AsApiError)]``` attribute to your enum.  
/// By default, the kind is ```snake case```.  
/// ```#[error(kind = "your_message_id")]``` attribute to the variant.  
/// You can also add a custom code to the error by adding the ```#[error(code = 400)]``` attribute to the variant.  
/// The following status are available and return the corresponding status code: 
/// ``` rust
/// fn get_status_code(error_kind: &str) -> u16 {
///     match error_kind {
///         "BadRequest" => 400,
///         "Unauthorized" => 401,
///         "Forbidden" => 403,
///         "NotFound" => 404,
///         "MethodNotAllowed" => 405,
///         "Conflict" => 409,
///         "Gone" => 410,
///         "PayloadTooLarge" => 413,
///         "UnsupportedMediaType" => 415,
///         "UnprocessableEntity" => 422,
///         "TooManyRequests" => 429,
///         "InternalServerError" => 500,
///         "NotImplemented" => 501,
///         "BadGateway" => 502,
///         "ServiceUnavailable" => 503,
///         "GatewayTimeout" => 504,
///         _ => 0, // Or some other default/error handling
///     }
/// }
///
/// // Example usage:
/// let code = get_status_code("NotFound");
/// assert_eq!(code, 404);
/// let default_code = get_status_code("SomeOtherError");
/// assert_eq!(default_code, 0);
/// ```
#[proc_macro_derive(AsApiError, attributes(error))]
pub fn derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let ast = parse_macro_input!(input as DeriveInput); 
    let ident_name = &ast.ident;

    // Get the variants
    let enum_data = match &ast.data {
        syn::Data::Enum(data) => data,
        _ => {
            return syn::Error::new_spanned(
                &ast, "AsApiError can only be derived for enums"
            ).to_compile_error().into();
        }
    };
    let variants_data = &enum_data.variants;

    // Generate the match arms for the as_api_error method
    let match_arms_results: Vec<Result<proc_macro2::TokenStream, syn::Error>> = variants_data.iter().map(|v| {
        let variant_ident = &v.ident;
        
        // Determine the pattern for matching fields
        let field_pats = match &v.fields {
            syn::Fields::Unnamed(f) => {
                let idents = f.unnamed.iter().enumerate().map(|(i, _)| format_ident!("a{}", i));
                quote! { ( #( #idents ),* ) }
            }
            syn::Fields::Named(f) => {
                let idents = f.named.iter().map(|field| field.ident.as_ref().unwrap());
                quote! { { #( #idents ),* } }
            }
            syn::Fields::Unit => quote! {},
        };

        let opts = match Opts::from_variant(&v) {
            Ok(opts) => opts,
            Err(e) => return Err(e.into()), // darling::Error can be converted to syn::Error then to_compile_error
        };
            
        let status_code_val = if let Some(code) = opts.code {
            code
        } else if let Some(ref error_kind_str) = opts.status {
            match error_kind_str.as_str() {
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
                _ => {
                    return Err(syn::Error::new_spanned(
                        &v.ident, // Span to the variant identifier
                        format!("Invalid status attribute \"{}\" for variant {}", error_kind_str, variant_ident), // Corrected string escaping
                    ));
                }
            }
        } else {
            500 // Default status code
        };
        
        // Validate status code
        if let Err(e) = actix_web::http::StatusCode::from_u16(status_code_val) {
             return Err(syn::Error::new_spanned(
                 &v.ident, 
                 format!("Invalid status code {} for variant {}: {}", status_code_val, variant_ident, e)
            ));
        }
        
        let kind_str = opts.kind.unwrap_or_else(|| variant_ident.to_string().to_case(Case::Snake));

        // Generate the message expression
        let message_expr = match opts.msg {
            Some(ref msg_s) => {
                if opts.ignore {
                    quote! { #msg_s.to_owned() }
                } else if let syn::Fields::Unnamed(f) = &v.fields {
                    // For tuple variants, interpolate fields named a0, a1, ...
                    let field_vars = f.unnamed.iter().enumerate().map(|(i, _)| format_ident!("a{}", i));
                    quote! { format!(#msg_s, #( #field_vars ),*) }
                } else if let syn::Fields::Named(_) = &v.fields {
                    // For struct variants, msg is a format string. Fields are interpolated from local scope.
                    quote! { format!(#msg_s) } 
                } else { // Unit variants
                    quote! { #msg_s.to_owned() }
                }
            }
            None => quote! { String::new() }, // Default empty message
        };
        
        let mut details_expr = quote! { None };

        if !opts.group {
            // Check if the variant has exactly one unnamed field
            if let syn::Fields::Unnamed(fields_unnamed) = &v.fields {
                if fields_unnamed.unnamed.len() == 1 {
                    let first_field = fields_unnamed.unnamed.first().unwrap();
                    // Get the type of the first field
                    let field_ty = &first_field.ty;
                    // Convert the type to a string for comparison
                    let type_string = quote!(#field_ty).to_string();
                    
                    let field_ident = format_ident!("a0"); // Identifier for the first unnamed field

                    // Check if the type is serde_json::Value
                    if type_string == "serde_json :: Value" {
                        details_expr = quote! { Some(#field_ident.clone()) };
                    }
                    // Check if the type is Option<serde_json::Value>
                    else if type_string == "Option < serde_json :: Value >" || type_string == "std :: option :: Option < serde_json :: Value >" {
                        details_expr = quote! { #field_ident.clone() };
                    }
                }
            }
        }
        
        // Generate the ApiError construction call
        let api_error_call = if opts.group {
            // Assumes the first field of a tuple variant is 'a0' if 'group' is true
            let group_var = format_ident!("a0"); 
            quote! { #group_var.as_api_error() }
        } else {
            quote! { ApiError::new(#status_code_val, #kind_str, #message_expr, #details_expr) } 
        };

        Ok(quote! {
            #ident_name::#variant_ident #field_pats => {
                #api_error_call
            }
        })
    }).collect();

    // Handle any errors that occurred during match arm generation
    let mut compiled_match_arms = Vec::new();
    for result in match_arms_results {
        match result {
            Ok(ts) => compiled_match_arms.push(ts),
            Err(e) => return TokenStream::from(e.to_compile_error()),
        }
    }

    // Generate the final implementations
    let expanded = quote! {
        impl AsApiErrorTrait for #ident_name {
            fn as_api_error(&self) -> ApiError {
                match self {
                    #( #compiled_match_arms ),*
                }
            }
        }
    
        impl std::fmt::Debug for #ident_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                // Use the generated as_api_error method
                let api_error = self.as_api_error();
                write!(f, "{:?}", api_error)
            }
        }
    
        impl std::fmt::Display for #ident_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                // Use the generated as_api_error method
                let api_error = self.as_api_error();
                write!(f, "{}", api_error)
            }
        }
    
        impl actix_web::ResponseError for #ident_name {
            fn status_code(&self) -> actix_web::http::StatusCode {
                let api_error = self.as_api_error();
                actix_web::http::StatusCode::from_u16(api_error.code)
                    .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR)
            }
        
            fn error_response(&self) -> actix_web::HttpResponse {
                let api_error = self.as_api_error();
                let status = actix_web::http::StatusCode::from_u16(api_error.code)
                    .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);
                actix_web::HttpResponse::build(status).json(api_error)
            }
        }
    };

    TokenStream::from(expanded)
}
