use darling::FromVariant;
use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};


#[derive(FromVariant, Default)]
#[darling(default, attributes(error))]
struct Opts {
    code: Option<u16>,
    msg_id: String,
}


#[proc_macro_derive(ApiError, attributes(po_path, error))]
pub fn derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let ast = parse_macro_input!(input as DeriveInput); 
    let ident_name = ast.ident;

    // Get the path to the po file
    let po_path = ast.attrs.iter().find(|a| a.path.is_ident("po_path")).unwrap().tokens.to_string();

    // Get the variants
    let enum_data = match ast.data {
        syn::Data::Enum(data) => data,
        _ => panic!("ApiError can only be derived for enums"),
    };
    let variants = enum_data.variants;

    let variants = variants.iter().map(|v| {
        let ident = &v.ident;
        let opts = Opts::from_variant(&v).unwrap();
        let code = opts.code.unwrap_or(500);
        quote! {
            #ident_name::#ident => #code,
        }
    });

    let mut code = String::new();
    code.push_str(&format!("impl ApiError for {ident_name} {{\n"));
    code.push_str(" fn to_code(&self) -> u16 {\n");
    code.push_str("     match &self {\n");
    for v in variants {
        code.push_str(&v.to_string());
    }
    code.push_str("     }\n");
    code.push_str(" }\n");
    code.push_str("}\n");

    println!("{code}");

    code.parse().unwrap()
}