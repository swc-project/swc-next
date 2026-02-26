use std::collections::BTreeMap;

use convert_case::{Case, Casing};
use phf::{Set as PhfSet, phf_set};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::schema::{self, AstEnum, AstStruct, AstType, Schema};

/// Maximum bytes for inline storage
pub const INLINE_DATA_U32_SIZE: usize = std::mem::size_of::<u32>(); // NodeData.inline_data (u32)
pub const INLINE_DATA_U24_SIZE: usize = std::mem::size_of::<[u8; 3]>(); // AstNode.inline_data (U24)
pub const INLINE_TOTAL_SIZE: usize = INLINE_DATA_U32_SIZE + INLINE_DATA_U24_SIZE; // 7 bytes

/// Inline storage mode for a struct
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InlineStorageMode {
    /// Total size ≤4 bytes: all fields stored in NodeData.inline_data (u32)
    FourBytes,
    /// Total size >4 and ≤7 bytes: use NodeData.inline_data + AstNode.inline_data
    Full,
    /// Total size >7 bytes: some fields in AstNode.inline_data (U24), others in ExtraData
    Partial,
}

/// Get the byte size for a field type, or None if not inlinable
pub fn field_byte_size(ty: &AstType, schema: &Schema) -> Option<usize> {
    match ty {
        // NodeId types are 4 bytes (u32)
        AstType::Struct(_) => Some(4),
        AstType::Enum(_) => Some(8),
        // Vec requires SubRange (8 bytes) - too big for inline
        AstType::Vec(_) => None,
        // Option<NodeId> is 4 bytes (OptionalNodeId)
        AstType::Option(ast_option) => {
            let inner_ty = &schema.types[ast_option.inner_type_id];
            match inner_ty {
                // Option<Vec<T>> needs OptionalSubRange - too big
                AstType::Vec(_) => None,
                // Option<Struct> is OptionalNodeId (4 bytes)
                AstType::Struct(_) => Some(4),
                // Option<Enum> is 8 bytes
                AstType::Enum(_) => Some(8),
                // Option<Primitive> not supported for inline storage
                // (would need proper niche optimization handling)
                _ => None,
            }
        }
        AstType::Primitive(prim) => {
            // Check built-in Rust primitives first
            match prim.name {
                "bool" => Some(1),
                "u8" | "i8" => Some(1),
                "u16" | "i16" => Some(2),
                "u32" | "i32" => Some(4),
                "BigIntId" => Some(4),
                "Utf8Ref" | "Wtf8Ref" | "OptionalUtf8Ref" | "OptionalWtf8Ref" => Some(8),
                // For other types, check if they have #[repr(uN)] in schema
                name => schema.repr_sizes.get(name).copied(),
            }
        }
    }
}

/// Layout information for inline storage
#[derive(Debug, Clone)]
pub struct InlineLayout {
    /// Storage mode: FourBytes or Full
    pub mode: InlineStorageMode,
    /// Fields with their byte offsets within the 7-byte inline space
    /// Offset 0-3: NodeData.inline_data (u32)
    /// Offset 4-6: AstNode.inline_data ([u8; 3])
    pub fields: Vec<(usize, usize, usize)>, // (field_index, byte_offset, byte_size)
}

