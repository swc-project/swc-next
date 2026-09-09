use std::collections::HashSet;

use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use super::ast_node::concrete_node_kinds;
use crate::{
    AST_CRATE_PATH,
    output::{RawOutput, RustOutput, output_path},
    schema::{AstEnum, AstStruct, AstStructField, AstType, Schema},
    util::{
        InlineLayout, InlineStorageMode, calculate_inline_layout, generate_field_to_u32,
        generate_field_to_u64, generate_u32_to_field, generate_u64_to_field, safe_ident,
    },
};

pub fn ast_property(schema: &Schema) -> RawOutput {
    let mut impls = TokenStream::new();
    for ty in schema.types.iter() {
        match ty {
            AstType::Struct(ast_struct) => {
                impls.extend(generate_property_for_struct(ast_struct, schema))
            }
            AstType::Enum(ast_enum) => impls.extend(generate_property_for_enum(ast_enum, schema)),
            _ => continue,
        };
    }

    let output = quote! {
            #![allow(unused, clippy::useless_conversion, clippy::identity_op, clippy::erasing_op, clippy::let_and_return)]
            use crate::*;
            use oxc_index::Idx;

            #impls
    };

    RustOutput {
        path: output_path(AST_CRATE_PATH, "ast_property"),
        tokens: output,
    }
    .into()
}

fn generate_property_for_struct(ast: &AstStruct, schema: &Schema) -> TokenStream {
    let name = format_ident!("{}", ast.name);

    let mut field_getters = TokenStream::new();
    let mut field_setters = TokenStream::new();

    // Check if this struct uses inline storage
    if let Some(layout) = calculate_inline_layout(ast, schema) {
        generate_inline_property_accessors(
            ast,
            schema,
            &mut field_getters,
            &mut field_setters,
            &layout,
        );
        if layout.mode == InlineStorageMode::Partial {
            generate_extra_data_property_accessors_partial(
                ast,
                schema,
                &mut field_getters,
                &mut field_setters,
                &layout,
            );
        }
    } else {
        generate_extra_data_property_accessors(ast, schema, &mut field_getters, &mut field_setters);
    }

    quote! {
        impl #name {
            #field_getters
            #field_setters
        }
    }
}

/// Generate getters/setters for fields stored in NodePayload.
fn generate_inline_property_accessors(
    ast: &AstStruct,
    schema: &Schema,
    field_getters: &mut TokenStream,
    field_setters: &mut TokenStream,
    layout: &InlineLayout,
) {
    for &(field_idx, byte_offset, byte_size) in &layout.fields {
        let field = &ast.fields[field_idx];
        let field_name = format_ident!("{}", field.name);
        let field_ty = &schema.types[field.type_id];
        let getter_name = safe_ident(&field.name.to_case(Case::Snake));
        let ret_ty = field_ty.repr_ident(schema);

        let read_expr = generate_inline_read(field_ty, schema, byte_offset, byte_size);
        field_getters.extend(quote! {
            #[inline]
            pub fn #getter_name<'a>(self, ast: &crate::Ast<'a>) -> #ret_ty {
                #read_expr
            }
        });

        let setter_name = format_ident!("set_{}", field_name);
        let invalidations = generate_invalidations(field);
        let write_expr = generate_inline_write(
            field_ty,
            &field_name,
            byte_offset,
            byte_size,
            schema,
            invalidations,
        );
        field_setters.extend(quote! {
            #[inline]
            pub fn #setter_name<'a>(self, ast: &mut crate::Ast<'a>, #field_name: #ret_ty) {
                #write_expr
            }
        });
    }
}

/// Generate read expression for a field from inline storage using bit operations
fn generate_inline_read(
    field_ty: &AstType,
    schema: &Schema,
    byte_offset: usize,
    byte_size: usize,
) -> TokenStream {
    let extract_expr = generate_extract_bits(byte_offset, byte_size);
    let cast_expr = if byte_size == std::mem::size_of::<u64>() {
        generate_u64_to_field(field_ty, schema)
    } else {
        generate_u32_to_field(field_ty, schema)
    };

    quote! {
        #extract_expr
        #cast_expr
    }
}

