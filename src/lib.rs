use quote::{quote, ToTokens};
use syn::{parse_macro_input, ItemImpl};

#[proc_macro_attribute]
/// Expands a `CPU` implementation to a `Stack` and `CPU` implementation.
///
/// # Example
///
/// ```ignore
/// #[impl_stack]
/// impl<T, D, S> ElementWise<T, D, S> for CPU
/// where
///     T: Number,
///     D: MainMemory,
///     S: Shape
/// {
///     fn add(&self, lhs: &Buffer<T, D, S>, rhs: &Buffer<T, D, S>) -> Buffer<T, CPU, S> {
///         let mut out = self.retrieve(lhs.len, (lhs, rhs));
///         cpu_element_wise(lhs, rhs, &mut out, |o, a, b| *o = a + b);
///         out
///     }
/// 
///     fn mul(&self, lhs: &Buffer<T, D, S>, rhs: &Buffer<T, D, S>) -> Buffer<T, CPU, S> {
///         let mut out = self.retrieve(lhs.len, (lhs, rhs));
///         cpu_element_wise(lhs, rhs, &mut out, |o, a, b| *o = a * b);
///         out
///     }
/// }
/// 
/// '#[impl_stack]' expands the implementation above to the following 'Stack' implementation:
/// 
/// impl<T, D, S> ElementWise<T, D, S> for Stack
/// where
///     T: Number,
///     D: MainMemory,
///     S: Shape
/// {
///     fn add(&self, lhs: &Buffer<T, D, S>, rhs: &Buffer<T, D, S>) -> Buffer<T, Stack, S> {
///         let mut out = self.retrieve(lhs.len, (lhs, rhs));
///         cpu_element_wise(lhs, rhs, &mut out, |o, a, b| *o = a + b);
///         out
///     }
/// 
///     fn mul(&self, lhs: &Buffer<T, D, S>, rhs: &Buffer<T, D, S>) -> Buffer<T, Stack, S> {
///         let mut out = self.retrieve(lhs.len, (lhs, rhs));
///         cpu_element_wise(lhs, rhs, &mut out, |o, a, b| *o = a * b);
///         out
///     }
/// }
/// 
/// // Now is it possible to execute this operations with a CPU and Stack device.
///
/// ```
pub fn impl_stack(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as ItemImpl);
    proc_macro::TokenStream::from(add_stack_impl_simpl(input))
}

const ERROR_MSG: &str = "Can't use #[impl_stack] on this implement block.";

fn add_stack_impl_simpl(impl_block: ItemImpl) -> proc_macro2::TokenStream {
    let stack_impl_block = impl_block
        .to_token_stream()
        .to_string()
        .replace("CPU", "Stack");

    let stack_impl_block: proc_macro2::TokenStream =
        syn::parse_str(&stack_impl_block).expect(ERROR_MSG);

    quote!(
        #[cfg(feature = "cpu")]
        #impl_block

        #[cfg(feature = "stack")]
        #stack_impl_block
    )
}

/*

fn add_stack_impl(impl_block: ItemImpl) -> proc_macro2::TokenStream {
    let attrs = impl_block.attrs.iter().fold(quote!(), |mut acc, attr| {
        acc.extend(attr.to_token_stream());
        acc
    });
    let spawn_generics = impl_block.generics.params.to_token_stream();
    let where_clause = impl_block.generics.where_clause.as_ref().unwrap();

    if let Some(generic_type) = impl_block.generics.type_params().next() {
        let generic_ident = &generic_type.ident;
        /*if generic_type.ident != "T" {
            panic!("{ERROR_MSG}");
            //panic!("--> should use the datatype provided from ...? e.g. #[impl_stack(f32)]");
        }*/

        let impl_trait = &impl_block
            .trait_
            .as_ref()
            .expect(ERROR_MSG)
            .1
            .to_token_stream()
            .to_string();
        let mut path_generics = impl_trait.split('<');

        let trait_name = path_generics.next().expect(ERROR_MSG);
        let generics_no_const = path_generics.next().expect(ERROR_MSG);
        let trait_generics = format!(
            "{}<{}, N >",
            trait_name,
            &generics_no_const[..generics_no_const.len() - 2]
        );

        let trait_path: Path = syn::parse_str(&trait_generics).expect(ERROR_MSG);

        //let generics = remove_lit(generics);

        let methods_updated = impl_block
            .items
            .clone()
            .into_iter()
            .flat_map(|item| match item {
                syn::ImplItem::Method(method) => Some(method),
                _ => None,
            })
            .fold(quote!(), |mut acc, mut meth| {
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
                            syn::FnArg::Typed(typed) => {
                                insert_const_n_to_buf(typed.to_token_stream())
                            }
                        }
                    })
                    .collect();

                acc.extend(meth.to_token_stream());
                acc
            });

        //panic!("methods: {}", methods_updated.to_token_stream().to_string());

        return quote! {
            #impl_block

            #[cfg(feature = "stack")]
            #attrs
            impl<#spawn_generics, const N: usize> #trait_path for custos::stack::Stack
            #where_clause
            custos::stack::Stack: custos::Alloc<#generic_ident, N>
            {
                #methods_updated
            }
        };
        //panic!("x: {}", x.to_string());
    }
    panic!("{ERROR_MSG}")
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

*/