use quote::{quote, ToTokens};
use syn::{parse_macro_input, ItemImpl, TypeParam};

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
        if generic_type.ident.to_string() != "T" {
            panic!("--> should use the datatype provided from ...? e.g. #[impl_stack(f32)]");
        }

        let generic_token_stream = generic_type.to_token_stream();
        let impl_trait = &impl_block.trait_.as_ref().unwrap().1.to_token_stream().to_string();
        let mut path_generics = impl_trait.split('<');
        
        let trait_name = path_generics.next().unwrap().to_token_stream();
        let generics_no_const = path_generics.next().unwrap();
        let generics = format!("{}, N", &generics_no_const[..generics_no_const.len()-1]).to_token_stream();
        
        quote! {
            impl<#generics, const N: usize> #trait_name<#generics> for Stack
            where Stack: Alloc<#generic_token_stream, N>
            {
    
            }
        };
    }

    quote! {
        
    }
}
