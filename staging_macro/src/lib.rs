use proc_macro::TokenStream;
use staging_core::derive_checker;

#[proc_macro_derive(Staging, attributes(staging))]
pub fn derive(input: TokenStream) -> TokenStream {
    derive_checker(input.into()).into()
}
