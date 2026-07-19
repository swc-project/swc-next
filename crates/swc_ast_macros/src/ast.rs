use proc_macro2::TokenStream;
use quote::{format_ident, quote};
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
    let name = item.ident.clone();
    let data_name = format_ident!("{name}Data");
    item.ident = data_name;

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

    let constructors = item.variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let field = variant
            .fields
            .iter()
            .next()
            .expect("AST enum variants must contain one field");
        let field_ty = &field.ty;
        quote! {
            #[allow(non_snake_case)]
            #[inline]
            pub fn #variant_name(node: #field_ty) -> Self {
                Self(node.node_id())
            }
        }
    });

    quote! {
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        pub struct #name(pub(crate) crate::NodeId);

        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        #repr_attr
        #item

        impl #name {
            #(#constructors)*
        }
    }
}
