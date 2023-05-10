use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::ItemTrait;

use crate::trait_builds::extract_lhs_generics_to_len;

pub fn add_maybe_empty_trait(input: ItemTrait) -> TokenStream {
    // panic!("{}", input.supertraits.to_token_stream().to_string());

    let ident = &input.ident;
    let generics = &input.generics;
    let super_traits = &input.supertraits;

    let type_params_len = input.generics.type_params().count();
    let lhs_generics = extract_lhs_generics_to_len(input.generics.clone(), type_params_len);

    panic!("gens: {:?}", generics.to_token_stream().to_string());

    quote! {
        #[cfg(feature="autograd")]
        #input

        #[cfg(not(feature="autograd"))]
        pub trait #ident #generics : #super_traits {}

        #[cfg(not(feature="autograd"))]

    }
}
