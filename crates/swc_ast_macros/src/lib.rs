use proc_macro::TokenStream;
use syn::{Ident, Item, Token, parse_macro_input, punctuated::Punctuated};

mod ast;

#[proc_macro_attribute]
pub fn ast(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args with Punctuated::<Ident, Token![,]>::parse_terminated);
    let input = parse_macro_input!(input as Item);
    let expanded = ast::expand_ast(args.iter().any(|arg| arg == "repr"), input);
    TokenStream::from(expanded)
}
