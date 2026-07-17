use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    AST_CRATE_PATH,
    output::{RawOutput, RustOutput, output_path},
    schema::{AstEnum, AstStruct, AstType, Schema},
    util::{
        INLINE_DATA_U64_SIZE, InlineLayout, InlineStorageMode, calculate_inline_layout,
        generate_field_to_u32, generate_field_to_u64, generate_u32_to_field, generate_u64_to_field,
        safe_ident,
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
        let write_expr =
            generate_inline_write(field_ty, &field_name, byte_offset, byte_size, schema);
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
    let cast_expr = if byte_size == INLINE_DATA_U64_SIZE {
        generate_u64_to_field()
    } else {
        generate_u32_to_field(field_ty, schema)
    };

    quote! {
        #extract_expr
        #cast_expr
    }
}

/// Generate code to extract a field value as u32 using bit operations
fn generate_extract_bits(byte_offset: usize, byte_size: usize) -> TokenStream {
    if byte_size == INLINE_DATA_U64_SIZE {
        debug_assert_eq!(byte_offset, 0);
        return quote! {
            let raw = unsafe { ast.nodes.data_unchecked(self.0).inline_data() };
        };
    }

    let bit_size = byte_size * 8;
    let mask: u32 = if bit_size >= 32 {
        u32::MAX
    } else {
        (1u32 << bit_size) - 1
    };

    if byte_offset < INLINE_DATA_U64_SIZE {
        debug_assert!(byte_offset + byte_size <= INLINE_DATA_U64_SIZE);
        let bit_offset = byte_offset * 8;
        quote! {
            let raw = ((unsafe { ast.nodes.data_unchecked(self.0).inline_data() } >> #bit_offset) as u32) & #mask;
        }
    } else {
        let word_offset = byte_offset - INLINE_DATA_U64_SIZE;
        debug_assert!(word_offset + byte_size <= 4);
        let bit_offset = word_offset * 8;
        quote! {
            let raw = (unsafe { ast.nodes.data_unchecked(self.0).trailing_inline_data() } >> #bit_offset) & #mask;
        }
    }
}

/// Generate write expression for a field to inline storage using bit operations
fn generate_inline_write(
    field_ty: &AstType,
    field_name: &syn::Ident,
    byte_offset: usize,
    byte_size: usize,
    schema: &Schema,
) -> TokenStream {
    let insert_expr = generate_insert_bits(byte_offset, byte_size);

    if byte_size == INLINE_DATA_U64_SIZE {
        let to_u64 = generate_field_to_u64(field_ty, field_name);
        quote! {
            let field_val: u64 = #to_u64;
            #insert_expr
        }
    } else {
        let to_u32 = generate_field_to_u32(field_ty, field_name, schema);
        quote! {
            let field_val: u32 = #to_u32;
            #insert_expr
        }
    }
}

/// Generate code to insert a field value using bit operations (read-modify-write)
fn generate_insert_bits(byte_offset: usize, byte_size: usize) -> TokenStream {
    if byte_size == INLINE_DATA_U64_SIZE {
        debug_assert_eq!(byte_offset, 0);
        return quote! {
            unsafe { ast.nodes.data_mut_unchecked(self.0) }.set_inline_data(field_val);
        };
    }

    let bit_size = byte_size * 8;
    let mask = if bit_size >= 64 {
        u64::MAX
    } else {
        (1u64 << bit_size) - 1
    };

    if byte_offset < INLINE_DATA_U64_SIZE {
        debug_assert!(byte_offset + byte_size <= INLINE_DATA_U64_SIZE);
        let bit_offset = byte_offset * 8;
        let clear_mask = !(mask << bit_offset);
        quote! {
            let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
            let old = payload.inline_data();
            payload.set_inline_data(
                (old & #clear_mask) | (((field_val as u64) & #mask) << #bit_offset)
            );
        }
    } else {
        let word_offset = byte_offset - INLINE_DATA_U64_SIZE;
        debug_assert!(word_offset + byte_size <= 4);
        let bit_offset = word_offset * 8;
        let mask = mask as u32;
        let clear_mask = !(mask << bit_offset);
        quote! {
            let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
            let old = unsafe { payload.trailing_inline_data() };
            unsafe {
                payload.set_trailing_inline_data(
                    (old & #clear_mask) | ((field_val & #mask) << #bit_offset)
                );
            }
        }
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
        field_getters.extend(quote! {
            #[inline]
            pub fn #getter_name<'a>(self, ast: &crate::Ast<'a>) -> #field_ty {
                let offset = unsafe { ExtraDataId::from_usize_unchecked(ast.nodes.data_unchecked(self.0).extra_data_start().index().wrapping_add(#offset)) };

                debug_assert!(offset.index() < ast.extra_data.len());
                unsafe {
                    ExtraDataCompact::from_extra_data(
                        *ast.extra_data.as_slice().get_unchecked(offset.index()),
                        ast,
                    )
                }
            }
        });

        let setter_name = format_ident!("set_{}", field_name);
        field_setters.extend(quote! {
            #[inline]
            pub fn #setter_name<'a>(self, ast: &mut crate::Ast<'a>, #field_name: #field_ty) {
                let offset = unsafe { ExtraDataId::from_usize_unchecked(ast.nodes.data_unchecked(self.0).extra_data_start().index().wrapping_add(#offset)) };

                debug_assert!(offset.index() < ast.extra_data.len());
                unsafe {
                    *ast.extra_data.as_mut_slice().get_unchecked_mut(offset.index()) = #field_name.to_extra_data();
                };
            }
        });
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

        field_getters.extend(quote! {
            #[inline]
            pub fn #getter_name<'a>(self, ast: &crate::Ast<'a>) -> #field_ty {
                let offset = unsafe { ExtraDataId::from_usize_unchecked(ast.nodes.data_unchecked(self.0).extra_data_start().index().wrapping_add(#offset)) };

                debug_assert!(offset.index() < ast.extra_data.len());
                unsafe {
                    ExtraDataCompact::from_extra_data(
                        *ast.extra_data.as_slice().get_unchecked(offset.index()),
                        ast,
                    )
                }
            }
        });

        let setter_name = format_ident!("set_{}", field_name);
        field_setters.extend(quote! {
            #[inline]
            pub fn #setter_name<'a>(self, ast: &mut crate::Ast<'a>, #field_name: #field_ty) {
                let offset = unsafe { ExtraDataId::from_usize_unchecked(ast.nodes.data_unchecked(self.0).extra_data_start().index().wrapping_add(#offset)) };

                debug_assert!(offset.index() < ast.extra_data.len());
                unsafe {
                    *ast.extra_data.as_mut_slice().get_unchecked_mut(offset.index()) = #field_name.to_extra_data();
                };
            }
        });

        extra_offset += 1;
    }
}

fn generate_property_for_enum(ast: &AstEnum, schema: &Schema) -> TokenStream {
    let name = format_ident!("{}", ast.name);

    let mut is_variant = TokenStream::new();
    let mut as_variant = TokenStream::new();

    for variant in ast.variants.iter() {
        let variant_name = format_ident!("{}", variant.name);
        let is_fn_name = format_ident!("is_{}", variant.name.to_case(Case::Snake));
        is_variant.extend(quote! {
            #[inline]
            pub fn #is_fn_name(self) -> bool {
                matches!(self, Self::#variant_name(_))
            }
        });

        let as_fn_name = format_ident!("as_{}", variant.name.to_case(Case::Snake));
        let struct_name = format_ident!("{}", schema.types[variant.type_id.unwrap()].name());
        as_variant.extend(quote! {
            #[inline]
            pub fn #as_fn_name(self) -> Option<#struct_name> {
                match self {
                    Self::#variant_name(it) => Some(it),
                    _ => None,
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
