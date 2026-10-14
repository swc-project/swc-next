use proc_macro::TokenStream;
use syn::{Ident, Item, Token, parse_macro_input, punctuated::Punctuated};

mod ast;

#[proc_macro_attribute]
pub fn ast(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args with Punctuated::<Ident, Token![,]>::parse_terminated);
    let input = parse_macro_input!(input as Item);
    let expanded = ast::expand_ast(
        args.iter().any(|arg| arg == "repr"),
        args.iter().any(|arg| arg == "nested"),
        input,
    );
    TokenStream::from(expanded)
}

/// Scalar enum declaration metadata is consumed by ast_tools. Keep the Rust enum
/// intact and remove variant attributes before rustc checks the expanded item.
#[proc_macro_attribute]
pub fn estree(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as Item);
    if let Item::Enum(enumeration) = &mut input {
        for variant in &mut enumeration.variants {
            variant.attrs.retain(|attr| !attr.path().is_ident("estree"));
        }
    }
    quote::quote!(#input).into()
}
