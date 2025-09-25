use std::borrow::Cow;

use darling::{FromDeriveInput, FromField, ast::Data, util::Flag};
use proc_macro2::TokenStream;
use quote::{ToTokens, TokenStreamExt, quote};
use syn::{Ident, Path, parse_quote, parse_quote_spanned, spanned::Spanned};

pub fn derive_checker(input: TokenStream) -> TokenStream {
    match try_derive_checker(input) {
        Ok(tokens) => tokens,
        Err(err) => err.write_errors(),
    }
}

fn try_derive_checker(input: TokenStream) -> darling::Result<TokenStream> {
    let receiver = Receiver::from_derive_input(&syn::parse2(input)?)?;
    let mut tokens = TokenStream::new();
    receiver.to_tokens(&mut tokens);
    Ok(tokens)
}

#[derive(Debug, Clone, FromField)]
#[darling(attributes(staging))]
struct Field {
    ident: Option<syn::Ident>,
    ty: syn::Type,
}

#[derive(Debug, Clone, FromDeriveInput)]
#[darling(attributes(staging))]
struct Receiver {
    ident: syn::Ident,
    vis: syn::Visibility,
    data: Data<(), Field>,
    /// Name for the generated checker type
    name: Option<Ident>,
    /// Path to the error type
    error: Path,
    /// The final error type to return (defaults to `error` if not specified)
    final_error: Option<Path>,
    /// Crate root path (defaults to `::staging_core` if not specified)
    crate_root: Option<Path>,
    /// If set, the generated struct will have an extra `Vec` to store errors that
    /// could not be associated with a specific field.
    additional_errors: Flag,
}

impl Receiver {
    pub fn checker_name(&self) -> Ident {
        self.name
            .clone()
            .unwrap_or_else(|| Ident::new(&format!("{}Staging", self.ident), self.ident.span()))
    }

    pub fn final_error(&self) -> &Path {
        self.final_error.as_ref().unwrap_or(&self.error)
    }

    pub fn crate_root<'a>(&'a self) -> Cow<'a, Path> {
        self.crate_root
            .as_ref()
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Owned(parse_quote!(::staging_core)))
    }

    fn additional_errors_ident(&self) -> Option<Ident> {
        if self.additional_errors.is_present() {
            Some(Ident::new(
                "additional_errors",
                self.additional_errors.span(),
            ))
        } else {
            None
        }
    }
}

impl ToTokens for Receiver {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            ident, data, vis, ..
        } = self;

        let root = self.crate_root();
        let checker_name = self.checker_name();
        let final_error = self.final_error();

        let fields = data
            .as_ref()
            .map_struct_fields(|field| ReceiverField {
                receiver: self,
                field,
            })
            .take_struct()
            .expect("Only structs are supported")
            .fields;

        let field_decls = fields.iter().map(ReceiverField::field_decl);
        let take_errors = fields.iter().map(ReceiverField::take_error);
        let initializers = fields.iter().map(ReceiverField::initializer);

        let errors_decl: Option<syn::Field> = self.additional_errors_ident().map(|ident| {
            let error = &self.error;
            parse_quote! {
                pub #ident: #root::export::Vec<#error>
            }
        });

        let errors_init: syn::Expr = if let Some(ident) = self.additional_errors_ident() {
            parse_quote! {checker.#ident}
        } else {
            parse_quote!(#root::export::Vec::new())
        };

        tokens.append_all(quote! {
            #vis struct #checker_name {
                #(#field_decls,)*
                #errors_decl
            }

            impl #root::export::TryFrom<#checker_name> for #ident {
                type Error = #final_error;

                fn try_from(checker: #checker_name) -> #root::export::Result<Self, Self::Error> {
                    let mut __errors = #errors_init;
                    #(#take_errors);*

                    if !__errors.is_empty() {
                        return #root::export::Err(__errors.into_iter().collect());
                    }

                    #root::export::Ok(#ident {
                        #(#initializers),*
                    })
                }
            }
        });
    }
}

struct ReceiverField<'a> {
    receiver: &'a Receiver,
    field: &'a Field,
}

impl<'a> ReceiverField<'a> {
    fn field_decl(&self) -> syn::Field {
        let ident = &self.field.ident;
        let ty = self.field_type();

        parse_quote! {
            pub #ident: #ty
        }
    }

    fn field_type(&self) -> syn::Type {
        let ty = &self.field.ty;
        let error = &self.receiver.error;
        let root = self.receiver.crate_root();
        parse_quote_spanned! {self.field.ty.span()=>
            #root::export::Result<#ty, #error>
        }
    }

    fn take_error(&self) -> syn::Stmt {
        let ident = self
            .field
            .ident
            .as_ref()
            .expect("Unnamed fields not supported");

        let root = self.receiver.crate_root();
        parse_quote! {
            let #ident = match checker.#ident {
                #root::export::Result::Ok(value) => Some(value),
                #root::export::Result::Err(err) => {
                    __errors.push(err);
                    None
                }
            };
        }
    }

    fn initializer(&self) -> syn::FieldValue {
        let ident = self
            .field
            .ident
            .as_ref()
            .expect("Unnamed fields not supported");
        parse_quote! {
            #ident: #ident.unwrap()
        }
    }
}

pub mod export {
    pub use std::convert::TryFrom;
    pub use std::result::Result::{self, Err, Ok};
    pub use std::vec::Vec;
}
