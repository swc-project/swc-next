use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn ast(_args: TokenStream, input: TokenStream) -> TokenStream {
    input
}
