use std::collections::HashSet;

use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    AST_CRATE_PATH,
    output::{RawOutput, RustOutput, output_path},
    schema::{AstEnum, AstStruct, AstType, Schema},
};

pub fn ast_node_id(schema: &Schema) -> RawOutput {
    let mut impls = TokenStream::new();
    let mut data_accessors = TokenStream::new();
    for ty in schema.types.iter() {
        match ty {
            AstType::Struct(ast_struct) => {
                impls.extend(generate_node_id_for_struct(ast_struct, schema))
            }
            AstType::Enum(ast_enum) => {
                impls.extend(generate_node_id_for_enum(ast_enum, schema));
                data_accessors.extend(generate_data_accessor(ast_enum, schema));
            }
            _ => continue,
        };
    }

    let output = quote! {
            #![allow(unused)]
            use crate::{Ast, GetSpan, NodeKind, SetSpan};
            use crate::{ast::*, node_id::*};

            #impls

            impl<'a> Ast<'a> {
                #data_accessors
            }
    };

    RustOutput {
        path: output_path(AST_CRATE_PATH, "ast_node_id"),
        tokens: output,
    }
    .into()
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
        for node_kind in node_kinds {
            if !seen.insert(node_kind.clone()) {
                continue;
            }
            let node_kind = format_ident!("{node_kind}");
            arms.extend(quote! {
                NodeKind::#node_kind => #data_ident::#variant_ident(#payload_ident(node.0)),
            });
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

fn concrete_node_kinds(type_id: crate::schema::TypeId, schema: &Schema) -> Vec<String> {
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

fn generate_node_id_for_struct(ast: &AstStruct, _schema: &Schema) -> TokenStream {
    let name = format_ident!("{}", ast.name);
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

fn generate_node_id_for_enum(ast: &AstEnum, _schema: &Schema) -> TokenStream {
    let name = format_ident!("{}", ast.name);

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
