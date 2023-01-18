use darling::FromVariant;
use std::path::PathBuf;

use syn::{parse_macro_input, DeriveInput};

use proc_macro2::TokenTree;
use proc_macro::TokenStream;


#[cfg(feature = "json")]
mod json;
#[cfg(feature = "json")]
use crate::json::get_json_error_messages;

#[cfg(feature = "po")]
use crate::po::get_po_error_messages;
#[cfg(feature = "po")]
mod po;


#[derive(FromVariant, Default)]
#[darling(default, attributes(error))]
struct Opts {
    code: Option<u16>,
    msg_id: Option<String>,
    kind: Option<String>,
    status: Option<String>,
}


#[cfg(any(feature = "po", feature = "json"))]
fn get_dir_attr(attrs: &Vec<syn::Attribute>, attr_name: &str) -> Option<PathBuf> {
    let mut directory_tokens = attrs.iter().find(|attr| attr.path.is_ident(attr_name)).expect("Couldn't get the attribute").tokens.clone().into_iter();
    match directory_tokens.next() {
        Some(TokenTree::Punct(punct)) if punct.as_char() == '=' => (),
        _ => panic!("Expected leading '=' in {attr_name} attribute"),
    }
    let directory = match directory_tokens.next() {
        Some(TokenTree::Literal(value)) => value.to_string(),
        _ => panic!("Expected literal in {attr_name} attribute")
    };
    let directory = directory.trim_matches('"');
    
    // Check if the directory exists and contains at least one .po file.
    let directory = std::path::PathBuf::from(directory);
    if !directory.exists() {
        panic!("The {attr_name} directory does not exist");
    }
    if !directory.is_dir() {
        panic!("The path {attr_name} is not a directory");
    }
    
    let mut files = std::fs::read_dir(&directory).expect("Couldn't read the directory");
    if files.next().is_none() {
        panic!("The path {attr_name} does not contain any files");
    }

    Some(directory)
}

