use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    AST_CRATE_PATH,
    output::{RawOutput, RustOutput, output_path},
    schema::{AstEnum, AstStruct, AstType, Schema},
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
            use crate::{Ast, GetSpan, SetSpan};
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
        impl #name {
            #[inline]
            pub fn node_id(self) -> NodeId {
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

    let mut node_id_getter_arms = TokenStream::new();
    for variant in ast.variants.iter() {
        let variant_ident = format_ident!("{}", variant.name);
        node_id_getter_arms.extend(quote! {
            Self::#variant_ident(it) => it.node_id(),
        });
    }

    quote! {
        impl #name {
            #[inline]
            pub fn node_id(self) -> NodeId {
                match self {
                    #node_id_getter_arms
                }
            }

        }
        impl GetSpan for #name {
            #[inline]
            fn span(self, ast: &Ast<'_>) -> crate::Span {
                unsafe { *ast.nodes.span_unchecked(self.node_id()) }
            }
        }

        impl SetSpan for #name {
            #[inline]
            fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
                unsafe {
                    *ast.nodes.span_mut_unchecked(self.node_id()) = span;
                }
            }
        }
    }
}