/// Calculate the inline layout for a struct
/// Returns None if the struct cannot be inlined
///
/// This function uses an optimized bin-packing strategy to minimize fields
/// spanning across the u32 (bytes 0-3) and u24 (bytes 4-6) boundary.
pub fn calculate_inline_layout(ast: &AstStruct, schema: &Schema) -> Option<InlineLayout> {
    // Collect all fields with their sizes
    let mut field_info: Vec<(usize, usize)> = Vec::new(); // (field_index, size)
    let mut total_size = 0usize;

    for (index, field) in ast.fields.iter().enumerate() {
        let field_ty = &schema.types[field.type_id];
        let size = field_byte_size(field_ty, schema)?;
        field_info.push((index, size));
        total_size += size;
    }

    if total_size > INLINE_TOTAL_SIZE {
        // Try partial inlining: pack small fields into u24 (bytes 4-6)
        // Fields that don't fit will go to ExtraData

        // Re-use sorted fields logic but only target u24
        let mut sorted_fields = field_info.clone();
        sorted_fields.sort_by(|a, b| b.1.cmp(&a.1));

        let mut u24_fields: Vec<(usize, usize)> = Vec::new();
        let mut u24_used = 0usize;

        for (field_index, size) in sorted_fields {
            if u24_used + size <= INLINE_DATA_U24_SIZE {
                u24_fields.push((field_index, size));
                u24_used += size;
            }
        }

        if u24_fields.is_empty() {
            return None; // No fields fit in u24, fall back to full ExtraData
        }

        // Layout for Partial mode
        let mut fields: Vec<(usize, usize, usize)> = Vec::new(); // (field_index, offset, size)
        let mut current_offset = INLINE_DATA_U32_SIZE; // Start at offset 4

        for (field_index, size) in &u24_fields {
            fields.push((*field_index, current_offset, *size));
            current_offset += size;
        }

        // Sort fields back by index
        fields.sort_by_key(|(field_index, _, _)| *field_index);

        return Some(InlineLayout {
            mode: InlineStorageMode::Partial,
            fields,
        });
    }

    // Sort fields by size (descending) for better packing, but keep original indices
    let mut sorted_fields = field_info.clone();
    sorted_fields.sort_by(|a, b| b.1.cmp(&a.1));

    // Try to pack fields optimally into u32 region (4 bytes) and u24 region (3 bytes)
    let mut u32_fields: Vec<(usize, usize)> = Vec::new(); // (field_index, size)
    let mut u24_fields: Vec<(usize, usize)> = Vec::new(); // (field_index, size)
    let mut u32_used = 0usize;
    let mut u24_used = 0usize;

    // Greedy bin-packing: try to fill u32 region first, then u24 region
    for (field_index, size) in sorted_fields {
        if u32_used + size <= INLINE_DATA_U32_SIZE {
            // Fits in u32 region
            u32_fields.push((field_index, size));
            u32_used += size;
        } else if u24_used + size <= INLINE_DATA_U24_SIZE {
            // Fits in u24 region
            u24_fields.push((field_index, size));
            u24_used += size;
        } else {
            // Try to fit in u32 region if there's space
            if u32_used + size <= INLINE_DATA_U32_SIZE {
                u32_fields.push((field_index, size));
                u32_used += size;
            } else {
                // This shouldn't happen if total_size <= INLINE_TOTAL_SIZE
                return None;
            }
        }
    }

    // Assign offsets: u32 fields get offsets 0-3, u24 fields get offsets 4-6
    let mut fields: Vec<(usize, usize, usize)> = Vec::new(); // (field_index, offset, size)

    let mut current_offset = 0usize;
    for (field_index, size) in &u32_fields {
        fields.push((*field_index, current_offset, *size));
        current_offset += size;
    }

    // Start u24 region at offset 4
    current_offset = INLINE_DATA_U32_SIZE;
    for (field_index, size) in &u24_fields {
        fields.push((*field_index, current_offset, *size));
        current_offset += size;
    }

    // Sort fields back by original field index for consistent ordering
    fields.sort_by_key(|(field_index, _, _)| *field_index);

    let mode = if total_size <= INLINE_DATA_U32_SIZE {
        InlineStorageMode::FourBytes
    } else {
        InlineStorageMode::Full
    };

    Some(InlineLayout { mode, fields })
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

pub fn map_field_type_to_extra_field(ast: &AstType, schema: &Schema) -> String {
    match ast {
        AstType::Struct(_) => "node".to_owned(),
        AstType::Enum(ast_enum) => {
            if ast_enum.name == "AssignTarget" || ast_enum.name == "ModuleItem" {
                "node".to_owned()
            } else {
                ast_enum.name.to_case(Case::Snake)
            }
        }
        AstType::Vec(_) => "sub_range".to_owned(),
        AstType::Option(ast_option) => {
            let inner_ty = &schema.types[ast_option.inner_type_id];
            match inner_ty {
                AstType::Vec(_) => "optional_sub_range".to_owned(),
                AstType::Enum(ast_enum) => {
                    if ast_enum.name == "AssignTarget" || ast_enum.name == "ModuleItem" {
                        "optional_node".to_owned()
                    } else {
                        format!("optional_{}", ast_enum.name.to_case(Case::Snake))
                    }
                }
                _ => "optional_node".to_owned(),
            }
        }
        AstType::Primitive(ast_primitive) => {
            let name = match ast_primitive.name {
                "Span" => "span",
                "Utf8Ref" => "utf8",
                "Wtf8Ref" => "wtf8",
                "OptionalUtf8Ref" => "optional_utf8",
                "OptionalWtf8Ref" => "optional_wtf8",
                "BigIntId" => "bigint",
                "bool" => "bool",
                "f64" => "number",
                _ => "other",
            };
            name.to_owned()
        }
    }
}

/// Generate expression to convert a field value to u32 (for inline storage)
pub fn generate_field_to_u32(
    field_ty: &AstType,
    field_name: &syn::Ident,
    schema: &Schema,
) -> TokenStream {
    match field_ty {
        AstType::Struct(_) | AstType::Enum(_) => {
            quote!(#field_name.node_id().index() as u32)
        }
        AstType::Option(_) => {
            quote!(crate::OptionalNodeId::from(#field_name.map(|n| n.node_id())).into_raw())
        }
        AstType::Primitive(prim) => match prim.name {
            "u32" => quote!(#field_name),
            "i32" => quote!(#field_name as u32),
            "BigIntId" => quote!(#field_name.index() as u32),
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

/// Generate expression to convert u32 to a field type (for inline storage)
pub fn generate_u32_to_field(field_ty: &AstType, schema: &Schema) -> TokenStream {
    match field_ty {
        AstType::Struct(_) | AstType::Enum(_) => {
            let field_inner_ty = field_ty.repr_ident(schema);
            quote! {
                unsafe { #field_inner_ty::from_node_id_unchecked(crate::NodeId::from_raw_unchecked(raw), ast) }
            }
        }
        AstType::Option(ast_option) => {
            let inner_ty = &schema.types[ast_option.inner_type_id];
            let field_inner_ident = inner_ty.repr_ident(schema);
            quote! {
                let opt = crate::OptionalNodeId::from_raw(raw);
                opt.map(|id| unsafe { #field_inner_ident::from_node_id_unchecked(id, ast) })
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
            "BigIntId" => quote! { crate::BigIntId::from_raw_unchecked(raw) },
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

pub struct FlattenElement {
    /// Final nested struct ident, corresponding to NodeKind::XXX
    pub kind: Ident,
    /// From the first enum::variant to the last enum::variant
    pub path: Vec<TokenStream>,
}

pub fn flat_enum_type(ast: &AstEnum, schema: &Schema) -> BTreeMap<String, FlattenElement> {
    let mut flatten = BTreeMap::new();
    let mut worklist = Vec::new();

    let enum_ident = format_ident!("{}", ast.name);
    for variant in ast.variants.iter() {
        let variant_ident = format_ident!("{}", variant.name);
        let path = quote!( #enum_ident::#variant_ident );
        worklist.push((variant, vec![path]));
    }

    while let Some((variant, path)) = worklist.pop() {
        let Some(variant_inner_ty_id) = variant.type_id else {
            continue;
        };

        let variant_inner_ty = &schema.types[variant_inner_ty_id];
        match variant_inner_ty {
            schema::AstType::Struct(ast_struct) => {
                let kind = format_ident!("{}", ast_struct.name);
                flatten.insert(ast_struct.name.clone(), FlattenElement { kind, path });
            }
            schema::AstType::Enum(ast_enum) => {
                let nested_enum_ident = format_ident!("{}", ast_enum.name);
                for nested_variant in ast_enum.variants.iter() {
                    let nested_variant_ident = format_ident!("{}", nested_variant.name);
                    let mut path = path.clone();
                    path.push(quote! ( #nested_enum_ident::#nested_variant_ident ));
                    worklist.push((nested_variant, path));
                }
            }
            _ => unreachable!(),
        }
    }

    flatten
}
