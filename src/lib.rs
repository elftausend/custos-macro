mod impl_nnapi_op;
mod impl_using_autograd;
mod trait_builds;
mod cuda;

use std::{process::Command, hash::{Hash, Hasher}, fmt::Debug};

use impl_nnapi_op::add_nnapi_op_impl;

use impl_using_autograd::add_maybe_empty_trait;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, ItemFn, ItemImpl, ItemTrait, LitStr};


/*struct MyMacroInput {
    src: String
}

impl syn::parse::Parse for MyMacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {

        let span = input.span();
        let src: String = Punctuated::<LitStr, Token![+]>::parse_separated_nonempty(input)?
            .iter()
            .map(LitStr::value)
            .collect();

        Ok(MyMacroInput {})
    }
}*/


#[proc_macro]
pub fn cuda(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // let input = syn::parse_macro_input!(input as LitStr).value();
    let input = match syn::parse::<LitStr>(input.clone()) {
        Ok(syntax_tree) => syntax_tree.value(),
        Err(_) => input.to_string(),
    };
    // let input = input.to_string();

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    input.hash(&mut hasher);
    let hash = hasher.finish();

    let input_file_path = format!("./target/{hash}.cu");

    let out_file_path = format!("./target/{hash}.ptx");

    std::fs::write(&input_file_path, input.as_bytes()).unwrap();

    let out = Command::new("nvcc")
        .arg("-c")
        .arg(input_file_path)
        .arg("-o")
        .arg(&out_file_path)
        .arg("--ptx")
        .output().unwrap();
    
    let stderr_utf8 = std::str::from_utf8(&out.stderr).unwrap();
    if !out.stderr.is_empty() {
        panic!("{stderr_utf8}")
    }

    let ptx_src = std::fs::read_to_string(out_file_path).unwrap();

    quote!(
        custos::cuda::Ptx {
            src: #ptx_src.to_string()
        }
    ).into()
    // ptx_src.to_token_stream().into()
}


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
#[proc_macro_attribute]
pub fn impl_stack(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as ItemImpl);
    proc_macro::TokenStream::from(add_stack_impl_simple(input))
}

const ERROR_MSG: &str = "Can't use #[impl_stack] on this implement block.";

fn add_stack_impl_simple(impl_block: ItemImpl) -> proc_macro2::TokenStream {
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

#[proc_macro_attribute]
pub fn stack_cpu_test(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    proc_macro::TokenStream::from(add_stack_cpu_test(input))
}

const STACK_CPU_TEST_ERROR_MSG: &str = "Can't use #[stack_cpu_test] on this implement block.";

fn add_stack_cpu_test(input: ItemFn) -> proc_macro2::TokenStream {
    let stack_test_block = input
        .to_token_stream()
        .to_string()
        .replace("cpu", "stack")
        .replace("CPU :: new()", "custos::Stack");

    let stack_test_block: proc_macro2::TokenStream =
        syn::parse_str(&stack_test_block).expect(STACK_CPU_TEST_ERROR_MSG);

    quote! {
        #[cfg(feature = "cpu")]
        #input

        #[cfg(feature = "stack")]
        #stack_test_block
    }
}

/// Implements a custos operation trait for the custos `NNapiDevice` using an array of `OperationCode`.
/// Does not support constants or type definitions.
/// The output shape should be determined by "OS" or "S".
///
/// # Example
///
/// // --- before ---
///
/// ```
/// pub trait BinaryElementWise<T, S: Shape = (), D: Device = Self>: Device {
///     fn add(&self, lhs: &Buffer<T, D, S>, rhs: &Buffer<T, D, S>) -> Buffer<T, D, S>;
///     fn mul(&self, lhs: &Buffer<T, D, S>, rhs: &Buffer<T, D, S>) -> Buffer<T, D, S>;
///     fn sub(&self, lhs: &Buffer<T, D, S>, rhs: &Buffer<T, D, S>) -> Buffer<T, D, S>;
/// }
///
///
///
/// #[cfg(feature = "nnapi")]
/// impl<T: custos::nnapi::AsOperandCode, S: Shape> BinaryElementWise<T, S> for custos::NnapiDevice {
///     fn add(&self, lhs: &Buffer<T, Self, S>, rhs: &Buffer<T, Self, S>) -> Buffer<T, Self, S> {
///         self.retrieve_with_init::<T, S>(S::LEN, |out| {
///             let activation_idx = self.add_operand(&Operand::activation()).unwrap();
///             let mut model = self.model.borrow_mut();
///
///             model
///                 .set_activation_operand_value(activation_idx as i32)
///                 .unwrap();
///             model
///                 .add_operation(
///                     OperationCode::ANEURALNETWORKS_ADD,
///                     &[lhs.ptr.idx, rhs.ptr.idx, activation_idx],
///                     &[out.ptr.idx],
///                 )
///                 .unwrap();
///         })
///     }
///
///     fn mul(&self, lhs: &Buffer<T, Self, S>, rhs: &Buffer<T, Self, S>) -> Buffer<T, Self, S> {
///         self.retrieve_with_init::<T, S>(S::LEN, |out| {
///             let activation_idx = self.add_operand(&Operand::activation()).unwrap();
///             let mut model = self.model.borrow_mut();
///
///             model
///                 .set_activation_operand_value(activation_idx as i32)
///                 .unwrap();
///             model
///                 .add_operation(
///                     OperationCode::ANEURALNETWORKS_MUL,
///                     &[lhs.ptr.idx, rhs.ptr.idx, activation_idx],
///                     &[out.ptr.idx],
///                 )
///                 .unwrap();
///         })
///     }
///
///     fn sub(&self, lhs: &Buffer<T, Self, S>, rhs: &Buffer<T, Self, S>) -> Buffer<T, Self, S> {
///         unimplemented!("This operation is not supported by NNAPI. (it is, however, this is only an example)")
///     }
/// }
///
/// // --- after ---
///
/// // This macro simplifies this implementation into a single macro line:
///
/// #[impl_nnapi_op(ANEURALNETWORKS_ADD, ANEURALNETWORKS_MUL, None)]
/// pub trait BinaryElementWise<T, S: Shape = (), D: Device = Self>: Device {
///     fn add(&self, lhs: &Buffer<T, D, S>, rhs: &Buffer<T, D, S>) -> Buffer<T, D, S>;
///     fn mul(&self, lhs: &Buffer<T, D, S>, rhs: &Buffer<T, D, S>) -> Buffer<T, D, S>;
///     fn sub(&self, lhs: &Buffer<T, D, S>, rhs: &Buffer<T, D, S>) -> Buffer<T, D, S>;
/// }
///
/// ```
#[proc_macro_attribute]
pub fn impl_nnapi_op(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as ItemTrait);
    proc_macro::TokenStream::from(add_nnapi_op_impl(input, attr.into()))
}

#[proc_macro_attribute]
pub fn using_autograd(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as ItemTrait);
    proc_macro::TokenStream::from(add_maybe_empty_trait(input))
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
