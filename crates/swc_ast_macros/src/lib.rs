use proc_macro::TokenStream;
use syn::{Item, Meta, Token, parse_macro_input, punctuated::Punctuated};

mod ast;

#[proc_macro_attribute]
pub fn ast(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args with Punctuated::<Meta, Token![,]>::parse_terminated);
    let input = parse_macro_input!(input as Item);
    let expanded = ast::expand_ast(args.iter().any(|arg| arg.path().is_ident("repr")), input);
    TokenStream::from(expanded)
}
