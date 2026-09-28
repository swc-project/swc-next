use std::collections::{HashMap, HashSet};

use oxc_index::IndexVec;
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};

oxc_index::define_index_type! {
    pub struct TypeId = u32;
}

impl TypeId {
    pub const DUMMY: Self = Self::from_raw_unchecked(0);
}

pub struct Schema {
    pub types: IndexVec<TypeId, AstType>,
    /// Mapping of enum names to their #[repr(uN)] sizes in bytes
    /// e.g., "UnaryOperator" -> 1 for #[repr(u8)]
    pub repr_sizes: HashMap<String, usize>,
}

#[derive(Debug)]
pub enum AstType {
    Struct(AstStruct),
    Enum(AstEnum),
    SubRange(AstSubRange),
    Option(AstOption),
    Primitive(AstPrimitive),
}

impl AstType {
    pub fn name(&self) -> &str {
        match self {
            AstType::Struct(ast) => &ast.name,
            AstType::Enum(ast) => &ast.name,
            AstType::SubRange(ast) => &ast.name,
            AstType::Option(ast) => &ast.name,
            AstType::Primitive(ast) => ast.name,
        }
    }

    pub fn repr_ident<'a>(&'a self, schema: &'a Schema) -> TokenStream {
        match self {
            AstType::Struct(ast) => format_ident!("{}", &ast.name).into_token_stream(),
            AstType::Enum(ast) => format_ident!("{}", &ast.name).into_token_stream(),
            AstType::SubRange(ast) => {
                let name = format_ident!("SubRange");
                let inner_name = schema.types[ast.inner_type_id].repr_ident(schema);
                quote!( #name<#inner_name> )
            }
            AstType::Option(ast) => {
                let name = format_ident!("Option");
                let inner_name = schema.types[ast.inner_type_id].repr_ident(schema);
                quote!( #name<#inner_name> )
            }
            AstType::Primitive(ast) => format_ident!("{}", &ast.name).into_token_stream(),
        }
    }
}

#[derive(Debug)]
pub struct AstStruct {
    pub type_id: TypeId,
    pub name: String,
    pub fields: Vec<AstStructField>,
    pub layout: Layout,
    pub _attrs: AstAttrs,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LayoutMode {
    #[default]
    Full,
    Partial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldLocation {
    Payload { offset: usize, size: usize },
    Extra { index: usize },
}

#[derive(Debug, Default)]
pub struct Layout {
    pub mode: LayoutMode,
    /// One location per struct field, in schema order.
    pub fields: Vec<FieldLocation>,
    pub extra_count: usize,
    pub extra_start_offset: Option<usize>,
}

impl Layout {
    pub fn payload_fields(&self) -> impl Iterator<Item = (usize, usize, usize)> + '_ {
        self.fields
            .iter()
            .enumerate()
            .filter_map(|(field_index, location)| match location {
                FieldLocation::Payload { offset, size } => Some((field_index, *offset, *size)),
                FieldLocation::Extra { .. } => None,
            })
    }

    pub fn extra_fields(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        self.fields
            .iter()
            .enumerate()
            .filter_map(|(field_index, location)| match location {
                FieldLocation::Payload { .. } => None,
                FieldLocation::Extra { index } => Some((field_index, *index)),
            })
    }

    pub fn extra_start_offset(&self) -> Option<usize> {
        self.extra_start_offset
    }
}

#[derive(Debug)]
pub struct AstStructField {
    pub type_id: TypeId,
    pub name: String,
    pub attrs: AstFieldAttrs,
}

#[derive(Debug)]
pub struct AstEnum {
    pub type_id: TypeId,
    pub name: String,
    pub variants: Vec<AstEnumVariant>,
    pub _attrs: AstAttrs,
}

#[derive(Debug)]
pub struct AstEnumVariant {
    pub type_id: Option<TypeId>,
    pub name: String,
}

#[derive(Debug)]
pub struct AstSubRange {
    pub type_id: TypeId,
    pub name: String,
    pub inner_type_id: TypeId,
}

#[derive(Debug)]
pub struct AstOption {
    pub type_id: TypeId,
    pub name: String,
    pub inner_type_id: TypeId,
}

#[derive(Debug)]
#[allow(unused)]
pub struct AstNode {
    pub type_id: TypeId,
    pub name: String,
}

#[derive(Debug)]
pub struct AstPrimitive {
    pub type_id: TypeId,
    pub name: &'static str,
}

#[derive(Debug)]
#[allow(unused)]
pub struct AstAttrs {
    pub ast_attrs: HashSet<String>,
    pub generate_derives: HashSet<String>,
}

#[derive(Debug, Default)]
pub struct AstFieldAttrs {
    pub estree_skip: bool,
    pub invalidates: Vec<String>,
}
