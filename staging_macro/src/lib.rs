use proc_macro::TokenStream;
use staging_core::derive_staging;

#[proc_macro_derive(Staging, attributes(staging))]
pub fn derive(input: TokenStream) -> TokenStream {
    derive_staging(input.into()).into()
}
