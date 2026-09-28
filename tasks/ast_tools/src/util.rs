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
    debug_assert!(matches!(
        field_ty,
        AstType::Enum(_) | AstType::SubRange(_) | AstType::Option(_) | AstType::Primitive(_)
    ));

    if let AstType::Option(option) = field_ty
        && schema.types[option.inner_type_id].name() == "AstUtf8"
    {
        return quote!(#field_name.map_or(u64::from(u32::MAX), AstUtf8::raw));
    }

    // Other encoded eight-byte representations share the extra-data format.
    quote!(#field_name.into_extra_data().raw())
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
                            <#field_inner_ident as oxc_index::Idx>::from_usize_unchecked(
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
                    // Fallback to from_extra_data for unrecognized types
                    quote! { #prim_ty::from_extra_data(raw as u64) }
                }
            }
        },
        _ => unreachable!(),
    }
}

/// Generate an expression that reconstructs an eight-byte field from inline data.
pub fn generate_u64_to_field(field_ty: &AstType, schema: &Schema) -> TokenStream {
    if let AstType::Option(option) = field_ty
        && schema.types[option.inner_type_id].name() == "AstUtf8"
    {
        return quote! {
            (raw != u64::from(u32::MAX)).then(|| AstUtf8::from_raw(raw))
        };
    }
    quote! {
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
}