/// Generate code to extract a field value using u128 bit operations.
fn generate_extract_bits(byte_offset: usize, byte_size: usize) -> TokenStream {
    let bit_offset = byte_offset * 8;
    if byte_size == std::mem::size_of::<u64>() {
        return quote! {
            let raw =
                (unsafe { ast.nodes.data_unchecked(self.0).0 } >> #bit_offset) as u64;
        };
    }

    let bit_size = byte_size * 8;
    let mask = if bit_size >= 32 {
        u32::MAX
    } else {
        (1u32 << bit_size) - 1
    };
    quote! {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> #bit_offset) as u32) & #mask;
    }
}

/// Generate write expression for a field to inline storage using bit operations
fn generate_inline_write(
    field_ty: &AstType,
    field_name: &syn::Ident,
    byte_offset: usize,
    byte_size: usize,
    schema: &Schema,
    invalidations: TokenStream,
) -> TokenStream {
    let insert_expr = generate_insert_bits(byte_offset, byte_size);
    let change_guard = (!invalidations.is_empty()).then(|| {
        let extract_expr = generate_extract_bits(byte_offset, byte_size);
        quote! {
            #extract_expr
            if raw == field_val {
                return;
            }
        }
    });

    if byte_size == std::mem::size_of::<u64>() {
        let to_u64 = generate_field_to_u64(field_ty, field_name, schema);
        quote! {
            let field_val: u64 = #to_u64;
            #change_guard
            #insert_expr
            #invalidations
        }
    } else {
        let to_u32 = generate_field_to_u32(field_ty, field_name, quote!(ast), schema);
        quote! {
            let field_val: u32 = #to_u32;
            #change_guard
            #insert_expr
            #invalidations
        }
    }
}

/// Generate code to insert a field value using bit operations (read-modify-write)
fn generate_insert_bits(byte_offset: usize, byte_size: usize) -> TokenStream {
    let bit_size = byte_size * 8;
    let mask = if bit_size >= 128 {
        u128::MAX
    } else {
        (1u128 << bit_size) - 1
    };
    let bit_offset = byte_offset * 8;
    let clear_mask = !(mask << bit_offset);
    quote! {
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 =
            (payload.0 & #clear_mask)
            | (((field_val as u128) & #mask) << #bit_offset);
    }
}

/// Generate getters/setters for extra_data storage (original implementation)
fn generate_extra_data_property_accessors(
    ast: &AstStruct,
    schema: &Schema,
    field_getters: &mut TokenStream,
    field_setters: &mut TokenStream,
) {
    for (offset, field) in ast.fields.iter().enumerate() {
        let field_name = format_ident!("{}", field.name);
        let field_ty = &schema.types[field.type_id].repr_ident(schema);

        let getter_name = safe_ident(&field.name.to_case(Case::Snake));
        let extra_offset = generate_extra_data_offset(offset);
        field_getters.extend(quote! {
            #[inline]
            pub fn #getter_name<'a>(self, ast: &crate::Ast<'a>) -> #field_ty {
                #extra_offset
                unsafe {
                    ExtraDataValue::from_extra_data(
                        ast.extra_data.as_slice().get_unchecked(offset.index()),
                    )
                }
            }
        });

        field_setters.extend(generate_extra_data_setter(
            field,
            &field_name,
            field_ty,
            offset,
        ));
    }
}

fn generate_extra_data_property_accessors_partial(
    ast: &AstStruct,
    schema: &Schema,
    field_getters: &mut TokenStream,
    field_setters: &mut TokenStream,
    layout: &InlineLayout,
) {
    let mut extra_offset = 0usize;

    for (field_index, field) in ast.fields.iter().enumerate() {
        if layout
            .fields
            .iter()
            .any(|(inline_index, _, _)| *inline_index == field_index)
        {
            continue;
        }

        let field_name = format_ident!("{}", field.name);
        let field_ty = &schema.types[field.type_id].repr_ident(schema);
        let getter_name = safe_ident(&field.name.to_case(Case::Snake));
        let offset = extra_offset;
        let offset_expr = generate_extra_data_offset(offset);

        field_getters.extend(quote! {
            #[inline]
            pub fn #getter_name<'a>(self, ast: &crate::Ast<'a>) -> #field_ty {
                #offset_expr
                unsafe {
                    ExtraDataValue::from_extra_data(
                        ast.extra_data.as_slice().get_unchecked(offset.index()),
                    )
                }
            }
        });

        field_setters.extend(generate_extra_data_setter(
            field,
            &field_name,
            field_ty,
            offset,
        ));

        extra_offset += 1;
    }
}

fn generate_extra_data_offset(field_offset: usize) -> TokenStream {
    quote! {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
        let offset = unsafe { start.unchecked_add(#field_offset) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
    }
}

fn generate_extra_data_setter(
    field: &AstStructField,
    field_name: &syn::Ident,
    field_ty: &TokenStream,
    offset: usize,
) -> TokenStream {
    let setter_name = format_ident!("set_{}", field_name);
    let invalidations = generate_invalidations(field);
    let extra_offset = generate_extra_data_offset(offset);
    if invalidations.is_empty() {
        return quote! {
            #[inline]
            pub fn #setter_name<'a>(self, ast: &mut crate::Ast<'a>, #field_name: #field_ty) {
                #extra_offset
                unsafe {
                    *ast.extra_data.as_mut_slice().get_unchecked_mut(offset.index()) =
                        #field_name.into_extra_data();
                };
            }
        };
    }

    quote! {
        #[inline]
        pub fn #setter_name<'a>(self, ast: &mut crate::Ast<'a>, #field_name: #field_ty) {
            let field_value = #field_name.into_extra_data();
            #extra_offset
            let previous = unsafe {
                *ast.extra_data.as_slice().get_unchecked(offset.index())
            };
            if previous.raw() == field_value.raw() {
                return;
            }
            unsafe {
                *ast.extra_data.as_mut_slice().get_unchecked_mut(offset.index()) = field_value;
            };
            #invalidations
        }
    }
}

fn generate_invalidations(field: &AstStructField) -> TokenStream {
    let invalidations = field.attrs.invalidates.iter().map(|target| {
        let setter_name = format_ident!("set_{target}");
        quote! {
            self.#setter_name(ast, None);
        }
    });
    quote!(#(#invalidations)*)
}

fn generate_property_for_enum(ast: &AstEnum, schema: &Schema) -> TokenStream {
    let name = format_ident!("{}", ast.name);
    let has_contextual_variants = ast.name == "ExportDefaultDeclarationKind";

    let mut is_variant = TokenStream::new();
    let mut as_variant = TokenStream::new();
    let mut classified_node_kinds = HashSet::new();

    for variant in ast.variants.iter() {
        let is_fn_name = format_ident!("is_{}", variant.name.to_case(Case::Snake));
        let node_kind_names = concrete_node_kinds(variant.type_id.unwrap(), schema);
        let node_kinds: Vec<_> = node_kind_names
            .iter()
            .map(|name| format_ident!("{name}"))
            .collect();
        let kind_check = if has_contextual_variants {
            generate_contextual_variant_check(&variant.name, &node_kinds)
        } else {
            if let [node_kind] = node_kinds.as_slice() {
                quote!(ast.node_kind(self.0) == NodeKind::#node_kind)
            } else {
                quote!(matches!(ast.node_kind(self.0), #(NodeKind::#node_kinds)|*))
            }
        };
        let kind_load = has_contextual_variants
            .then(|| quote!(let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };));
        is_variant.extend(quote! {
            #[inline]
            pub fn #is_fn_name(self, ast: &Ast<'_>) -> bool {
                #kind_load
                #kind_check
            }
        });

        let as_fn_name = format_ident!("as_{}", variant.name.to_case(Case::Snake));
        let variant_type = schema.types[variant.type_id.unwrap()].repr_ident(schema);
        let classified_kinds: Vec<_> = node_kind_names
            .into_iter()
            .filter(|node_kind| classified_node_kinds.insert(node_kind.clone()))
            .map(|node_kind| format_ident!("{node_kind}"))
            .collect();
        let classified_check = if has_contextual_variants {
            generate_contextual_variant_check(&variant.name, &classified_kinds)
        } else {
            quote!(matches!(kind, #(NodeKind::#classified_kinds)|*))
        };
        as_variant.extend(quote! {
            #[inline]
            pub fn #as_fn_name(self, ast: &Ast<'_>) -> Option<#variant_type> {
                let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
                if #classified_check {
                    Some(#variant_type(self.0))
                } else {
                    None
                }
            }
        });
    }

    quote! {
        impl #name {
            #is_variant
            #as_variant
        }
    }
}

fn generate_contextual_variant_check(variant_name: &str, node_kinds: &[syn::Ident]) -> TokenStream {
    match variant_name {
        "Expr" => {
            let plain_node_kinds = node_kinds
                .iter()
                .filter(|node_kind| node_kind != &"Function" && node_kind != &"Class");
            quote! {
                match kind {
                    #(NodeKind::#plain_node_kinds => true,)*
                    NodeKind::Function => {
                        let function = Function(self.0);
                        function.function_type(ast) != FunctionType::FunctionDeclaration
                    }
                    NodeKind::Class => {
                        let class = Class(self.0);
                        class.class_type(ast) != ClassType::ClassDeclaration
                    }
                    _ => false,
                }
            }
        }
        "Function" => quote! {
            if kind == NodeKind::Function {
                let function = Function(self.0);
                function.function_type(ast) == FunctionType::FunctionDeclaration
            } else {
                false
            }
        },
        "Class" => quote! {
            if kind == NodeKind::Class {
                let class = Class(self.0);
                class.class_type(ast) == ClassType::ClassDeclaration
            } else {
                false
            }
        },
        _ => quote!(matches!(kind, #(NodeKind::#node_kinds)|*)),
    }
}
