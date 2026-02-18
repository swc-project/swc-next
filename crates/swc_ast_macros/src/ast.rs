use proc_macro2::TokenStream;
use quote::quote;
use syn::{Item, ItemEnum, ItemStruct};

pub fn expand_ast(item: Item) -> TokenStream {
    match item {
        Item::Enum(item_enum) => expand_enum(item_enum),
        Item::Struct(item_struct) => expand_struct(item_struct),
        _ => panic!("AST type must be enum or struct"),
    }
}

fn expand_struct(item: ItemStruct) -> TokenStream {
    let name = item.ident.clone();
    quote! {
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        pub struct #name(pub(crate) crate::NodeId);
    }
}

fn expand_enum(item: ItemEnum) -> TokenStream {
    quote! {
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        #item
    }
}
