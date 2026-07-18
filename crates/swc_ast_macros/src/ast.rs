use proc_macro2::TokenStream;
use quote::quote;
use syn::{Expr, ExprLit, Item, ItemEnum, ItemStruct, Lit, LitInt, token::Eq};

pub fn expand_ast(repr: bool, item: Item) -> TokenStream {
    match item {
        Item::Enum(item_enum) => expand_enum(repr, item_enum),
        Item::Struct(item_struct) if !repr => expand_struct(item_struct),
        Item::Struct(_) => panic!("`#[ast(repr)]` is only valid on AST enums"),
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

fn expand_enum(repr: bool, mut item: ItemEnum) -> TokenStream {
    let repr_attr = repr.then(|| quote!(#[repr(C, u32)]));
    if repr {
        for (index, variant) in item.variants.iter_mut().enumerate() {
            assert!(
                variant.discriminant.is_none(),
                "`#[ast(repr)]` does not support explicit discriminants"
            );
            let value = LitInt::new(&(index + 1).to_string(), variant.ident.span());
            variant.discriminant = Some((
                Eq::default(),
                Expr::Lit(ExprLit {
                    attrs: Vec::new(),
                    lit: Lit::Int(value),
                }),
            ));
        }
    }

    quote! {
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        #repr_attr
        #item
    }
}
