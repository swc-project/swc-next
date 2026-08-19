//! Generate conversions and match macros for manually expanded enum variant groups.

use std::collections::HashSet;

use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    AST_CRATE_PATH,
    output::{RawOutput, RustOutput, output_path},
    schema::{AstEnum, AstType, Schema},
};

pub fn ast_groups(schema: &Schema) -> RawOutput {
    validate_groups(schema);

    let mut impls = TokenStream::new();
    for parent in schema.types.iter().filter_map(|ty| match ty {
        AstType::Enum(ast_enum) if !ast_enum.groups.is_empty() => Some(ast_enum),
        _ => None,
    }) {
        for child_name in all_groups(&parent.name, schema) {
            impls.extend(generate_conversions(
                parent,
                enum_by_name(child_name, schema),
            ));
        }
    }

    let group_names: HashSet<_> = schema
        .types
        .iter()
        .filter_map(|ty| match ty {
            AstType::Enum(ast_enum) => Some(ast_enum.groups.iter().map(String::as_str)),
            _ => None,
        })
        .flatten()
        .collect();
    let match_macros = schema.types.iter().filter_map(|ty| match ty {
        AstType::Enum(ast_enum) if group_names.contains(ast_enum.name.as_str()) => {
            Some(generate_match_macro(ast_enum))
        }
        _ => None,
    });

    RustOutput {
        path: output_path(AST_CRATE_PATH, "ast_groups"),
        tokens: quote! {
            //! Conversions and match macros for manually expanded enum variant groups.

            use crate::{Ast, ast::*};

            #impls
            #(#match_macros)*
        },
    }
    .into()
}

fn validate_groups(schema: &Schema) {
    for parent in schema.types.iter().filter_map(|ty| match ty {
        AstType::Enum(ast_enum) if !ast_enum.groups.is_empty() => Some(ast_enum),
        _ => None,
    }) {
        for group_name in &parent.groups {
            let group = enum_by_name(group_name, schema);
            for group_variant in &group.variants {
                assert!(
                    parent.variants.iter().any(|parent_variant| {
                        parent_variant.name == group_variant.name
                            && parent_variant.type_id == group_variant.type_id
                    }),
                    "`{}` is missing `{group_name}::{}` from its manually expanded group",
                    parent.name,
                    group_variant.name,
                );
            }
        }
    }
}

fn generate_conversions(parent: &AstEnum, child: &AstEnum) -> TokenStream {
    let parent_ident = format_ident!("{}", parent.name);
    let parent_data_ident = format_ident!("{}Data", parent.name);
    let parent_data_fn = format_ident!("{}_data", parent.name.to_case(Case::Snake));
    let parent_snake = parent.name.to_case(Case::Snake);
    let child_ident = format_ident!("{}", child.name);
    let child_snake = child.name.to_case(Case::Snake);
    let match_child = format_ident!("match_{child_snake}");
    let is_child = format_ident!("is_{child_snake}");
    let as_child = format_ident!("as_{child_snake}");
    let to_child = format_ident!("to_{child_snake}");
    let as_parent = format_ident!("as_{parent_snake}");
    let child_name = &child.name;
    let parent_name = &parent.name;
    let is_doc = format!("Returns whether this node is a [`{child_name}`].");
    let as_doc = format!(
        "Converts this [`{parent_name}`] to a [`{child_name}`] when it belongs to that variant group."
    );
    let to_doc = format!("Converts this [`{parent_name}`] to a [`{child_name}`].");
    let reverse_doc = format!("Converts this [`{child_name}`] to a [`{parent_name}`].");

    quote! {
        impl #parent_ident {
            #[doc = #is_doc]
            #[inline]
            pub fn #is_child(self, ast: &Ast<'_>) -> bool {
                matches!(ast.#parent_data_fn(self), #match_child!(#parent_data_ident))
            }

            #[doc = #as_doc]
            #[inline]
            pub fn #as_child(self, ast: &Ast<'_>) -> Option<#child_ident> {
                self.#is_child(ast).then_some(#child_ident(self.0))
            }

            #[doc = #to_doc]
            ///
            /// # Panics
            /// Panics when this node is not part of the target variant group.
            #[inline]
            pub fn #to_child(self, ast: &Ast<'_>) -> #child_ident {
                self.#as_child(ast).expect("AST enum variant is not part of the target group")
            }
        }

        impl #child_ident {
            #[doc = #reverse_doc]
            #[inline]
            pub fn #as_parent(self) -> #parent_ident {
                #parent_ident(self.0)
            }
        }

        impl From<#child_ident> for #parent_ident {
            #[inline]
            fn from(value: #child_ident) -> Self {
                value.#as_parent()
            }
        }
    }
}

fn generate_match_macro(ast_enum: &AstEnum) -> TokenStream {
    let macro_ident = format_ident!("match_{}", ast_enum.name.to_case(Case::Snake));
    let enum_name = &ast_enum.name;
    let doc = format!(" Matches all variants of the [`{enum_name}`] group.");
    let variants = ast_enum.variants.iter().map(|variant| {
        let variant_ident = format_ident!("{}", variant.name);
        quote!($ty::#variant_ident(_))
    });

    quote! {
        #[doc = #doc]
        #[macro_export]
        macro_rules! #macro_ident {
            ($ty:ident) => {
                #(#variants)|*
            };
        }
        pub use #macro_ident;
    }
}

fn all_groups<'a>(parent_name: &str, schema: &'a Schema) -> Vec<&'a str> {
    fn visit<'a>(
        parent_name: &str,
        schema: &'a Schema,
        seen: &mut HashSet<&'a str>,
        output: &mut Vec<&'a str>,
    ) {
        for group_name in &enum_by_name(parent_name, schema).groups {
            if seen.insert(group_name) {
                output.push(group_name);
                visit(group_name, schema, seen, output);
            }
        }
    }

    let mut seen = HashSet::new();
    let mut output = Vec::new();
    visit(parent_name, schema, &mut seen, &mut output);
    output
}

fn enum_by_name<'a>(name: &str, schema: &'a Schema) -> &'a AstEnum {
    schema
        .types
        .iter()
        .find_map(|ty| match ty {
            AstType::Enum(ast_enum) if ast_enum.name == name => Some(ast_enum),
            _ => None,
        })
        .unwrap_or_else(|| panic!("unknown AST enum `{name}` in `group(...)`"))
}
