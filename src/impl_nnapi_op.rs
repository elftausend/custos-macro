use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::Parser, punctuated::Punctuated, token::Comma, ExprArray, FnArg, Ident, ItemTrait,
    TraitItem,
};

use crate::trait_builds::{extract_lhs_generics_to_len, extract_rhs_generics_to_len};

pub fn add_nnapi_op_impl(
    mut input: ItemTrait,
    attr: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let mut op_enum_count = 0;

    let op_enum_ts: proc_macro2::TokenStream = attr
        .into_iter()
        .map(|attr| match attr {
            proc_macro2::TokenTree::Ident(ident) => {
                op_enum_count += 1;
                quote!(OperationCode::#ident)
            }
            proc_macro2::TokenTree::Punct(punct) => quote!(#punct),
            _ => panic!("Invalid"),
        })
        .collect();

    if op_enum_count != input.items.len() {
        panic!("The length of the provided operations does not match with the number of methods.")
    }

    let op_enums_ts_arr = quote! {
        [#op_enum_ts]
    };

    let array = syn::parse2::<ExprArray>(op_enums_ts_arr).unwrap();

    let ident = &input.ident;

    let type_params = input.generics.type_params().collect::<Vec<_>>();
    let type_params_len = type_params.len();

    let mut output_generic = "S";

    let rhs_generics = extract_rhs_generics_to_len(&input.generics, type_params_len - 1, |ident| {
        if ident.to_string() == "OS" {
            output_generic = "OS";
        }
    });

    let output_generic: Ident = syn::parse_str(output_generic)
        .expect("Should be 'S' or 'OS', which is fine to parse to an Ident");

    let lhs_generics = extract_lhs_generics_to_len(input.generics.clone(), type_params_len - 1);

    // methods
    // panic!("{:?}", input.items.iter().map(|x| x.to_token_stream().to_string()).collect::<Vec<_>>());

    let mut op_enums = array.elems.into_iter();

    let methods = &input
        .items
        .iter_mut()
        .map(|item| match item {
            TraitItem::Fn(function) => {
                let mut fun = function.sig.clone();

                let parser = Punctuated::<FnArg, Comma>::parse_terminated;
                let replace = fun
                    .inputs
                    .to_token_stream()
                    .to_string()
                    .replace("D", "custos::NnapiDevice");
                fun.inputs = parser.parse_str(&replace).expect("Invalid");

                if let syn::ReturnType::Type(_, ty) = &mut fun.output {
                    *ty = syn::parse_str(
                        &ty.into_token_stream()
                            .to_string()
                            .replace("D", "custos::NnapiDevice"),
                    )
                    .unwrap();
                }

                let param_idents = fun
                    .inputs
                    .iter()
                    .flat_map(|input| match input {
                        syn::FnArg::Typed(ty) => {
                            if !ty.ty.to_token_stream().to_string().contains("Buffer <") {
                                return None;
                            }
                            Some(
                                syn::parse2::<Ident>(ty.pat.to_token_stream())
                                    .expect("Invalid ident"),
                            )
                        }
                        _ => None,
                    })
                    .collect::<Vec<_>>();

                let param_idents = param_idents
                    .into_iter()
                    .map(|param_ident| quote!(#param_ident.ptr.idx,))
                    .collect::<TokenStream>();

                let op_enum = op_enums
                    .next()
                    .expect("Too few NNAPI operation enums were provided.");

                // panic!("{}", op_enum.to_token_stream().to_string().as_str());

                let (may_add_activation_operand, input_idxs, unimpl) = match op_enum
                    .to_token_stream()
                    .to_string()
                    .as_str()
                {
                    "OperationCode :: ANEURALNETWORKS_ADD"
                    | "OperationCode :: ANEURALNETWORKS_MUL"
                    | "OperationCode :: ANEURALNETWORKS_DIV" => (
                        Some(quote! {
                            let activation_idx = self.add_operand(&Operand::activation()).unwrap();

                            model
                                .set_activation_operand_value(activation_idx as i32)
                                .unwrap();
                        }),
                        quote!([lhs.ptr.idx, rhs.ptr.idx, activation_idx]),
                        None,
                    ),
                    "OperationCode :: None" => (
                        None,
                        quote!([]),
                        Some(quote!(
                            unimplemented!("This operation is not supported by NNAPI.");
                            #[allow(unreachable_code)]
                        )),
                    ),
                    _ => (None, quote!([#param_idents]), None),
                };

                quote! (
                    #fun {
                        #unimpl
                        self.retrieve_with_init::<T, #output_generic>(#output_generic::LEN, |out| {
                            let mut model = self.model.borrow_mut();

                            #may_add_activation_operand

                            model
                                .add_operation(
                                    #op_enum,
                                    &#input_idxs,
                                    // &[lhs.ptr.idx, rhs.ptr.idx/*, activation_idx*/],
                                    // &[#param_idents],
                                    &[out.ptr.idx],
                                )
                                .unwrap();
                        })
                    }
                )
            }
            _ => panic!("This trait is not supported for this macro."),
        })
        .collect::<TokenStream>();

    /*panic!("{}", quote! {
        #input

        #[cfg(feature = "nnapi")]
        impl <#lhs_generics> #ident <#rhs_generics> for custos::NnapiDevice
        where
            T: custos::AsOperandCode
        {
            #methods
        }
    });*/

    quote! {
        #input

        #[cfg(feature = "nnapi")]
        impl <#lhs_generics> #ident <#rhs_generics> for custos::NnapiDevice
        where
            T: custos::AsOperandCode
        {
            #methods
        }
    }
}
