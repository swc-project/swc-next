use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    AST_CRATE_PATH,
    output::{RawOutput, RustOutput, output_path},
    schema::{AstStruct, AstType, LayoutMode, Schema},
    util::safe_ident,
};

pub fn payload_member(ast: &AstStruct) -> syn::Ident {
    safe_ident(&ast.name.to_case(Case::Snake))
}

/// Whether the field's in-memory representation already matches its wire bytes.
/// Optional values retain their explicit wire encodings.
pub fn direct_payload_field(ty: &AstType, schema: &Schema) -> bool {
    match ty {
        AstType::Struct(_) | AstType::Enum(_) | AstType::SubRange(_) => true,
        AstType::Option(_) => false,
        AstType::Primitive(primitive) => {
            matches!(
                primitive.name,
                "bool"
                    | "u8"
                    | "i8"
                    | "u16"
                    | "i16"
                    | "u32"
                    | "i32"
                    | "u64"
                    | "f64"
                    | "Span"
                    | "AstUtf8"
                    | "AstWtf8"
            ) || schema.repr_sizes.contains_key(primitive.name)
        }
    }
}

pub fn ast_payload(schema: &Schema) -> RawOutput {
    let mut layouts = TokenStream::new();
    let mut members = TokenStream::new();
    let mut size_checks = TokenStream::new();
    let mut endian_swap_arms = TokenStream::new();

    for ty in schema.types.iter() {
        let AstType::Struct(ast) = ty else { continue };
        let layout = &ast.layout;
        if layout.fields.is_empty() {
            continue;
        }
        let layout_name = format_ident!("{}Payload", ast.name);
        let member = payload_member(ast);
        members.extend(quote!(pub(crate) #member: #layout_name,));
        size_checks.extend(quote!(assert!(std::mem::size_of::<#layout_name>() == 16);));

        let mut fields = layout.payload_fields().collect::<Vec<_>>();
        fields.sort_by_key(|&(_, offset, _)| offset);
        let mut definitions = TokenStream::new();
        let mut field_checks = TokenStream::new();
        let mut endian_swaps = TokenStream::new();
        let mut cursor = 0;
        let mut pad_index = 0;
        for (field_index, offset, size) in fields {
            if cursor < offset {
                let pad = format_ident!("_pad{pad_index}");
                let len = offset - cursor;
                definitions.extend(quote!(pub(crate) #pad: [u8; #len],));
                pad_index += 1;
            }
            let name = safe_ident(&ast.fields[field_index].name);
            let field_ty = &schema.types[ast.fields[field_index].type_id];
            if size == 2 || size == 4 {
                let end = offset + size;
                endian_swaps.extend(quote!(bytes[#offset..#end].reverse();));
            } else if size == 8 {
                if matches!(field_ty, AstType::Option(_))
                    || matches!(field_ty, AstType::Primitive(primitive) if matches!(primitive.name, "f64" | "u64"))
                {
                    let end = offset + size;
                    endian_swaps.extend(quote!(bytes[#offset..#end].reverse();));
                } else {
                    let middle = offset + 4;
                    let end = offset + size;
                    endian_swaps.extend(quote! {
                        bytes[#offset..#middle].reverse();
                        bytes[#middle..#end].reverse();
                    });
                }
            }
            let ty = if direct_payload_field(field_ty, schema) {
                field_ty.repr_ident(schema)
            } else {
                match size {
                    1 => quote!(u8),
                    2 => quote!(u16),
                    4 => quote!(u32),
                    8 => quote!(u64),
                    _ => unreachable!("unsupported inline field size"),
                }
            };
            definitions.extend(quote!(pub(crate) #name: #ty,));
            field_checks.extend(quote! {
                assert!(std::mem::size_of::<#ty>() == #size);
                assert!(std::mem::offset_of!(#layout_name, #name) == #offset);
            });
            cursor = offset + size;
        }
        if layout.mode == LayoutMode::Partial {
            let extra_start = layout
                .extra_start_offset()
                .expect("partial layout has extra data");
            if cursor < extra_start {
                let pad = format_ident!("_pad{pad_index}");
                let len = extra_start - cursor;
                definitions.extend(quote!(pub(crate) #pad: [u8; #len],));
            }
            definitions.extend(quote!(pub(crate) extra_data_start: ExtraDataId,));
            field_checks.extend(quote! {
                assert!(std::mem::size_of::<ExtraDataId>() == 4);
            });
            field_checks.extend(quote! {
                assert!(std::mem::offset_of!(#layout_name, extra_data_start) == #extra_start);
            });
        } else if cursor < 16 {
            let pad = format_ident!("_pad{pad_index}");
            let len = 16 - cursor;
            definitions.extend(quote!(pub(crate) #pad: [u8; #len],));
        }
        layouts.extend(quote! {
            #[repr(C)]
            #[derive(Clone, Copy)]
            pub(crate) struct #layout_name {
                #definitions
            }
        });
        size_checks.extend(field_checks);
        let kind = format_ident!("{}", ast.name);
        if layout.mode == LayoutMode::Partial {
            let start = layout.extra_start_offset().unwrap();
            let end = start + 4;
            endian_swaps.extend(quote!(bytes[#start..#end].reverse();));
        }
        if !endian_swaps.is_empty() {
            endian_swap_arms.extend(quote!(NodeKind::#kind => { #endian_swaps },));
        }
    }

    RustOutput {
        path: output_path(AST_CRATE_PATH, "ast_payload"),
        tokens: quote! {
            #![allow(unused)]
            use crate::*;

            #[repr(C)]
            #[derive(Clone, Copy)]
            pub(crate) union NodePayload {
                pub(crate) raw: [u8; 16],
                #members
            }

            #layouts

            const _: () = {
                #size_checks
            };

            /// Convert each numeric field between native and little-endian bytes.
            /// Swapping twice restores the original representation.
            pub(crate) fn swap_payload_endian(bytes: &mut [u8; 16], kind: NodeKind) {
                match kind {
                    #endian_swap_arms
                    _ => {}
                }
            }
        },
    }
    .into()
}
