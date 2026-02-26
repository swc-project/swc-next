use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    AST_CRATE_PATH,
    output::{RawOutput, RustOutput, output_path},
    schema::{AstEnum, AstStruct, AstType, Schema},
    util::flat_enum_type,
};

pub fn ast_node_id(schema: &Schema) -> RawOutput {
    let mut impls = TokenStream::new();
    for ty in schema.types.iter() {
        match ty {
            AstType::Struct(ast_struct) => {
                impls.extend(generate_node_id_for_struct(ast_struct, schema))
            }
            AstType::Enum(ast_enum) => impls.extend(generate_node_id_for_enum(ast_enum, schema)),
            _ => continue,
        };
    }

    let output = quote! {
            #![allow(unused)]
            use crate::{Ast, NodeKind};
            use crate::{ast::*, node_id::*};

            #impls
    };

    RustOutput {
        path: output_path(AST_CRATE_PATH, "ast_node_id"),
        tokens: output,
    }
    .into()
}

fn generate_node_id_for_struct(ast: &AstStruct, _schema: &Schema) -> TokenStream {
    let name = format_ident!("{}", ast.name);
    quote! {
        impl NodeIdTrait for #name {
            #[inline]
            fn node_id(&self) -> NodeId {
                self.0
            }

            #[inline]
            fn from_node_id(node_id: NodeId, ast: &Ast) -> Self {
                assert!(*ast.nodes.kind(node_id) == NodeKind::#name);
                Self(node_id)
            }

            #[inline]
            unsafe fn from_node_id_unchecked(node_id: NodeId, _ast: &Ast) -> Self {
                Self(node_id)
            }
        }
    }
}

fn generate_node_id_for_enum(ast: &AstEnum, schema: &Schema) -> TokenStream {
    let name = format_ident!("{}", ast.name);

    let mut node_id_getter_arms = TokenStream::new();
    for variant in ast.variants.iter() {
        let variant_ident = format_ident!("{}", variant.name);
        node_id_getter_arms.extend(quote! {
            Self::#variant_ident(it) => it.node_id(),
        });
    }

    let mut from_node_id_arms = TokenStream::new();
    for (_, element) in flat_enum_type(ast, schema) {
        let kind = element.kind;
        let body = element.path.iter().rev().fold(
            quote!({ #kind::from_node_id(id, ast) }),
            |acc, construcotr| quote!(#construcotr(#acc)),
        );
        from_node_id_arms.extend(quote! {
            NodeKind::#kind => #body,
        });
    }

    quote! {
        impl NodeIdTrait for #name {
            #[inline]
            fn node_id(&self) -> NodeId {
                match self {
                    #node_id_getter_arms
                }
            }

            #[inline]
            fn from_node_id(id: NodeId, ast: &Ast) -> Self {
                match ast.nodes.kind(id) {
                    #from_node_id_arms
                    _ => unreachable!(),
                }
            }

            #[inline]
            unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast) -> Self {
                Self::from_node_id(id, ast)
            }
        }
    }
}
