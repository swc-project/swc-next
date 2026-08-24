use std::collections::HashSet;

use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    AST_CRATE_PATH,
    output::{RawOutput, RustOutput, output_path},
    schema::{AstEnum, AstStruct, AstType, Schema},
    util::safe_ident,
};

use crate::generator::ast_node::concrete_node_kinds;

pub fn ast_clone_in(schema: &Schema) -> RawOutput {
    let mut impls = TokenStream::new();
    for ty in schema.types.iter() {
        match ty {
            AstType::Struct(ast_struct) => {
                impls.extend(generate_clone_in_for_struct(ast_struct, schema))
            }
            AstType::Enum(ast_enum) => impls.extend(generate_clone_in_for_enum(ast_enum, schema)),
            _ => continue,
        };
    }

    let output = quote! {
        #![allow(unused, clippy::manual_map)]
        use crate::{Ast, CloneIn, GetSpan, NodeKind};
        use crate::{ast::*, node_id::*};

        #impls
    };

    RustOutput {
        path: output_path(AST_CRATE_PATH, "ast_clone_in"),
        tokens: output,
    }
    .into()
}

fn generate_clone_in_for_struct(ast: &AstStruct, _schema: &Schema) -> TokenStream {
    let name = format_ident!("{}", ast.name);

    let mut fields_clone_in = TokenStream::new();
    let mut fields_params = Vec::new();
    fields_clone_in.extend(quote! { let span = self.span(ast).clone_in(ast); });
    fields_params.push(quote!(span));
    for field in ast.fields.iter() {
        let field_ident = safe_ident(&field.name.to_case(Case::Snake));
        fields_clone_in.extend(quote! {
            let #field_ident = self.#field_ident(ast).clone_in(ast);
        });
        fields_params.push(quote!(#field_ident));
    }

    let fn_name = safe_ident(&ast.name.to_case(Case::Snake));
    let clone_in = quote! {
        impl<'a> CloneIn<'a> for #name {
            type Cloned = #name;

            #[inline]
            fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
                #fields_clone_in
                ast.#fn_name( #(#fields_params),* )
            }
        }
    };

    quote! {
        #clone_in
    }
}

fn generate_clone_in_for_enum(ast: &AstEnum, schema: &Schema) -> TokenStream {
    let name = format_ident!("{}", ast.name);

    let mut arms = TokenStream::new();
    let mut seen_node_kinds = HashSet::new();
    for variant in ast.variants.iter() {
        let variant_ident = format_ident!("{}", variant.name);
        let variant_type_id = variant.type_id.unwrap();
        let variant_type = schema.types[variant_type_id].repr_ident(schema);
        let mut node_kind_names: Vec<_> = concrete_node_kinds(variant_type_id, schema)
            .into_iter()
            .filter(|node_kind| seen_node_kinds.insert(node_kind.clone()))
            .collect();

        if ast.name == "ExportDefaultDeclarationKind" && variant.name == "Expr" {
            for contextual_kind in ["Function", "Class"] {
                let Some(index) = node_kind_names
                    .iter()
                    .position(|node_kind| node_kind == contextual_kind)
                else {
                    continue;
                };
                node_kind_names.remove(index);

                let node_kind = format_ident!("{contextual_kind}");
                let concrete_type = format_ident!("{contextual_kind}");
                let (type_getter, declaration_type) = if contextual_kind == "Function" {
                    (
                        format_ident!("function_type"),
                        quote!(FunctionType::FunctionDeclaration),
                    )
                } else {
                    (
                        format_ident!("class_type"),
                        quote!(ClassType::ClassDeclaration),
                    )
                };
                arms.extend(quote! {
                    NodeKind::#node_kind => {
                        let concrete = #concrete_type(self.0);
                        if concrete.#type_getter(ast) == #declaration_type {
                            Self::#node_kind(concrete.clone_in(ast))
                        } else {
                            Self::#variant_ident(#variant_type(self.0).clone_in(ast))
                        }
                    }
                });
            }
        }

        if node_kind_names.is_empty() {
            continue;
        }
        let node_kinds: Vec<_> = node_kind_names
            .iter()
            .map(|node_kind| format_ident!("{node_kind}"))
            .collect();
        arms.extend(quote! {
            #(NodeKind::#node_kinds)|* => {
                Self::#variant_ident(#variant_type(self.0).clone_in(ast))
            }
        });
    }

    let node_id_getter = quote! {
        impl<'a> CloneIn<'a> for #name {
            type Cloned = #name;

            #[inline]
            fn clone_in(&self, ast: &mut Ast<'a>) -> Self::Cloned {
                let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
                match kind {
                    #arms
                    _ => unsafe { std::hint::unreachable_unchecked() },
                }
            }
        }
    };

    quote! {
        #node_id_getter
    }
}
