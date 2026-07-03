#![forbid(unsafe_code)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr};

mod generate;
mod manifest;

#[proc_macro]
pub fn include_manifest(input: TokenStream) -> TokenStream {
    let path_lit = parse_macro_input!(input as LitStr);
    let path = path_lit.value();
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set");
    let manifest_path = std::path::Path::new(&manifest_dir).join(&path);
    let text = match std::fs::read_to_string(&manifest_path) {
        Ok(text) => text,
        Err(_) => {
            return compile_error(format!("failed to read manifest `{path}`"));
        }
    };
    let manifest: manifest::Manifest = match toml::from_str(&text) {
        Ok(manifest) => manifest,
        Err(_) => {
            return compile_error(format!("failed to parse manifest `{path}`"));
        }
    };
    let generated = generate::generate(&manifest);
    quote! {
        const _: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", #path_lit));
        #generated
    }
    .into()
}

fn compile_error(message: String) -> TokenStream {
    quote! {
        compile_error!(#message);
    }
    .into()
}