#[cfg_attr(all(feature = "json", feature="po"), proc_macro_derive(AsApiError, attributes(error, msg_path)))]
#[cfg_attr(all(feature = "json", not(feature = "po")), proc_macro_derive(AsApiError, attributes(json_file, error)))]
#[cfg_attr(all(feature = "po", not(feature = "json")), proc_macro_derive(AsApiError, attributes(po_directory, error)))]
#[cfg_attr(not(any(feature = "po", feature = "json")), proc_macro_derive(AsApiError, attributes(error)))]
pub fn derive(input: TokenStream) -> TokenStream {
    use convert_case::{Case, Casing};

    // Parse the input tokens into a syntax tree
    let ast = parse_macro_input!(input as DeriveInput); 
    let ident_name = ast.ident;

    // Get the path to the po file
    #[cfg(all(feature = "po", not(feature = "json")))]
    let po_directory = get_dir_attr(&ast.attrs, "po_directory").expect("No po_directory attribute found");
    
    // Get the path to the json file
    #[cfg(all(feature = "json", not(feature = "po")))]
    let json_file = get_dir_attr(&ast.attrs, "json_file").expect("No json_file attribute found");

    #[cfg(all(feature = "json", feature = "po"))]
    let messages_catalog = {
        let path = get_dir_attr(&ast.attrs, "msg_path").expect("No path attribute found");
        // Check if the path is a directory
        if path.is_dir() {
            get_po_error_messages(path)
        } else {
            get_json_error_messages(path)
        }
    };
    
    #[cfg(all(feature = "json", not(feature = "po")))]
    let messages_catalog = get_json_error_messages(json_file);

    #[cfg(all(feature = "po", not(feature = "json")))]
    let messages_catalog = get_po_error_messages(po_directory);


    #[cfg(not(any(feature = "json", feature = "po")))]
    let messages_catalog: HashMap<String, HashMap<String, String>> = HashMap::new();

    // Get the variants
    let enum_data = match ast.data {
        syn::Data::Enum(data) => data,
        _ => panic!("ApiError can only be derived for enums"),
    };
    let variants = enum_data.variants;

    // Generate the variant's code 
    let variants = variants.iter().map(|v| {
        let ident = &v.ident;
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

        #[cfg(feature = "actix")]
        {
            use actix_web::http::StatusCode;
            if let Err(e) = StatusCode::from_u16(code) {
                panic!("Invalid status code for variant {}: {}", ident, e);
            }
        }
        // Get the messages for the variant
        let msg_id = opts.msg_id.unwrap_or_else(|| ident.to_string().to_case(Case::Snake));
        let mut messages = String::new();
        let mut list_vars = String::new();
        
        // Add the default messages for the variant in a hashmap
        for (k, v) in messages_catalog.get(&msg_id).expect(&format!("Couldn't get the messages for the variant \"{msg_id}\"")) {
            list_vars = String::new();
            let mut v = v.to_string();
            if let Some(tuple) = tuple {
                // Get the variables names and their calls
                let tup: (Vec<String>, Vec<String>)= tuple.unnamed.iter().enumerate().map(|(i, field)| {
                    if field.ty == syn::parse_str("Translation").unwrap() {
                        (format!("a{i}"), format!("a{i}.get(\"{k}\")"))
                    } else {
                        (format!("a{i}"), format!("a{i}"))
                    }
                }).unzip();
                // Get the variables names
                list_vars = tup.0.join(", ");
                // Get the variables calls
                let list_calls = tup.1.join(", ");
                // Count the number of "{}" in the message and compare it to the number of variables in the tuple
                let nb = v.matches("{}").count();
                if nb != tuple.unnamed.len() {
                    panic!("The number of variables in the message for the variant \"{msg_id}\" must be equal to the number of variables in the tuple");
                }
                messages.push_str(
                    &format!("(String::from(\"{k}\"), format!(\"{v}\", {list_calls})),")
                );
            } else if let Some(struc) = struc {
                let vars = v.split("{").skip(1).map(|s| s.split("}").next().unwrap().to_string()).collect::<Vec<String>>();
                let vars = vars.as_slice();
                // Replace all the variabels in the message
                for var in vars {
                    v = v.replace(&format!("{{{}}}", var.clone()), "{}").to_string();
                }
                list_vars = struc.named.iter().map(|f| f.ident.as_ref().unwrap().to_string()).collect::<Vec<String>>().join(", ");
                messages.push_str(
                    &format!("(String::from(\"{k}\"), format!(\"{v}\", {list_vars})),")
                );
            } else {
                messages.push_str(
                    &format!("(String::from(\"{k}\"), String::from(\"{v}\")),")
                );
            }
        }
        // Get the kind of the variant
        let kind = opts.kind.unwrap_or_else(|| ident.to_string().to_case(Case::Snake));
        // Add the tuple syntax if it exists
        if list_vars.len() > 0 {
            if struc.is_some() {
                list_vars = format!("{{ {} }}", list_vars);
            } else {
                list_vars = format!("( {} )", list_vars);
            }
        }
        format!("
            {ident_name}::{ident} {list_vars} => {{
                ApiError::new(
                    {code}, 
                    \"{kind}\",
                    HashMap::from([{messages}]), 
                )
            }},
        ")
    });

    // Implement the ApiError trait
    let mut code = String::new();
    code.push_str(&format!("impl AsApiError for {ident_name} {{\n"));
    code.push_str(" fn as_api_error(&self) -> ApiError {\n");
    code.push_str("     match &self {\n");
    for v in variants {
        code.push_str(&v.to_string());
    }
    code.push_str("\n    }\n");
    code.push_str("   }\n");
    code.push_str("}\n");

    #[cfg(feature = "verbose")]
    println!("code : {code}");

    code.parse().expect("Couldn't parse the code")
}


