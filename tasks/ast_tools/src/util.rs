use phf::{Set as PhfSet, phf_set};
use quote::format_ident;
use syn::Ident;

use crate::schema::{AstStructField, AstType, Schema, TypeId};

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

pub fn simple_enum_flag_width(name: &str) -> Option<usize> {
    Some(match name {
        "ImportPhase" => 2,
        "VarDeclKind" => 2,
        "UnaryOp" => 3,
        "UpdateOp" => 1,
        "AssignOp" => 5,
        "BinaryOp" => 5,
        "MetaPropKind" => 1,
        "MethodKind" => 2,
        _ => return None,
    })
}

pub fn should_skip_field(field: &AstStructField, schema: &Schema) -> bool {
    match &schema.types[field.type_id] {
        AstType::Primitive(ast) => matches!(ast.name, "ScopeId" | "SymbolId"),
        AstType::Option(ast) => {
            matches!(&schema.types[ast.inner_type_id], AstType::Primitive(inner) if matches!(inner.name, "ScopeId" | "SymbolId"))
        }
        _ => false,
    }
}

pub fn is_span_field(field: &AstStructField, schema: &Schema) -> bool {
    matches!(&schema.types[field.type_id], AstType::Primitive(ast) if ast.name == "Span")
}

pub fn type_has_lifetime(type_id: TypeId, schema: &Schema) -> bool {
    fn field_type_needs_lifetime(type_id: TypeId, schema: &Schema) -> bool {
        match &schema.types[type_id] {
            AstType::Struct(_) => true,
            AstType::Enum(_) => type_has_lifetime(type_id, schema),
            AstType::Box(_) => true,
            AstType::Vec(_) => true,
            AstType::Option(ast) => field_type_needs_lifetime(ast.inner_type_id, schema),
            AstType::Primitive(ast) => primitive_type_needs_lifetime(ast.name),
        }
    }

    fn primitive_type_needs_lifetime(name: &str) -> bool {
        matches!(name, "Atom" | "Wtf8Atom")
    }

    match &schema.types[type_id] {
        AstType::Struct(ast) => ast
            .fields
            .iter()
            .any(|field| field_type_needs_lifetime(field.type_id, schema)),
        AstType::Enum(ast) => ast.variants.iter().any(|variant| variant.type_id.is_some()),
        AstType::Box(_) => true,
        AstType::Vec(_) => true,
        AstType::Option(ast) => field_type_needs_lifetime(ast.inner_type_id, schema),
        AstType::Primitive(ast) => primitive_type_needs_lifetime(ast.name),
    }
}
