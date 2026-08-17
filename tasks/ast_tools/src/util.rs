use phf::{Set as PhfSet, phf_set};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::schema::{AstStruct, AstType, Schema};

/// Maximum bytes for fully inline storage.
pub const INLINE_TOTAL_SIZE: usize = std::mem::size_of::<u128>();
/// Bytes available to inline fields when the high word stores ExtraDataId.
pub const PARTIAL_INLINE_SIZE: usize = INLINE_TOTAL_SIZE - std::mem::size_of::<u32>();
/// Bit offset of ExtraDataId inside a partial NodePayload.
pub const EXTRA_DATA_ID_BIT_OFFSET: usize = PARTIAL_INLINE_SIZE * 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InlineStorageMode {
    /// All fields fit in the 16-byte payload.
    Full,
    /// Some fields use the low 12 bytes and the high word stores ExtraDataId.
    Partial,
}

/// Get the byte size for a field type, or None if not inlinable
pub fn field_byte_size(ty: &AstType, schema: &Schema) -> Option<usize> {
    match ty {
        // Concrete AST IDs are statically typed by their four-byte wrapper.
        AstType::Struct(_) => Some(4),
        // All typed AST values are represented by a NodeId wrapper.
        AstType::Enum(_) => Some(4),
        // Vec is represented by an 8-byte SubRange.
        AstType::Vec(_) => Some(8),
        // Optional typed inline AST IDs are 4 bytes.
        AstType::Option(ast_option) => {
            let inner_ty = &schema.types[ast_option.inner_type_id];
            match inner_ty {
                // Option<Vec<T>> is represented by an 8-byte OptionalSubRange.
                AstType::Vec(_) => Some(8),
                // Option<Struct> uses u32::MAX as its absent sentinel.
                AstType::Struct(_) => Some(4),
                // Option<Enum> uses u32::MAX as its absent sentinel.
                AstType::Enum(_) => Some(4),
                AstType::Primitive(primitive) if primitive.name == "Span" => Some(8),
                AstType::Primitive(primitive) if primitive.name == "AstUtf8" => Some(8),
                AstType::Primitive(primitive)
                    if matches!(primitive.name, "ScopeId" | "SymbolId") =>
                {
                    Some(4)
                }
                // Other Option<Primitive> representations are not encoded here.
                AstType::Primitive(_) | AstType::Option(_) => None,
            }
        }
        AstType::Primitive(prim) => {
            // Check built-in Rust primitives first
            match prim.name {
                "bool" => Some(1),
                "u8" | "i8" => Some(1),
                "u16" | "i16" => Some(2),
                "u32" | "i32" => Some(4),
                "u64" | "f64" | "Span" | "AstUtf8" | "AstWtf8" => Some(8),
                // For other types, check if they have #[repr(uN)] in schema
                name => schema.repr_sizes.get(name).copied(),
            }
        }
    }
}

/// Layout information for inline storage
#[derive(Debug, Clone)]
pub struct InlineLayout {
    pub mode: InlineStorageMode,
    /// Inlined fields with byte offsets within NodePayload.
    pub fields: Vec<(usize, usize, usize)>, // (field_index, byte_offset, byte_size)
}

/// Calculate the inline layout for a struct
/// Returns None if no field can be inlined.
pub fn calculate_inline_layout(ast: &AstStruct, schema: &Schema) -> Option<InlineLayout> {
    let mut field_info: Vec<(usize, usize)> = Vec::new(); // (field_index, size)
    let mut total_size = 0usize;
    let mut all_fields_inlinable = true;

    for (index, field) in ast.fields.iter().enumerate() {
        let field_ty = &schema.types[field.type_id];
        if let Some(size) = field_byte_size(field_ty, schema) {
            debug_assert!(size <= std::mem::size_of::<u64>());
            field_info.push((index, size));
            total_size += size;
        } else {
            all_fields_inlinable = false;
        }
    }

    if all_fields_inlinable && total_size <= INLINE_TOTAL_SIZE {
        let mut sorted_fields = field_info;
        sorted_fields.sort_by_key(|&(_, size)| std::cmp::Reverse(size));
        let mut fields = Vec::with_capacity(sorted_fields.len());
        let mut used = 0usize;

        for (field_index, size) in sorted_fields {
            fields.push((field_index, used, size));
            used += size;
        }

        fields.sort_by_key(|(field_index, _, _)| *field_index);
        return Some(InlineLayout {
            mode: InlineStorageMode::Full,
            fields,
        });
    }

    // Partial mode reserves the high u32 for ExtraDataId. Pack the smallest
    // fields first to maximize the number of inline fields and therefore
    // minimize the number of ExtraData entries.
    let mut fields = select_partial_inline_fields(field_info);

    if fields.is_empty() {
        return None;
    }

    fields.sort_by_key(|(field_index, _, _)| *field_index);
    Some(InlineLayout {
        mode: InlineStorageMode::Partial,
        fields,
    })
}

fn select_partial_inline_fields(mut field_info: Vec<(usize, usize)>) -> Vec<(usize, usize, usize)> {
    field_info.sort_by_key(|&(field_index, size)| (size, field_index));

    let mut fields = Vec::new();
    let mut used = 0usize;
    for (field_index, size) in field_info {
        if used + size <= PARTIAL_INLINE_SIZE {
            fields.push((field_index, used, size));
            used += size;
        }
    }
    fields
}

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
    ast: TokenStream,
    _schema: &Schema,
) -> TokenStream {
    debug_assert!(matches!(
        field_ty,
        AstType::Enum(_) | AstType::Vec(_) | AstType::Option(_) | AstType::Primitive(_)
    ));

    // Every eight-byte representation is defined by ExtraDataCompact. Reading
    // the union's raw word observes the same bits that will be stored inline.
    quote!(unsafe { #field_name.to_extra_data(#ast).other })
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
pub fn generate_u64_to_field(_field_ty: &AstType, _schema: &Schema) -> TokenStream {
    quote! {
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
}
