use proc_macro2::TokenStream;
use syn::{ExprCall, ItemFn};

/// ```rust
/// apply_fn(x: &Buffer, out: &mut Buffer, f: fn());
/// 
/// apply_fn(x, &mut out, f);
/// 
/// self.add_op((x, &mut out, f.no_id()), |(x, out, f)| {
///     apply_fn(x, out, **f);
/// });
/// ```
pub fn add_op_expansion(input: ExprCall) -> TokenStream {
    // input.func
    quote::quote!()
}
