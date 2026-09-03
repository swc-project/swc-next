use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn bridge(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = proc_macro2::TokenStream::from(args);
    let input = proc_macro2::TokenStream::from(input);

    quote!(
        #[napi_derive::napi(#args)]
        #input
    )
    .into()
}
