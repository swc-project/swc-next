use proc_macro2::TokenStream;
use quote::quote;
use syn::{Item, ItemEnum, ItemStruct};

pub fn expand_ast(repr: bool, nested: bool, item: Item) -> TokenStream {
    assert!(!nested || repr, "`nested` requires `repr`");
    match item {
        Item::Enum(item_enum) => expand_enum(item_enum),
        Item::Struct(item_struct) if !repr && !nested => expand_struct(item_struct),
        Item::Struct(_) => panic!("`#[ast(repr)]` is only valid on AST enums"),
        _ => panic!("AST type must be enum or struct"),
    }
}

fn expand_struct(item: ItemStruct) -> TokenStream {
    let name = item.ident.clone();
    quote! {
        #[repr(transparent)]
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        pub struct #name(pub(crate) crate::NodeId);
    }
}

fn expand_enum(item: ItemEnum) -> TokenStream {
    let name = item.ident;
    quote! {
        #[repr(transparent)]
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        pub struct #name(pub(crate) crate::NodeId);
    }
}
