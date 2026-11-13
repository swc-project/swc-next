use phf::{Set as PhfSet, phf_set};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::schema::{AstType, Schema};

/// Reserved word in Rust.
/// From <https://doc.rust-lang.org/reference/keywords.html>.
pub(crate) static RESERVED_NAMES: PhfSet<&'static str> = phf_set! {
    // Strict keywords
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for", "if",
    "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return", "self", "Self",
    "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where", "while", "async",
    "await", "dyn",
    // Reserved keywords
    "abstract", "become", "box", "do", "final", "macro", "override", "priv", "typeof", "unsized",
    "virtual", "yield", "try",
    // Weak keywords
    "macro_rules", "union", // "dyn" also listed as a weak keyword, but is already on strict list
};

/// Returns `true` if `name` is a reserved word in Rust.
pub fn is_reserved_name(name: &str) -> bool {
    RESERVED_NAMES.contains(name)
}

pub fn safe_ident(name: &str) -> Ident {
    if is_reserved_name(name) {
        format_ident!("{name}_")
    } else {
        format_ident!("{name}")
    }
}

/// Generate expression to convert a field value to u32 (for inline storage)
pub fn generate_field_to_u32(
    field_ty: &AstType,
    field_name: &syn::Ident,
    _ast: TokenStream,
    schema: &Schema,
) -> TokenStream {
    match field_ty {
        AstType::Struct(_) | AstType::Enum(_) => {
            quote!(#field_name.node_id().index() as u32)
        }
        AstType::Option(ast_option) => match &schema.types[ast_option.inner_type_id] {
            AstType::Struct(_) | AstType::Enum(_) => {
                quote!(#field_name.map_or(u32::MAX, |node| node.node_id().index() as u32))
            }
            AstType::Primitive(primitive) if matches!(primitive.name, "ScopeId" | "SymbolId") => {
                quote!(#field_name.map_or(0, |id| id.index() as u32 + 1))
            }
            _ => unreachable!("Unexpected option in u32 conversion"),
        },
        AstType::Primitive(prim) => match prim.name {
            "u32" => quote!(#field_name),
            "i32" => quote!(#field_name as u32),
            "u16" | "i16" => quote!(#field_name as u32),
            "bool" => quote!(#field_name as u32),
            "u8" | "i8" => quote!(#field_name as u32),
            // Enums with #[repr(uN)] - just cast to u32
            name => {
                if let Some(&size) = schema.repr_sizes.get(name) {
                    assert!(size <= 4, "Cannot cast #[repr(u{})] enum to u32", size * 8);
                    quote!(#field_name as u32)
                } else {
                    unreachable!("Unexpected primitive in u32 conversion: {}", prim.name)
                }
            }
        },
        _ => unreachable!(),
    }
}

/// Generate an expression that reinterprets an eight-byte field as inline data.
pub fn generate_field_to_u64(
    field_ty: &AstType,
    field_name: &syn::Ident,
    schema: &Schema,
) -> TokenStream {
    match field_ty {
        AstType::Struct(_) | AstType::Enum(_) => quote!(#field_name.node_id().index() as u64),
        AstType::SubRange(_) => quote! {
            (#field_name.start.index() as u64) | ((#field_name.end.index() as u64) << 32)
        },
        AstType::Option(option) => match &schema.types[option.inner_type_id] {
            AstType::Struct(_) | AstType::Enum(_) => quote! {
                #field_name.map_or(u64::from(u32::MAX), |node| node.node_id().index() as u64)
            },
            AstType::Primitive(primitive) => match primitive.name {
                "AstUtf8" => quote!(#field_name.map_or(u64::from(u32::MAX), AstUtf8::raw)),
                "ImportPhase" => {
                    quote!(#field_name.map_or(u64::from(u8::MAX), |phase| phase as u64))
                }
                _ => unreachable!("Unexpected option in u64 conversion"),
            },
            _ => unreachable!("Unexpected option in u64 conversion"),
        },
        AstType::Primitive(primitive) => match primitive.name {
            "AstUtf8" => quote!(#field_name.raw()),
            "f64" => quote!(#field_name.to_bits()),
            _ => quote!(#field_name as u64),
        },
    }
}

/// Generate expression to convert u32 to a field type (for inline storage)
pub fn generate_u32_to_field(field_ty: &AstType, schema: &Schema) -> TokenStream {
    match field_ty {
        AstType::Struct(_) | AstType::Enum(_) => {
            let field_inner_ty = field_ty.repr_ident(schema);
            quote! {
                #field_inner_ty(NodeId::from_raw_unchecked(raw))
            }
        }
        AstType::Option(ast_option) => {
            let inner_ty = &schema.types[ast_option.inner_type_id];
            let field_inner_ident = inner_ty.repr_ident(schema);
            match inner_ty {
                AstType::Struct(_) | AstType::Enum(_) => quote! {
                    (raw != u32::MAX).then(|| {
                        #field_inner_ident(NodeId::from_raw_unchecked(raw))
                    })
                },
                AstType::Primitive(primitive)
                    if matches!(primitive.name, "ScopeId" | "SymbolId") =>
                {
                    quote! {
                        (raw != 0).then(|| unsafe {
                            <#field_inner_ident as swc_next_allocator::Idx>::from_usize_unchecked(
                                raw.wrapping_sub(1) as usize
                            )
                        })
                    }
                }
                _ => unreachable!("Unexpected option in u32 conversion"),
            }
        }
        AstType::Primitive(prim) => match prim.name {
            "bool" => quote! { raw != 0 },
            "u8" => quote! { raw as u8 },
            "i8" => quote! { raw as i8 },
            "u16" => quote! { raw as u16 },
            "i16" => quote! { raw as i16 },
            "u32" => quote! { raw },
            "i32" => quote! { raw as i32 },
            // Enums with #[repr(uN)]
            name => {
                let prim_ty = format_ident!("{}", name);
                if let Some(&size) = schema.repr_sizes.get(name) {
                    // SAFETY: The raw value was originally obtained from a valid enum discriminant
                    // during storage via `as u32` cast. Transmuting back is safe because:
                    // 1. The enum has #[repr(uN)] ensuring stable discriminant values
                    // 2. The value was validated when the enum was originally constructed
                    // 3. Inline storage is only used for AST nodes built through typed APIs
                    match size {
                        1 => quote! { unsafe { std::mem::transmute::<u8, #prim_ty>(raw as u8) } },
                        2 => quote! { unsafe { std::mem::transmute::<u16, #prim_ty>(raw as u16) } },
                        4 => quote! { unsafe { std::mem::transmute::<u32, #prim_ty>(raw) } },
                        _ => unreachable!("Unsupported repr size: {}", size),
                    }
                } else {
                    unreachable!("Unexpected primitive in u32 conversion: {}", prim.name)
                }
            }
        },
        _ => unreachable!(),
    }
}

/// Generate an expression that reconstructs an eight-byte field from inline data.
pub fn generate_u64_to_field(field_ty: &AstType, schema: &Schema) -> TokenStream {
    match field_ty {
        AstType::SubRange(_) => quote! {
            SubRange::new(ExtraDataId::from_raw(raw as u32), ExtraDataId::from_raw((raw >> 32) as u32))
        },
        AstType::Option(option) if schema.types[option.inner_type_id].name() == "AstUtf8" => {
            quote! {
                (raw != u64::from(u32::MAX)).then(|| AstUtf8::from_raw(raw))
            }
        }
        AstType::Primitive(primitive) if primitive.name == "AstUtf8" => {
            let ty = field_ty.repr_ident(schema);
            quote!(#ty::from_raw(raw))
        }
        AstType::Primitive(primitive) if primitive.name == "f64" => quote!(f64::from_bits(raw)),
        _ => unreachable!("Unexpected type in u64 conversion: {}", field_ty.name()),
    }
}

/// Number of four-byte storage units required by an out-of-line field.
pub fn extra_data_unit_count(field_ty: &AstType, schema: &Schema) -> usize {
    match field_ty {
        AstType::SubRange(_) => 2,
        AstType::Option(option) => {
            extra_data_unit_count(&schema.types[option.inner_type_id], schema)
        }
        AstType::Primitive(primitive)
            if matches!(
                primitive.name,
                "AstUtf8" | "AstWtf8" | "Span" | "u64" | "f64"
            ) =>
        {
            2
        }
        _ => 1,
    }
}

/// Generate a typed union value for a field stored outside the node payload.
pub fn generate_field_to_extra_data(
    field_ty: &AstType,
    value: TokenStream,
    schema: &Schema,
) -> TokenStream {
    let (member, value) = match field_ty {
        AstType::Struct(_) | AstType::Enum(_) => {
            (format_ident!("node_id"), quote!((#value).node_id()))
        }
        AstType::SubRange(_) => (
            format_ident!("sub_range"),
            quote!(SubRange::new((#value).start, (#value).end)),
        ),
        AstType::Option(option) => match &schema.types[option.inner_type_id] {
            AstType::Struct(_) | AstType::Enum(_) => (
                format_ident!("optional_node_id"),
                quote! {
                    (#value).map_or(u32::MAX, |node| node.node_id().index() as u32)
                },
            ),
            AstType::Primitive(primitive) if primitive.name == "AstUtf8" => (
                format_ident!("utf8"),
                quote! {
                    (#value).unwrap_or(AstUtf8 { start: u32::MAX, end: 0 })
                },
            ),
            AstType::Primitive(primitive) if primitive.name == "ImportPhase" => (
                format_ident!("optional_import_phase"),
                quote! {
                    (#value).map_or(u32::from(u8::MAX), |phase| phase as u32)
                },
            ),
            _ => unreachable!("Unexpected extra-data option"),
        },
        AstType::Primitive(primitive) => {
            let member = match primitive.name {
                "AstUtf8" => format_ident!("utf8"),
                name => unreachable!("Unexpected extra-data primitive: {name}"),
            };
            (member, value)
        }
    };
    if extra_data_unit_count(field_ty, schema) == 2 {
        quote!(ExtraData { #member: #value })
    } else {
        quote!(ExtraDataUnit { #member: #value })
    }
}

/// Generate a typed read from an extra-data union slot.
pub fn generate_extra_data_to_field(
    field_ty: &AstType,
    data: TokenStream,
    schema: &Schema,
) -> TokenStream {
    match field_ty {
        AstType::Struct(_) | AstType::Enum(_) => {
            let ty = field_ty.repr_ident(schema);
            quote!(#ty((#data).node_id))
        }
        AstType::SubRange(_) => quote! {{
            let range = (#data).sub_range;
            SubRange::new(range.start, range.end)
        }},
        AstType::Option(option) => match &schema.types[option.inner_type_id] {
            AstType::Struct(_) | AstType::Enum(_) => {
                let ty = schema.types[option.inner_type_id].repr_ident(schema);
                quote! {{
                    let raw = (#data).optional_node_id;
                    (raw != u32::MAX).then(|| #ty(NodeId::from_raw_unchecked(raw)))
                }}
            }
            AstType::Primitive(primitive) if primitive.name == "AstUtf8" => quote! {{
                let value = (#data).utf8;
                (value.start != u32::MAX || value.end != 0).then_some(value)
            }},
            AstType::Primitive(primitive) if primitive.name == "ImportPhase" => quote! {{
                let raw = (#data).optional_import_phase as u8;
                (raw != u8::MAX).then(|| std::mem::transmute::<u8, ImportPhase>(raw))
            }},
            _ => unreachable!("Unexpected extra-data option"),
        },
        AstType::Primitive(primitive) => {
            let member = match primitive.name {
                "AstUtf8" => format_ident!("utf8"),
                name => unreachable!("Unexpected extra-data primitive: {name}"),
            };
            quote!((#data).#member)
        }
    }
}
