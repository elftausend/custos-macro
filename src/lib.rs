use proc_macro2::Ident;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, ItemImpl, TypeParam, Type, punctuated::Punctuated, FnArg, token::Comma, parse_quote::ParseQuote, Signature, PatType, Pat, ReturnType};

#[proc_macro_attribute]
pub fn impl_stack(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as ItemImpl);
    proc_macro::TokenStream::from(add_stack_impl(input))
}

fn add_stack_impl(mut impl_block: ItemImpl) -> proc_macro2::TokenStream {
    
    let generics = impl_block.generics.to_token_stream();
    //let impl_trait = impl_block.trait_.unwrap().2.to_token_stream();

    if let Some(generic_type) = impl_block.generics.type_params().next() {
        if generic_type.ident != "T" {
            panic!("--> should use the datatype provided from ...? e.g. #[impl_stack(f32)]");
        }

        let generic_token_stream = generic_type.to_token_stream();
        let impl_trait = &impl_block
            .trait_
            .as_ref()
            .unwrap()
            .1
            .to_token_stream()
            .to_string();
        let mut path_generics = impl_trait.split('<');

        let trait_name = path_generics.next().unwrap().to_token_stream();
        let generics_no_const = path_generics.next().unwrap();
        let generics =
            format!("{}, N", &generics_no_const[..generics_no_const.len() - 1]).to_token_stream();

        let methods = impl_block
            .items
            .into_iter()
            .flat_map(|item| match item {
                syn::ImplItem::Method(method) => Some(method),
                _ => None,
            })
            .collect::<Vec<_>>();

        let methods_updated = methods.into_iter().fold(quote!(), |mut acc, mut meth| {
            if let ReturnType::Type(_, output) = &mut meth.sig.output {
                let stack_buf = output.to_token_stream().to_string().replace("CPU", "Stack");
                *output = insert_const_n_to_buf(stack_buf.to_token_stream(), output.clone());
            }
            
            meth.sig.inputs = meth.sig.inputs.iter_mut().map(|input| {
                match input.clone() {
                    // self
                    syn::FnArg::Receiver(_) => input.clone(),
                    // other args
                    syn::FnArg::Typed(typed) => {
                        insert_const_n_to_buf(typed.to_token_stream(), input.clone())
                    },
                }
            }).collect();
            
            acc.extend(meth.to_token_stream());

            acc
        });

        panic!("methods: {}", methods_updated.to_token_stream().to_string());

        quote! {
            impl<#generics, const N: usize> #trait_name<#generics> for custos::Stack
            where custos::Stack: custos::Alloc<#generic_token_stream, N>
            {

            }
        };
    }

    quote! {}
}

fn insert_const_n_to_buf<R: syn::parse::Parse + Clone>(tokens: proc_macro2::TokenStream, input: R) -> R {
    let mut tokens = tokens.to_string();
    if !tokens.contains("Buffer") {
        return input.clone();
    }

    let idx = tokens.find('>').unwrap();
    tokens.insert_str(idx-1, ", N ");

    syn::parse_str(&tokens).unwrap()
}
