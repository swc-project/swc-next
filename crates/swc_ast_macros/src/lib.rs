use proc_macro::TokenStream;
use quote::quote;
use syn::{Fields, Item, parse_macro_input};

#[proc_macro_attribute]
pub fn ast(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item = parse_macro_input!(input as Item);
    strip_span_attrs(&mut item);
    quote!(#item).into()
}

fn strip_span_attrs(item: &mut Item) {
    match item {
        Item::Struct(item) => strip_fields_span_attrs(&mut item.fields),
        Item::Enum(item) => {
            for variant in &mut item.variants {
                strip_fields_span_attrs(&mut variant.fields);
            }
        }
        _ => {}
    }
}

fn strip_fields_span_attrs(fields: &mut Fields) {
    for field in fields.iter_mut() {
        field.attrs.retain(|attr| !attr.path().is_ident("span"));
    }
}
