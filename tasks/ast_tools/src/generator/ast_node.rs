use std::collections::HashSet;

use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    AST_CRATE_PATH,
    output::{RawOutput, RustOutput, output_path},
    schema::{AstEnum, AstStruct, AstType, Schema},
};

pub fn ast_node(schema: &Schema) -> RawOutput {
    let mut node_impls = TokenStream::new();
    let mut data_accessors = TokenStream::new();
    let mut node_kind_variants = TokenStream::new();
    let mut node_kind_enum_checks = TokenStream::new();
    let mut node_kind_data_variants = TokenStream::new();
    let mut node_kind_data_arms = TokenStream::new();

    for ty in &schema.types {
        match ty {
            AstType::Struct(ast_struct) => {
                node_impls.extend(generate_node_impls_for_struct(ast_struct));

                let name = format_ident!("{}", ast_struct.name);
                node_kind_variants.extend(quote!(#name,));
                node_kind_data_variants.extend(quote!(#name(#name),));
                node_kind_data_arms.extend(quote! {
                    NodeKind::#name => NodeKindData::#name(#name(node_id)),
                });
            }
            AstType::Enum(ast_enum) => {
                node_impls.extend(generate_node_impls_for_enum(ast_enum));
                data_accessors.extend(generate_data_accessor(ast_enum, schema));
                node_kind_enum_checks.extend(generate_node_kind_enum_check(ast_enum, schema));
            }
            _ => {}
        }
    }

    RustOutput {
        path: output_path(AST_CRATE_PATH, "ast_node"),
        tokens: quote! {
            #![allow(unused)]
            use crate::{Ast, GetSpan, SetSpan};
            use crate::{ast::*, node_id::*};

            /// The concrete type stored at a node ID.
            #[repr(u8)]
            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            pub enum NodeKind {
                #node_kind_variants
            }

            impl NodeKind {
                #node_kind_enum_checks
            }

            /// The typed wrapper for the concrete node stored at a node ID.
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            pub enum NodeKindData {
                #node_kind_data_variants
            }

            #node_impls

            impl<'a> Ast<'a> {
                /// Returns the typed wrapper for the concrete node stored at `node_id`.
                #[inline]
                pub fn kind_data(&self, node_id: NodeId) -> NodeKindData {
                    match self.node_kind(node_id) {
                        #node_kind_data_arms
                    }
                }

                #data_accessors
            }
        },
    }
    .into()
}

fn generate_node_kind_enum_check(ast: &AstEnum, schema: &Schema) -> TokenStream {
    let fn_name = format_ident!("is_{}", ast.name.to_case(Case::Snake));
    let mut seen = HashSet::new();
    let node_kinds: Vec<_> = ast
        .variants
        .iter()
        .filter_map(|variant| variant.type_id)
        .flat_map(|type_id| concrete_node_kinds(type_id, schema))
        .filter(|node_kind| seen.insert(node_kind.clone()))
        .map(|node_kind| format_ident!("{node_kind}"))
        .collect();

    quote! {
        #[inline]
        pub const fn #fn_name(self) -> bool {
            matches!(self, #(Self::#node_kinds)|*)
        }
    }
}

fn generate_data_accessor(ast: &AstEnum, schema: &Schema) -> TokenStream {
    let wrapper_ident = format_ident!("{}", ast.name);
    let data_ident = format_ident!("{}Data", ast.name);
    let fn_name = format_ident!("{}_data", ast.name.to_case(Case::Snake));
    let mut seen = HashSet::new();
    let mut arms = TokenStream::new();

    for variant in &ast.variants {
        let Some(type_id) = variant.type_id else {
            continue;
        };
        let variant_ident = format_ident!("{}", variant.name);
        let payload_ident = schema.types[type_id].repr_ident(schema);
        let node_kinds = concrete_node_kinds(type_id, schema);
        for node_kind_name in node_kinds {
            if !seen.insert(node_kind_name.clone()) {
                continue;
            }
            let node_kind = format_ident!("{node_kind_name}");
            let arm = match (
                ast.name.as_str(),
                variant.name.as_str(),
                node_kind_name.as_str(),
            ) {
                // `Expr` also contains function and class nodes. Recover their
                // declaration/expression form from the concrete node payload.
                ("ExportDefaultDeclarationKind", "Expr", "Function") => quote! {
                    NodeKind::Function => {
                        let function = Function(node.0);
                        if function.function_type(self) == FunctionType::FunctionDeclaration {
                            #data_ident::Function(function)
                        } else {
                            #data_ident::Expr(#payload_ident(node.0))
                        }
                    },
                },
                ("ExportDefaultDeclarationKind", "Expr", "Class") => quote! {
                    NodeKind::Class => {
                        let class = Class(node.0);
                        if class.class_type(self) == ClassType::ClassDeclaration {
                            #data_ident::Class(class)
                        } else {
                            #data_ident::Expr(#payload_ident(node.0))
                        }
                    },
                },
                _ => quote! {
                    NodeKind::#node_kind => #data_ident::#variant_ident(#payload_ident(node.0)),
                },
            };
            arms.extend(arm);
        }
    }

    quote! {
        #[inline]
        pub fn #fn_name(&self, node: #wrapper_ident) -> #data_ident {
            match unsafe { *self.nodes.kind_unchecked(node.0) } {
                #arms
                // SAFETY: Typed wrappers can only be created by typed AST
                // constructors, so their node kind belongs to this enum.
                _ => unsafe { std::hint::unreachable_unchecked() },
            }
        }
    }
}

pub(super) fn concrete_node_kinds(type_id: crate::schema::TypeId, schema: &Schema) -> Vec<String> {
    match &schema.types[type_id] {
        AstType::Struct(ast) => vec![ast.name.clone()],
        AstType::Enum(ast) => ast
            .variants
            .iter()
            .filter_map(|variant| variant.type_id)
            .flat_map(|type_id| concrete_node_kinds(type_id, schema))
            .collect(),
        _ => unreachable!("AST enum variants must contain AST nodes"),
    }
}

fn generate_node_impls_for_struct(ast: &AstStruct) -> TokenStream {
    let name = format_ident!("{}", ast.name);
    generate_node_impls(&name)
}

fn generate_node_impls_for_enum(ast: &AstEnum) -> TokenStream {
    let name = format_ident!("{}", ast.name);
    generate_node_impls(&name)
}

fn generate_node_impls(name: &proc_macro2::Ident) -> TokenStream {
    quote! {
        impl #name {
            #[inline]
            pub fn node_id(self) -> NodeId {
                self.0
            }
        }

        impl AstNode for #name {
            #[inline]
            fn node_id(self) -> NodeId {
                self.0
            }
        }

        impl GetSpan for #name {
            #[inline]
            fn span(self, ast: &Ast<'_>) -> crate::Span {
                unsafe { *ast.nodes.span_unchecked(self.0) }
            }
        }

        impl SetSpan for #name {
            #[inline]
            fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
                unsafe {
                    *ast.nodes.span_mut_unchecked(self.0) = span;
                }
            }
        }
    }
}
