use std::path::PathBuf;

use convert_case::{Case, Casing};
use darling::{FromVariant};
use proc_macro::{self, TokenStream};
use proc_macro2::TokenTree;
use syn::{parse_macro_input, DeriveInput};

#[derive(FromVariant, Default)]
#[darling(default, attributes(error))]
struct Opts {
    code: Option<u16>,
    msg_id: Option<String>,
    kind: Option<String>,
    status: Option<String>,
}

fn get_dir_attr(attrs: &Vec<syn::Attribute>, attr_name: &str) -> Option<PathBuf> {
    let mut directory_tokens = attrs.iter().find(|attr| attr.path.is_ident(attr_name)).unwrap().tokens.clone().into_iter();
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
        panic!("The {attr_name} does not exist");
    }
    if !directory.is_dir() {
        panic!("The {attr_name} is not a directory");
    }
    
    let mut files = std::fs::read_dir(&directory).unwrap();
    if files.next().is_none() {
        panic!("The {attr_name} does not contain any files");
    }

    Some(directory)
}

#[cfg_attr(feature = "po", proc_macro_derive(AsApiError, attributes(po_directory, error)))]
#[cfg_attr(not(feature = "po"), proc_macro_derive(AsApiError, attributes(error)))]
pub fn derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let ast = parse_macro_input!(input as DeriveInput); 
    let ident_name = ast.ident;

    // Get the path to the po file
    #[cfg(feature = "po")]
    {
        let po_directory = get_dir_attr(&ast.attrs, "po_directory").unwrap();
    }

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
            syn ::Fields::Unit => None,
            _ => panic!("ApiError can only be derived for enums with tuple variants or unit variants"),
        };
        let opts = Opts::from_variant(&v).unwrap();
        let code = opts.code.unwrap_or(500);

        #[cfg(feature = "actix")]
        {
            use actix_web::http::StatusCode;
            if let Err(e) = StatusCode::from_u16(code) {
                panic!("Invalid status code for variant {}: {}", ident, e);
            }
        }
        let msg_id = opts.msg_id;
        let mut messages = String::new();
        let mut list_vars = String::new();
        // Add the default messages for the variant in a hashmap
        for (k, v) in [("en", "Invalid password {}")] {
            list_vars = String::new();
            if let Some(tuple) = tuple {
                list_vars = tuple.unnamed.iter().enumerate().map(|(i,_)| format!("a{i}")).collect::<Vec<String>>().join(", ");
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
            list_vars = format!("({})", list_vars);
        }
        format!("
            {ident_name}::{ident}{list_vars} => ApiError {{
                code: {code},
                messages: HashMap::from([{messages}]),
                kind: \"{kind}\",
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

    println!("{code}");

    code.parse().unwrap()
}

