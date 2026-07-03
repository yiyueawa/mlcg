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
    let text = std::fs::read_to_string(&manifest_path).unwrap_or_else(|error| {
        panic!(
            "failed to read manifest {}: {error}",
            manifest_path.display()
        )
    });
    let manifest: manifest::Manifest = toml::from_str(&text).unwrap_or_else(|error| {
        panic!(
            "failed to parse manifest {}: {error}",
            manifest_path.display()
        )
    });
    let generated = generate::generate(&manifest);
    quote! {
        const _: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", #path_lit));
        #generated
    }
    .into()
}
