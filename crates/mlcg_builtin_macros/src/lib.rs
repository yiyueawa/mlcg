#![forbid(unsafe_code)]

use proc_macro::TokenStream;

#[proc_macro]
pub fn include_manifest(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}
