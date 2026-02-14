use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    AST_CRATE_PATH,
    output::{RawOutput, RustOutput, output_path},
    schema::{AstEnum, AstStruct, AstType, Schema},
    util::{
        INLINE_DATA_U32_SIZE, InlineLayout, InlineStorageMode, calculate_inline_layout,
        generate_field_to_u32, generate_u32_to_field, safe_ident,
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
        match layout.mode {
            InlineStorageMode::Partial => {
                // Partial inlining: generates both inline accessors and extra data accessors
                generate_inline_property_accessors(
                    ast,
                    schema,
                    &mut field_getters,
                    &mut field_setters,
                    &layout,
                );
                generate_extra_data_property_accessors_partial(
                    ast,
                    schema,
                    &mut field_getters,
                    &mut field_setters,
                    &layout,
                );
            }
            _ => {
                generate_inline_property_accessors(
                    ast,
                    schema,
                    &mut field_getters,
                    &mut field_setters,
                    &layout,
                );
            }
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

/// Generate getters/setters for inline storage
/// Layout (7 bytes total):
/// - Bytes 0-3: NodeData.inline_data (u32)
/// - Bytes 4-6: AstNode.inline_data ([u8; 3])
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

        // Optimization: single field at offset 0 (1-4 bytes) can directly use u32
        let is_direct_u32 =
            byte_offset == 0 && byte_size <= 4 && layout.mode == InlineStorageMode::FourBytes;

        if is_direct_u32 {
            // Generate optimized getter (direct u32 read)
            let cast_expr = generate_u32_to_field(field_ty, schema);
            field_getters.extend(quote! {
                #[inline]
                pub fn #getter_name(&self, ast: &crate::Ast) -> #ret_ty {
                    let node = unsafe { ast.get_node_unchecked(self.0) };
                    let raw = unsafe { node.data.inline_data };
                    #cast_expr
                }
            });

            // Generate optimized setter (direct u32 write)
            let to_u32_expr = generate_field_to_u32(field_ty, &field_name, schema);
            let setter_name = format_ident!("set_{}", field_name);
            field_setters.extend(quote! {
                #[inline]
                pub fn #setter_name(&self, ast: &mut crate::Ast, #field_name: #ret_ty) {
                    let node = unsafe { ast.get_node_unchecked_mut(self.0) };
                    node.data.inline_data = #to_u32_expr;
                }
            });
        } else {
            // General case: use byte packing/unpacking
            let read_expr =
                generate_inline_read(field_ty, schema, byte_offset, byte_size, &layout.mode);
            field_getters.extend(quote! {
                #[inline]
                pub fn #getter_name(&self, ast: &crate::Ast) -> #ret_ty {
                    let node = unsafe { ast.get_node_unchecked(self.0) };
                    #read_expr
                }
            });

            let setter_name = format_ident!("set_{}", field_name);
            let write_expr = generate_inline_write(
                field_ty,
                &field_name,
                byte_offset,
                byte_size,
                &layout.mode,
                schema,
            );
            field_setters.extend(quote! {
                #[inline]
                pub fn #setter_name(&self, ast: &mut crate::Ast, #field_name: #ret_ty) {
                    let node = unsafe { ast.get_node_unchecked_mut(self.0) };
                    #write_expr
                }
            });
        }
    }
}

/// Generate read expression for a field from inline storage using bit operations
fn generate_inline_read(
    field_ty: &AstType,
    schema: &Schema,
    byte_offset: usize,
    byte_size: usize,
    mode: &InlineStorageMode,
) -> TokenStream {
    // Extract raw u32 value using bit operations
    let extract_expr = generate_extract_bits(byte_offset, byte_size, mode);
    // Then, cast to the appropriate type
    let cast_expr = generate_u32_to_field(field_ty, schema);

    quote! {
        #extract_expr
        #cast_expr
    }
}

