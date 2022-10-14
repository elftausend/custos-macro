use quote::{quote, ToTokens};
use syn::{parse_macro_input, ItemImpl, Path, ReturnType};

#[proc_macro_attribute]
pub fn impl_stack(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as ItemImpl);
    proc_macro::TokenStream::from(add_stack_impl(input))
}

fn add_stack_impl(impl_block: ItemImpl) -> proc_macro2::TokenStream {
    let attrs = impl_block.attrs.iter().fold(quote!(), |mut acc, attr| {
        acc.extend(attr.to_token_stream());
        acc
    });
    let spawn_generics = impl_block.generics.params.to_token_stream();
    let where_clause = impl_block.generics.where_clause.as_ref().unwrap();

    if let Some(generic_type) = impl_block.generics.type_params().next() {
        if generic_type.ident != "T" {
            panic!("--> should use the datatype provided from ...? e.g. #[impl_stack(f32)]");
        }

        let impl_trait = &impl_block
            .trait_
            .as_ref()
            .unwrap()
            .1
            .to_token_stream()
            .to_string();
        let mut path_generics = impl_trait.split('<');

        let trait_name = path_generics.next().unwrap();
        let generics_no_const = path_generics.next().unwrap();
        let trait_generics = format!(
            "{}<{}, N >",
            trait_name,
            &generics_no_const[..generics_no_const.len() - 2]
        );

        let trait_path: Path = syn::parse_str(&trait_generics).unwrap();

        //let generics = remove_lit(generics);

        let methods = impl_block
            .items
            .clone()
            .into_iter()
            .flat_map(|item| match item {
                syn::ImplItem::Method(method) => Some(method),
                _ => None,
            })
            .collect::<Vec<_>>();

        let methods_updated = methods.into_iter().fold(quote!(), |mut acc, mut meth| {
            if let ReturnType::Type(_, output) = &mut meth.sig.output {
                *output = insert_const_n_to_buf(output.to_token_stream());
            }

            meth.sig.inputs = meth
                .sig
                .inputs
                .iter_mut()
                .map(|input| {
                    match input.clone() {
                        // self
                        syn::FnArg::Receiver(_) => input.clone(),
                        // other args
                        syn::FnArg::Typed(typed) => insert_const_n_to_buf(typed.to_token_stream()),
                    }
                })
                .collect();

            acc.extend(meth.to_token_stream());

            acc
        });

        //panic!("methods: {}", methods_updated.to_token_stream().to_string());

        return quote! {
            #impl_block

            #attrs
            impl<#spawn_generics, const N: usize> #trait_path for custos::stack::Stack
            #where_clause
            custos::stack::Stack: custos::Alloc<T, N>
            {
                #methods_updated
            }
        };
        //panic!("x: {}", x.to_string());
    }

    quote! {}
}

fn insert_const_n_to_buf<R: syn::parse::Parse + Clone>(tokens: proc_macro2::TokenStream) -> R {
    let tokens = tokens.to_string();
    if !tokens.contains("Buffer") {
        return syn::parse_str(&tokens).unwrap();
    }
    let mut tokens = tokens.replace("CPU", "Stack");

    let idx = tokens.find('>').unwrap();
    tokens.insert_str(idx - 1, ", N ");
    syn::parse_str(&tokens).unwrap()
}
