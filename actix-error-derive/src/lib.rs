use darling::FromVariant;
use syn::{parse_macro_input, DeriveInput};
use proc_macro::TokenStream;
use quote::{quote, format_ident};
use convert_case::{Case, Casing};

#[derive(FromVariant, Default)] 
#[darling(default, attributes(api_error))]
struct Opts {
    code: Option<u16>,
    status: Option<String>,
    kind: Option<String>,
    msg: Option<String>,
    ignore: bool,
    group: bool,
}


/// Derives the `AsApiErrorTrait` for an enum, allowing it to be converted into an `ApiError`
/// suitable for Actix-Web responses. It also conditionally implements `std::fmt::Display`.
///
/// ## Attributes
///
/// Attributes are placed on enum variants using `#[api_error(...)]`:
///
/// - `code = <u16>`: Specifies a raw HTTP status code (e.g., `code = 404`).
///   If both `code` and `status` are provided, `code` takes precedence.
///
/// - `status = "<StatusCodeString>"`: Specifies the HTTP status using a predefined string.
///   (e.g., `status = "NotFound"`). See below for a list of supported strings.
///   If neither `code` nor `status` is provided, defaults to `500` (Internal Server Error).
///
/// - `kind = "<string>"`: Sets the `kind` field in the `ApiError`.
///   Defaults to the `snake_case` version of the variant name (e.g., `MyVariant` becomes `"my_variant"`).
///
/// - `msg = "<string>"`: Provides a custom error message.
///   - For variants with named fields: `msg = "Error for {field_name}"`.
///   - For variants with unnamed (tuple) fields: `msg = "Error with value {0} and {1}"`.
///   - If `msg` is not provided, the message is generated based on the `Display` trait:
///     - If this macro generates `Display` (see "Conditional `std::fmt::Display` Implementation" below), 
///       it will be the variant name or a simple format derived from it.
///     - If the user provides `Display` (e.g., via `thiserror`), that implementation is used (`self.to_string()`).
///
/// - `ignore = <bool>`: (Default: `false`)
///   - If `true`, `msg` is *not* provided, and the macro does *not* generate `Display`,
///     the message will be the variant name, and fields will not be automatically formatted into the message.
///   - This attribute does *not* prevent field interpolation if a `msg` attribute *is* provided
///     (e.g., `#[api_error(msg = "Value: {0}", ignore)] MyVariant(i32)` will still print the value).
///   - Its primary use is to simplify the message to just the variant name when no `msg` is given
///     and `Display` is not generated by this macro, overriding default field formatting.
///
/// - `group = <bool>`: (Default: `false`)
///   - If `true`, the variant is expected to hold a single field that itself implements `AsApiErrorTrait`.
///     The `as_api_error()` method of this inner error will be called.
///     Other attributes like `code`, `status`, `msg`, `kind` on the group variant are ignored.
///
/// ## Automatic `details` Field Population
///
/// If a variant is *not* a `group` and contains a single field of type `serde_json::Value`
/// or `Option<serde_json::Value>`, this field's value will automatically populate the
/// `details` field of the generated `ApiError`.
///
/// ## Conditional `std::fmt::Display` Implementation
///
/// The `std::fmt::Display` trait is implemented for the enum by this macro *if and only if*
/// at least one variant has an explicit `#[api_error(msg = "...")]` attribute.
/// - If implemented by the macro:
///   - Variants with `msg` will use that formatted message for their `Display` output.
///   - Variants without `msg` will display as their variant name (e.g., `MyEnum::VariantName` displays as "VariantName").
///
/// If no variants use `#[api_error(msg = "...")]`, you are expected to provide your own
/// `Display` implementation (e.g., using the `thiserror` crate or manually).
/// The `as_api_error` method will then use `self.to_string()` for the `ApiError` message if `msg` is not set on the variant.
///
/// ## Supported `status` Strings and Their Codes
///
/// ```rust
/// // "BadRequest" => 400
/// // "Unauthorized" => 401
/// // "Forbidden" => 403
/// // "NotFound" => 404
/// // "MethodNotAllowed" => 405
/// // "Conflict" => 409
/// // "Gone" => 410
/// // "PayloadTooLarge" => 413
/// // "UnsupportedMediaType" => 415
/// // "UnprocessableEntity" => 422
/// // "TooManyRequests" => 429
/// // "InternalServerError" => 500 (Default if no code/status is specified)
/// // "NotImplemented" => 501
/// // "BadGateway" => 502
/// // "ServiceUnavailable" => 503
/// // "GatewayTimeout" => 504
/// ```
/// Using an unsupported string in `status` will result in a compile-time error.
///
/// ## Example
///
/// ```rust
/// use actix_error_derive::AsApiError;
/// // Ensure ApiError and AsApiErrorTrait are in scope, typically via:
/// // use actix_error::{ApiError, AsApiErrorTrait}; 
/// use serde_json::json;
///
/// // Dummy AnotherErrorType for the group example
/// #[derive(Debug)]
/// pub struct AnotherErrorType;
/// impl actix_error::AsApiErrorTrait for AnotherErrorType {
///     fn as_api_error(&self) -> actix_error::ApiError {
///         actix_error::ApiError::new(401, "auth_failure", "Authentication failed".to_string(), None)
///     }
/// }
/// impl std::fmt::Display for AnotherErrorType { 
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
///         write!(f, "AnotherErrorType: Authentication Failed")
///     }
/// }
///
/// #[derive(Debug, AsApiError)]
/// pub enum MyError {
///     #[api_error(status = "NotFound", msg = "Resource not found.")]
///     NotFound, // Display will be "Resource not found."
///
///     // No msg, so if Display is macro-generated, it's "InvalidInput".
///     // If user provides Display (e.g. with thiserror), that's used for ApiError.message.
///     #[api_error(code = 400, kind = "input_validation")]
///     InvalidInput { field: String, reason: String }, 
///
///     #[api_error(status = "UnprocessableEntity", msg = "Cannot process item: {0}")]
///     Unprocessable(String), // Display will be "Cannot process item: <value>"
///
///     // 'details' will be auto-populated from the serde_json::Value field.
///     // msg is present, so Display is "Detailed error occurred."
///     #[api_error(status = "BadRequest", msg = "Detailed error occurred.")] 
///     DetailedError(serde_json::Value),
///
///     #[api_error(group)]
///     AuthError(AnotherErrorType), // Delegates to AnotherErrorType's AsApiErrorTrait
/// }
///
/// // Since MyError has variants with `msg`, `Display` is generated by AsApiError.
/// // If no variants had `msg`, you would need to implement `Display` manually or with `thiserror`:
/// //
/// // #[derive(Debug, AsApiError, thiserror::Error)] // Example with thiserror
/// // pub enum MyErrorWithoutMacroDisplay {
/// //     #[error("Item {0} was not found")] // thiserror message
/// //     #[api_error(status = "NotFound")]
/// //     NotFound(String),
/// //
/// //     #[error("Input is invalid: {reason}")]
/// //     #[api_error(code = 400, kind = "bad_input")]
/// //     InvalidInput { reason: String }
/// // }
/// ```
#[proc_macro_derive(AsApiError, attributes(api_error))]
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

    // Determine if any variant has an explicit 'msg' attribute.
    // This will decide if a Display impl should be generated by this macro.
    let mut any_variant_has_explicit_msg = false;
    for v in variants_data.iter() {
        match Opts::from_variant(v) {
            Ok(opts) => {
                if opts.msg.is_some() {
                    any_variant_has_explicit_msg = true;
                    break;
                }
            }
            Err(e) => return TokenStream::from(e.write_errors()), // Propagate error from Opts parsing
        }
    }

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
            Err(e) => return Err(e.into()),
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
                    // Handle unknown status string
                    return Err(syn::Error::new_spanned(
                        // Span to where 'status = "..."' would be, or the variant if not directly available
                        v, // Spanning to the variant is a good approximation
                        format!("Invalid status attribute \"{}\" for variant {}. Supported values are: BadRequest, Unauthorized, etc.", error_kind_str, variant_ident),
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
            )); // Removed .into() as to_compile_error is not needed here
        }
        
        let kind_str = opts.kind.unwrap_or_else(|| variant_ident.to_string().to_case(Case::Snake));

        // Generate the message expression
        let message_expr = match opts.msg {
            Some(ref msg_s) => {
                match &v.fields {
                    syn::Fields::Unnamed(f) => {
                        // For unnamed fields, format if msg_s contains placeholders and there are fields.
                        // The 'ignore' attribute does not prevent formatting for unnamed fields here.
                        if f.unnamed.is_empty() || !msg_s.contains('{') { // Heuristic: check for presence of '{'
                            quote! { #msg_s.to_owned() } // Treat as literal
                        } else {
                            let field_vars_for_format = f.unnamed.iter().enumerate().map(|(i, _)| format_ident!("a{}", i));
                            quote! { format!(#msg_s, #( #field_vars_for_format ),*) }
                        }
                    }
                    syn::Fields::Named(f) => {
                        // For named fields, format only if 'ignore' is false, msg_s has placeholders, and there are fields.
                        if opts.ignore || f.named.is_empty() || !msg_s.contains('{') { // Heuristic: check for presence of '{'
                            quote! { #msg_s.to_owned() } // Treat as literal
                        } else {
                            let named_field_idents = f.named.iter().map(|field| field.ident.as_ref().unwrap());
                            let format_assignments = named_field_idents.map(|ident| quote! { #ident = #ident }).collect::<Vec<_>>();
                            quote! { format!(#msg_s, #( #format_assignments ),*) }
                        }
                    }
                    syn::Fields::Unit => {
                        // For unit variants, msg_s is always used as a literal string.
                        quote! { #msg_s.to_owned() }
                    }
                }
            }
            None => {
                // If no `msg` attribute is provided in `api_error`:
                if any_variant_has_explicit_msg {
                    // If the macro is generating a Display impl for this enum (because some other variant has a msg),
                    // we default to the variant's name to avoid recursion with the macro-generated Display.
                    // This matches test expectations for variants like ErrorEn::MissingMessageVariant.
                    let variant_name_str = variant_ident.to_string();
                    quote! { #variant_name_str.to_owned() }
                } else {
                    // If the macro is NOT generating a Display impl (no variant has any msg attribute),
                    // we delegate to self.to_string() to allow using an external Display (e.g., from thiserror).
                    // This matches test expectations for enums like ErrorWithThiserrorDisplay.
                    quote! { self.to_string() }
                }
            }
        };
        
        let mut details_expr = quote! { None };

        // Automatic detection of a field to be used for 'details'.
        // This logic applies if the variant is not a 'group' error.
        if !opts.group {
            match &v.fields {
                syn::Fields::Named(fields_named) => {
                    for field in &fields_named.named {
                        if let Some(field_ident) = &field.ident {
                            let field_ty = &field.ty;
                            let type_string = quote!(#field_ty).to_string().replace(" ", ""); // Normalize spaces

                            if type_string == "Option<serde_json::Value>" || type_string == "std::option::Option<serde_json::Value>" {
                                details_expr = quote! { #field_ident.clone() };
                                break; // Use the first found Option<serde_json::Value> field
                            } else if type_string == "serde_json::Value" {
                                details_expr = quote! { Some(#field_ident.clone()) };
                                break; // Use the first found serde_json::Value field
                            }
                        }
                    }
                }
                syn::Fields::Unnamed(fields_unnamed) => {
                    for (i, field) in fields_unnamed.unnamed.iter().enumerate() {
                        let field_ty = &field.ty;
                        let field_pat_ident = format_ident!("a{}", i); // Field pattern is a0, a1, etc.
                        let type_string = quote!(#field_ty).to_string().replace(" ", ""); // Normalize spaces

                        if type_string == "Option<serde_json::Value>" || type_string == "std::option::Option<serde_json::Value>" {
                            details_expr = quote! { #field_pat_ident.clone() };
                            break; // Use the first found Option<serde_json::Value> field
                        } else if type_string == "serde_json::Value" {
                            details_expr = quote! { Some(#field_pat_ident.clone()) };
                            break; // Use the first found serde_json::Value field
                        }
                    }
                }
                syn::Fields::Unit => {
                    // Unit variants cannot have details fields.
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

        // If fields are destructured by field_pats but not necessarily used directly in api_error_call
        // (e.g. if message comes from self.to_string() or variant_name),
        // this dummy assignment helps to silence "unused variable" warnings.
        let dummy_field_usage = match (opts.msg.is_none(), &v.fields) {
            (true, syn::Fields::Unnamed(f)) if !f.unnamed.is_empty() && !opts.group => {
                let idents = f.unnamed.iter().enumerate().map(|(i, _)| format_ident!("a{}", i));
                quote! { let _ = (#( #idents ),*); }
            }
            (true, syn::Fields::Named(f)) if !f.named.is_empty() && !opts.group => {
                let idents = f.named.iter().map(|field| field.ident.as_ref().unwrap());
                quote! { let _ = (#( #idents ),*); }
            }
            _ => quote! {}, // No dummy usage needed if msg is Some, or it's a unit variant, or a group error
        };

        Ok(quote! {
            #ident_name::#variant_ident #field_pats => {
                #dummy_field_usage
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

    // Conditionally generate Display implementation for the enum.
    // It's generated if any variant has an explicit 'msg' attribute.
    // Otherwise, the user is expected to provide Display (e.g., via thiserror).
    let display_impl_block = if any_variant_has_explicit_msg {
        quote! {
            impl std::fmt::Display for #ident_name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    // The message for display should be consistent with ApiError's message.
                    // This message is constructed within the as_api_error method for each variant,
                    // which itself might call self.to_string() if a variant has no 'msg' attribute.
                    write!(f, "{}", self.as_api_error().message)
                }
            }
        }
    } else {
        quote! {} // Empty if no variant has an explicit 'msg' attribute.
    };

    // Generate the final implementations
    let expanded = quote! {
        impl AsApiErrorTrait for #ident_name {
            fn as_api_error(&self) -> ApiError {
                match self {
                    #(#compiled_match_arms)*
                }
            }
        }

        #display_impl_block // Include Display impl only if any_variant_has_explicit_msg is true

        // The user is expected to provide Debug, e.g., via #[derive(Debug)]
        // No Debug impl generated by this macro.
    
        impl actix_web::ResponseError for #ident_name {
            fn status_code(&self) -> actix_web::http::StatusCode {
                // Delegate to the status_code method of the ApiError generated from this enum variant.
                self.as_api_error().status_code()
            }
        
            fn error_response(&self) -> actix_web::HttpResponse {
                // Delegate to the error_response method of the ApiError generated from this enum variant.
                // This will ensure the ApiError struct (with kind, message, details) is serialized.
                self.as_api_error().error_response()
            }
        }
    };

    TokenStream::from(expanded)
}
