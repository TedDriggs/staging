use proc_macro::TokenStream;
use staging_core::derive_staging_with_crate_root;
use syn::parse_quote;

#[proc_macro_derive(Staging, attributes(staging))]
pub fn derive(input: TokenStream) -> TokenStream {
    derive_staging_with_crate_root(
        input.into(),
        // If this is being invoked, the caller is using the `staging` crate and may
        // not have `staging_core` crate imported, so default to `staging` as crate root.
        Some(parse_quote!(::staging)),
    )
    .into()
}
