use proc_macro2::TokenStream;
use syn::ItemFn;


pub fn add_op_expansion(input: ItemFn) -> TokenStream {
    quote::quote!()
}
