use proc_macro::TokenStream;
use syn::{Item, parse_macro_input};

mod ast;

#[proc_macro_attribute]
pub fn ast(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Item);
    let expanded = ast::expand_ast(input);
    TokenStream::from(expanded)
}
