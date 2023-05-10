use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::Generics;

pub fn extract_rhs_generics_to_len(
    generics: &Generics,
    type_params_len: usize,
    mut cb: impl FnMut(&Ident),
) -> TokenStream {
    generics
        .type_params()
        .take(type_params_len)
        .fold(quote!(), |mut acc, param| {
            let ident = &param.ident;

            cb(ident);
            /*// if the output shape is not the same as S
            if ident.to_string() == "OS" {
                output_generic = "OS"
            }*/

            acc.extend(quote!(#ident,));
            acc
        })
}

pub fn extract_lhs_generics_to_len(mut generics: Generics, type_params_len: usize) -> TokenStream {
    generics
        .params
        .iter_mut()
        .take(type_params_len)
        .fold(quote!(), |mut acc, param| {
            let param = match param {
                syn::GenericParam::Type(ty) => {
                    ty.default = None;
                    ty.to_token_stream()
                }
                _ => param.to_token_stream(),
            };
            acc.extend(quote!(#param,));
            acc
        })
}