/// Generate code to extract a field value as u32 using bit operations
fn generate_extract_bits(
    byte_offset: usize,
    byte_size: usize,
    mode: &InlineStorageMode,
) -> TokenStream {
    let bit_offset = byte_offset * 8;
    let bit_size = byte_size * 8;
    let field_end = byte_offset + byte_size;

    // Create mask for the field
    let mask: u32 = if bit_size >= 32 {
        u32::MAX
    } else {
        (1u32 << bit_size) - 1
    };

    match mode {
        InlineStorageMode::FourBytes => {
            // All data is in node.data.inline_data (u32)
            if byte_offset == 0 {
                quote! {
                    let raw = (unsafe { node.data.inline_data }) & #mask;
                }
            } else {
                quote! {
                    let raw = ((unsafe { node.data.inline_data }) >> #bit_offset) & #mask;
                }
            }
        }
        InlineStorageMode::Full => {
            // Bytes 0-3 in node.data.inline_data, bytes 4-6 in node.inline_data (U24)
            if field_end <= INLINE_DATA_U32_SIZE {
                // Field entirely in inline_data u32
                if byte_offset == 0 {
                    quote! {
                        let raw = (unsafe { node.data.inline_data }) & #mask;
                    }
                } else {
                    quote! {
                        let raw = ((unsafe { node.data.inline_data }) >> #bit_offset) & #mask;
                    }
                }
            } else if byte_offset >= INLINE_DATA_U32_SIZE {
                // Field entirely in U24
                let u24_bit_offset = (byte_offset - INLINE_DATA_U32_SIZE) * 8;
                if u24_bit_offset == 0 {
                    quote! {
                        let raw = u32::from(node.inline_data) & #mask;
                    }
                } else {
                    quote! {
                        let raw = (u32::from(node.inline_data) >> #u24_bit_offset) & #mask;
                    }
                }
            } else {
                // Field spans across u32 and U24 boundary
                let bits_in_u32 = (INLINE_DATA_U32_SIZE - byte_offset) * 8;
                let mask_u32: u32 = (1u32 << bits_in_u32) - 1;
                quote! {
                    let low_bits = ((unsafe { node.data.inline_data }) >> #bit_offset) & #mask_u32;
                    let high_bits = u32::from(node.inline_data) << #bits_in_u32;
                    let raw = (low_bits | high_bits) & #mask;
                }
            }
        }
        InlineStorageMode::Partial => {
            // Only fields in u24 (bytes 4-6) are supported in layout.fields
            // Logic is same as "Field entirely in U24" case of Full mode
            debug_assert!(
                byte_offset >= INLINE_DATA_U32_SIZE,
                "Partial mode only supports fields in u24 region"
            );
            let u24_bit_offset = (byte_offset - INLINE_DATA_U32_SIZE) * 8;
            if u24_bit_offset == 0 {
                quote! {
                    let raw = u32::from(node.inline_data) & #mask;
                }
            } else {
                quote! {
                    let raw = (u32::from(node.inline_data) >> #u24_bit_offset) & #mask;
                }
            }
        }
    }
}

/// Generate write expression for a field to inline storage using bit operations
fn generate_inline_write(
    field_ty: &AstType,
    field_name: &syn::Ident,
    byte_offset: usize,
    byte_size: usize,
    mode: &InlineStorageMode,
    schema: &Schema,
) -> TokenStream {
    // First, convert the value to u32
    let to_u32 = generate_field_to_u32(field_ty, field_name, schema);
    // Then, insert using bit operations
    let insert_expr = generate_insert_bits(byte_offset, byte_size, mode);

    quote! {
        let field_val: u32 = #to_u32;
        #insert_expr
    }
}

/// Generate code to insert a field value using bit operations (read-modify-write)
fn generate_insert_bits(
    byte_offset: usize,
    byte_size: usize,
    mode: &InlineStorageMode,
) -> TokenStream {
    let bit_offset = byte_offset * 8;
    let bit_size = byte_size * 8;
    let field_end = byte_offset + byte_size;

    // Create mask for the field
    let mask: u32 = if bit_size >= 32 {
        u32::MAX
    } else {
        (1u32 << bit_size) - 1
    };

    match mode {
        InlineStorageMode::FourBytes => {
            // All data in node.data.inline_data (u32)
            let clear_mask = !(mask << bit_offset);
            if byte_offset == 0 {
                quote! {
                    let old = unsafe { node.data.inline_data };
                    node.data.inline_data = (old & #clear_mask) | (field_val & #mask);
                }
            } else {
                quote! {
                    let old = unsafe { node.data.inline_data };
                    node.data.inline_data = (old & #clear_mask) | ((field_val & #mask) << #bit_offset);
                }
            }
        }
        InlineStorageMode::Full => {
            if field_end <= INLINE_DATA_U32_SIZE {
                // Field entirely in u32
                let clear_mask = !(mask << bit_offset);
                if byte_offset == 0 {
                    quote! {
                        let old = unsafe { node.data.inline_data };
                        node.data.inline_data = (old & #clear_mask) | (field_val & #mask);
                    }
                } else {
                    quote! {
                        let old = unsafe { node.data.inline_data };
                        node.data.inline_data = (old & #clear_mask) | ((field_val & #mask) << #bit_offset);
                    }
                }
            } else if byte_offset >= INLINE_DATA_U32_SIZE {
                // Field entirely in U24
                let u24_bit_offset = (byte_offset - INLINE_DATA_U32_SIZE) * 8;
                let u24_clear_mask = !(mask << u24_bit_offset) & 0xFFFFFF; // U24 max is 24 bits
                if u24_bit_offset == 0 {
                    quote! {
                        let old = u32::from(node.inline_data);
                        node.inline_data = ((old & #u24_clear_mask) | (field_val & #mask)).into();
                    }
                } else {
                    quote! {
                        let old = u32::from(node.inline_data);
                        node.inline_data = ((old & #u24_clear_mask) | ((field_val & #mask) << #u24_bit_offset)).into();
                    }
                }
            } else {
                // Field spans across u32 and U24 boundary
                let bits_in_u32 = (INLINE_DATA_U32_SIZE - byte_offset) * 8;
                let bits_in_u24 = bit_size - bits_in_u32;
                let mask_u32: u32 = (1u32 << bits_in_u32) - 1;
                let mask_u24: u32 = (1u32 << bits_in_u24) - 1;
                let clear_mask_u32 = !(mask_u32 << bit_offset);
                let clear_mask_u24 = !mask_u24 & 0xFFFFFF;
                quote! {
                    // Update u32 part
                    let old_u32 = unsafe { node.data.inline_data };
                    node.data.inline_data = (old_u32 & #clear_mask_u32) | ((field_val & #mask_u32) << #bit_offset);
                    // Update U24 part
                    let old_u24 = u32::from(node.inline_data);
                    node.inline_data = ((old_u24 & #clear_mask_u24) | ((field_val >> #bits_in_u32) & #mask_u24)).into();
                }
            }
        }
        InlineStorageMode::Partial => {
            // Only fields in u24
            debug_assert!(
                byte_offset >= INLINE_DATA_U32_SIZE,
                "Partial mode only supports fields in u24 region"
            );
            let u24_bit_offset = (byte_offset - INLINE_DATA_U32_SIZE) * 8;
            let u24_clear_mask = !(mask << u24_bit_offset) & 0xFFFFFF;
            if u24_bit_offset == 0 {
                quote! {
                    let old = u32::from(node.inline_data);
                    node.inline_data = ((old & #u24_clear_mask) | (field_val & #mask)).into();
                }
            } else {
                quote! {
                    let old = u32::from(node.inline_data);
                    node.inline_data = ((old & #u24_clear_mask) | ((field_val & #mask) << #u24_bit_offset)).into();
                }
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
            pub fn #getter_name(&self, ast: &crate::Ast) -> #field_ty {
                let offset = unsafe { ExtraDataId::from_usize_unchecked(ast.get_node_unchecked(self.0).data.extra_data_start.index().wrapping_add(#offset)) };

                debug_assert!(offset < ast.extra_data.len());
                unsafe {
                    ExtraDataCompact::from_extra_data(
                        *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                        ast,
                    )
                }
            }
        });

        let setter_name = format_ident!("set_{}", field_name);
        field_setters.extend(quote! {
            #[inline]
            pub fn #setter_name(&self, ast: &mut crate::Ast, #field_name: #field_ty) {
                let offset = unsafe { ExtraDataId::from_usize_unchecked(ast.get_node_unchecked(self.0).data.extra_data_start.index().wrapping_add(#offset)) };

                debug_assert!(offset < ast.extra_data.len());
                unsafe {
                    *ast.extra_data.as_raw_slice_mut().get_unchecked_mut(offset.index()) = #field_name.to_extra_data();
                };
            }
        });
    }
}

/// Generate getters/setters for extra_data storage when Partial inlining is used
fn generate_extra_data_property_accessors_partial(
    ast: &AstStruct,
    schema: &Schema,
    field_getters: &mut TokenStream,
    field_setters: &mut TokenStream,
    layout: &InlineLayout,
) {
    let mut current_extra_offset = 0usize;

    for (field_idx, field) in ast.fields.iter().enumerate() {
        // Skip fields that are inlined
        if layout.fields.iter().any(|(idx, _, _)| *idx == field_idx) {
            continue;
        }

        let field_name = format_ident!("{}", field.name);
        let field_ty = &schema.types[field.type_id].repr_ident(schema);

        let offset = current_extra_offset;
        let getter_name = safe_ident(&field.name.to_case(Case::Snake));
        field_getters.extend(quote! {
            #[inline]
            pub fn #getter_name(&self, ast: &crate::Ast) -> #field_ty {
                let offset = unsafe { ExtraDataId::from_usize_unchecked(ast.get_node_unchecked(self.0).data.extra_data_start.index().wrapping_add(#offset)) };

                debug_assert!(offset < ast.extra_data.len());
                unsafe {
                    ExtraDataCompact::from_extra_data(
                        *ast.extra_data.as_raw_slice().get_unchecked(offset.index()),
                        ast,
                    )
                }
            }
        });

        let setter_name = format_ident!("set_{}", field_name);
        field_setters.extend(quote! {
            #[inline]
            pub fn #setter_name(&self, ast: &mut crate::Ast, #field_name: #field_ty) {
                let offset = unsafe { ExtraDataId::from_usize_unchecked(ast.get_node_unchecked(self.0).data.extra_data_start.index().wrapping_add(#offset)) };

                debug_assert!(offset < ast.extra_data.len());
                unsafe {
                    *ast.extra_data.as_raw_slice_mut().get_unchecked_mut(offset.index()) = #field_name.to_extra_data();
                };
            }
        });

        current_extra_offset += 1;
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
            pub fn #is_fn_name(&self) -> bool {
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
