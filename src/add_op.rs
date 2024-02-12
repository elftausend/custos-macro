use proc_macro2::TokenStream;
use syn::ExprCall;

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
    let _fn_name;
    match *input.func {
        syn::Expr::Path(ref expr_path) => {
            if let Some(ident) = expr_path.path.get_ident() {
                _fn_name = ident
            } else {
                panic!();
            }
        }
        _ => panic!("Expected a function call to an identifier"),
    }

    let _args = input.args.iter().map(|arg| match arg {
        syn::Expr::Path(ref expr_path) => {
            if let Some(ident) = expr_path.path.get_ident() {
                ident.clone()
            } else {
                panic!();
            }
        }
        _ => panic!("Expected a function call to an identifier"),
    }).collect::<Vec<_>>();

    quote::quote!()
}
