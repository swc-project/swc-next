use std::{
    collections::{HashMap, HashSet},
    fs,
};

use indexmap::IndexSet;
use oxc_index::IndexVec;
use quote::ToTokens;
use syn::{
    Attribute, Field, GenericArgument, Ident, Item, ItemEnum, ItemStruct, Meta, PathArguments,
    Type, Variant, punctuated::Punctuated, token::Comma,
};

use crate::schema::{
    AstAttrs, AstEnum, AstEnumVariant, AstFieldAttrs, AstOption, AstPrimitive, AstStruct,
    AstStructField, AstSubRange, AstType, FieldLocation, Layout, LayoutMode, Schema, TypeId,
};

const PAYLOAD_SIZE: usize = 16;
const EXTRA_START_SIZE: usize = 4;
const EXTRA_START_OFFSET: usize = PAYLOAD_SIZE - EXTRA_START_SIZE;

struct Parser {
    /// Mapping all type names to their type ids
    type_names: IndexSet<String>,
    top_level_type_count: usize,
    /// Types exclude top level structs and enums
    extra_types: IndexVec<TypeId, AstType>,
    repr_sizes: HashMap<String, usize>,
}

pub fn parse_files(file_paths: &[&str]) -> Schema {
    struct PrototypeStruct {
        type_id: TypeId,
        item: ItemStruct,
        attrs: AstAttrs,
    }

    struct PrototypeEnum {
        type_id: TypeId,
        item: ItemEnum,
        attrs: AstAttrs,
    }

    enum PrototypeStructOrEnum {
        Struct(PrototypeStruct),
        Enum(PrototypeEnum),
    }

    // Collect declared enums and structs ahead of time and create their types
    let mut type_names = IndexSet::new();
    let mut prototypes = Vec::new();
    let mut repr_sizes = HashMap::new();
    let mut scalar_enums = Vec::new();

    for source in file_paths {
        let content =
            fs::read_to_string(source).unwrap_or_else(|_| panic!("Cannot not find {source}"));
        let file = syn::parse_file(&content)
            .unwrap_or_else(|error| panic!("cannot parse {source}: {error}"));
        for item in file.items {
            match item {
                Item::Struct(item) => {
                    // Filter structs with #[ast]
                    let Some(attrs) = parse_attrs(&item.attrs) else {
                        continue;
                    };

                    let name = item.ident.to_string();
                    let type_id = TypeId::from_usize(type_names.len());
                    type_names.insert(name);
                    prototypes.push(PrototypeStructOrEnum::Struct(PrototypeStruct {
                        type_id,
                        item,
                        attrs,
                    }));
                }
                Item::Enum(item) => {
                    if let Some(enumeration) = crate::estree::scalar(&item) {
                        scalar_enums.push(enumeration);
                    }
                    // Collect #[repr(uN)] sizes for all enums (not just #[ast] enums)
                    if let Some(size) = parse_repr_size(&item.attrs) {
                        repr_sizes.insert(item.ident.to_string(), size);
                    }

                    // Filter enums with #[ast]
                    let Some(attrs) = parse_attrs(&item.attrs) else {
                        continue;
                    };

                    let name = item.ident.to_string();
                    let type_id = TypeId::from_usize(type_names.len());
                    type_names.insert(name);
                    prototypes.push(PrototypeStructOrEnum::Enum(PrototypeEnum {
                        type_id,
                        item,
                        attrs,
                    }));
                }
                _ => continue,
            }
        }
    }

    let mut parser = Parser {
        top_level_type_count: type_names.len(),
        type_names,
        extra_types: IndexVec::default(),
        repr_sizes,
    };

    // Collect inner types like struct fields and enum variants
    let mut types = IndexVec::new();
    for prototype in prototypes {
        match prototype {
            PrototypeStructOrEnum::Struct(prototype_struct) => {
                types.push(parser.parse_struct(
                    prototype_struct.type_id,
                    prototype_struct.attrs,
                    prototype_struct.item,
                ));
            }
            PrototypeStructOrEnum::Enum(prototype_enum) => {
                types.push(parser.parse_enum(
                    prototype_enum.type_id,
                    prototype_enum.attrs,
                    prototype_enum.item,
                ));
            }
        }
    }

    types.extend(parser.extra_types);
    let schema = Schema {
        scalar_enums,
        types,
        repr_sizes: parser.repr_sizes,
    };
    validate_invalidations(&schema);
    schema
}

impl Parser {
    fn create_new_type(&mut self, mut item: AstType) -> TypeId {
        let type_id = TypeId::from_usize(self.type_names.len());
        match &mut item {
            AstType::Struct(ast) => ast.type_id = type_id,
            AstType::Enum(ast) => ast.type_id = type_id,
            AstType::SubRange(ast) => ast.type_id = type_id,
            AstType::Option(ast) => ast.type_id = type_id,
            AstType::Primitive(ast) => ast.type_id = type_id,
        }

        let not_contained = self.type_names.insert(item.name().to_string());
        assert!(not_contained, "{} is duplicated", item.name());

        self.extra_types.push(item);
        type_id
    }

    /// Get type id of the simple type which is not wrapped by `Option` or `SubRange`
    fn simple_type_id(&mut self, name: &str) -> TypeId {
        if let Some(type_id) = self.type_names.get_index_of(name) {
            return TypeId::from_usize(type_id);
        }

        let primitive = |name| {
            AstType::Primitive(AstPrimitive {
                type_id: TypeId::DUMMY,
                name,
            })
        };

        let type_def = match name {
            "bool" => primitive("bool"),
            "u8" => primitive("u8"),
            "u16" => primitive("u16"),
            "u32" => primitive("u32"),
            "u64" => primitive("u64"),
            "u128" => primitive("u128"),
            "usize" => primitive("usize"),
            "i8" => primitive("i8"),
            "i16" => primitive("i16"),
            "i32" => primitive("i32"),
            "i64" => primitive("i64"),
            "i128" => primitive("i128"),
            "isize" => primitive("isize"),
            "f32" => primitive("f32"),
            "f64" => primitive("f64"),

            // Custom enum
            "Span" => primitive("Span"),
            "ScopeId" => primitive("ScopeId"),
            "SymbolId" => primitive("SymbolId"),
            "AstInternedUtf8" => primitive("AstInternedUtf8"),
            "AstUtf8" => primitive("AstUtf8"),
            "AstWtf8" => primitive("AstWtf8"),
            "ImportPhase" => primitive("ImportPhase"),
            "SourceType" => primitive("SourceType"),
            "MethodDefinitionKind" => primitive("MethodDefinitionKind"),
            "Accessibility" => primitive("Accessibility"),
            "ClassType" => primitive("ClassType"),
            "BinaryOperator" => primitive("BinaryOperator"),
            "LogicalOperator" => primitive("LogicalOperator"),
            "UnaryOperator" => primitive("UnaryOperator"),
            "UpdateOperator" => primitive("UpdateOperator"),
            "AssignmentOperator" => primitive("AssignmentOperator"),
            "VariableKind" => primitive("VariableKind"),
            "FunctionType" => primitive("FunctionType"),
            "FormalParameterKind" => primitive("FormalParameterKind"),
            "PropertyKind" => primitive("PropertyKind"),
            "NumericLiteralKind" => primitive("NumericLiteralKind"),
            "ImportOrExportKind" => primitive("ImportOrExportKind"),
            "TsTypeOperatorKind" => primitive("TsTypeOperatorKind"),
            "TsMethodSignatureKind" => primitive("TsMethodSignatureKind"),
            "TsModuleDeclarationKind" => primitive("TsModuleDeclarationKind"),
            "TsMappedTypeModifier" => primitive("TsMappedTypeModifier"),
            _ => panic!("Unknown primitive {name}"),
        };
        self.create_new_type(type_def)
    }
}

/// Parse #[repr(uN)] attribute and return the size in bytes
/// Handles whitespace and multiple repr arguments (e.g., `#[repr(C, u8)]`)
fn parse_repr_size(attrs: &[Attribute]) -> Option<usize> {
    for attr in attrs {
        if let Meta::List(meta_list) = &attr.meta
            && meta_list.path.is_ident("repr")
        {
            // Parse as comma-separated identifiers to handle `#[repr(C, u8)]`
            if let Ok(args) =
                meta_list.parse_args_with(Punctuated::<Ident, Comma>::parse_terminated)
            {
                for arg in args {
                    match arg.to_string().as_str() {
                        "u8" => return Some(1),
                        "u16" => return Some(2),
                        "u32" => return Some(4),
                        "u64" => return Some(8),
                        _ => continue,
                    }
                }
            }
        }
    }
    None
}

fn parse_attrs(attrs: &[Attribute]) -> Option<AstAttrs> {
    let mut has_ast_attr = false;
    let mut ast_attrs = HashSet::new();
    let mut generate_derives = HashSet::new();
    if !attrs.iter().any(|attr| attr.path().is_ident("ast")) {
        return None;
    }
    let estree = crate::estree::attrs(attrs);

    // Find AST metadata before the proc macro consumes the original fields.
    for attr in attrs {
        match &attr.meta {
            Meta::Path(path) => {
                if path.is_ident("ast") {
                    has_ast_attr = true;
                }
            }
            Meta::List(meta_list) => {
                if meta_list.path.is_ident("ast") {
                    has_ast_attr = true;
                    let args = meta_list
                        .parse_args_with(Punctuated::<Ident, Comma>::parse_terminated)
                        .expect("Unable to parse `#[ast]`");
                    ast_attrs.extend(args.into_iter().map(|arg| arg.to_string()));
                }
                if meta_list.path.is_ident("generate_derive") {
                    let args = meta_list
                        .parse_args_with(Punctuated::<Ident, Comma>::parse_terminated)
                        .expect("Unable to parse `#[generate_derive]`");
                    generate_derives.extend(args.into_iter().map(|arg| arg.to_string()));
                }
            }
            _ => continue,
        };
    }

    if !has_ast_attr {
        return None;
    }

    Some(AstAttrs {
        ast_attrs,
        generate_derives,
        estree,
    })
}

impl Parser {
    fn parse_struct(&mut self, type_id: TypeId, attrs: AstAttrs, item: ItemStruct) -> AstType {
        let name = item.ident.to_string();
        let fields: Vec<AstStructField> = item
            .fields
            .into_iter()
            .map(|field| self.parse_struct_field(field))
            .collect();
        let layout = self.calculate_layout(&fields);
        AstType::Struct(AstStruct {
            type_id,
            name,
            fields,
            layout,
            _attrs: attrs,
        })
    }

    fn parse_struct_field(&mut self, field: Field) -> AstStructField {
        let name = field.ident.unwrap().to_string();
        let type_id = self
            .parse_type_name(&field.ty)
            .unwrap_or_else(|| panic!("Cannot parse type {}", field.ty.to_token_stream()));
        let attrs = Self::parse_field_attrs(&field.attrs);

        AstStructField {
            type_id,
            name,
            attrs,
        }
    }

    fn parse_enum(&mut self, type_id: TypeId, attrs: AstAttrs, item: ItemEnum) -> AstType {
        let name = item.ident.to_string();
        assert!(
            item.variants
                .iter()
                .all(|variant| variant.discriminant.is_none()),
            "`#[ast(repr)]` enum `{name}` does not support explicit discriminants"
        );
        let variants = item
            .variants
            .into_iter()
            .map(|variant| self.parse_enum_variant(variant))
            .collect();
        AstType::Enum(AstEnum {
            type_id,
            name,
            variants,
            _attrs: attrs,
        })
    }

    fn parse_enum_variant(&mut self, variant: Variant) -> AstEnumVariant {
        let name = variant.ident.to_string();

        let type_id = if variant.fields.is_empty() {
            None
        } else {
            assert!(
                variant.fields.len() == 1,
                "Only variants with single field are supported"
            );
            let field = variant.fields.iter().next().unwrap();
            let type_id = self
                .parse_type_name(&field.ty)
                .unwrap_or_else(|| panic!("Cannot parse type {}", field.ty.to_token_stream()));
            Some(type_id)
        };

        AstEnumVariant {
            type_id,
            name,
            estree: crate::estree::attrs(&variant.attrs),
        }
    }

    fn parse_type_name(&mut self, ty: &Type) -> Option<TypeId> {
        let Type::Path(ty) = ty else {
            return None;
        };

        if ty.qself.is_some() || ty.path.leading_colon.is_some() || ty.path.segments.len() != 1 {
            return None;
        }

        let segment = ty.path.segments.first().unwrap();
        let name = segment.ident.to_string();
        match &segment.arguments {
            PathArguments::None => Some(self.simple_type_id(&name)),
            PathArguments::Parenthesized(_) => None,
            PathArguments::AngleBracketed(angle) => {
                let mut args = angle.args.iter();
                let arg = match args.next().unwrap() {
                    GenericArgument::Lifetime(_) => args.next(),
                    arg => Some(arg),
                };

                if let Some(arg) = arg {
                    self.parse_wrapped_type(&name, arg)
                } else {
                    Some(self.simple_type_id(&name))
                }
            }
        }
    }

    fn parse_wrapped_type(&mut self, wrapper_name: &str, arg: &GenericArgument) -> Option<TypeId> {
        let GenericArgument::Type(ty_arg) = arg else {
            return None;
        };

        let inner_type_id = self.parse_type_name(ty_arg)?;
        let type_id = match wrapper_name {
            "SubRange" => {
                let name = format!(
                    "{}<{}>",
                    wrapper_name,
                    self.type_names[inner_type_id.index()]
                );
                if let Some(type_id) = self.type_names.get_index_of(&name) {
                    return Some(TypeId::from(type_id));
                }

                let ast_type = AstType::SubRange(AstSubRange {
                    type_id: TypeId::DUMMY,
                    name,
                    inner_type_id,
                });
                self.create_new_type(ast_type)
            }
            "Option" => {
                let name = format!(
                    "{}<{}>",
                    wrapper_name,
                    self.type_names[inner_type_id.index()]
                );
                if let Some(type_id) = self.type_names.get_index_of(&name) {
                    return Some(TypeId::from(type_id));
                }

                let ast_type = AstType::Option(AstOption {
                    type_id: TypeId::DUMMY,
                    name,
                    inner_type_id,
                });
                self.create_new_type(ast_type)
            }
            _ => return None,
        };
        Some(type_id)
    }

    fn parse_field_attrs(attrs: &[Attribute]) -> AstFieldAttrs {
        let mut field_attrs = crate::estree::field_attrs(attrs);

        for attr in attrs {
            let Meta::List(meta_list) = &attr.meta else {
                continue;
            };
            if meta_list.path.is_ident("ast") {
                let args = meta_list
                    .parse_args_with(Punctuated::<Meta, Comma>::parse_terminated)
                    .expect("Unable to parse AST field attributes");
                for arg in args {
                    let Meta::List(list) = arg else {
                        panic!("AST field attributes must use list syntax");
                    };
                    if !list.path.is_ident("invalidates") {
                        panic!(
                            "Unknown AST field attribute `{}`",
                            list.path.to_token_stream()
                        );
                    }
                    let targets = list
                        .parse_args_with(Punctuated::<Ident, Comma>::parse_terminated)
                        .expect("Unable to parse `invalidates(...)`");
                    field_attrs
                        .invalidates
                        .extend(targets.into_iter().map(|target| target.to_string()));
                }
            }
        }

        field_attrs
    }
}

impl Parser {
    fn extra_type(&self, type_id: TypeId) -> Option<&AstType> {
        (type_id.index() >= self.top_level_type_count).then(|| {
            &self.extra_types[TypeId::from_usize(type_id.index() - self.top_level_type_count)]
        })
    }

    fn field_byte_size(&self, type_id: TypeId) -> Option<usize> {
        match self.extra_type(type_id) {
            // Top-level structs and enums both occupy four bytes in the payload.
            None | Some(AstType::Struct(_) | AstType::Enum(_)) => Some(4),
            Some(AstType::SubRange(_)) => Some(8),
            Some(AstType::Option(option)) => match self.extra_type(option.inner_type_id) {
                None | Some(AstType::Struct(_) | AstType::Enum(_)) => Some(4),
                Some(AstType::SubRange(_)) => Some(8),
                Some(AstType::Primitive(primitive)) => match primitive.name {
                    "Span" | "AstInternedUtf8" | "AstUtf8" => Some(8),
                    "ScopeId" | "SymbolId" => Some(4),
                    _ => None,
                },
                Some(AstType::Option(_)) => None,
            },
            Some(AstType::Primitive(primitive)) => match primitive.name {
                "bool" | "u8" | "i8" => Some(1),
                "u16" | "i16" => Some(2),
                "u32" | "i32" => Some(4),
                "u64" | "f64" | "Span" | "AstInternedUtf8" | "AstUtf8" | "AstWtf8" => Some(8),
                name => self.repr_sizes.get(name).copied(),
            },
        }
    }

    fn calculate_layout(&self, fields: &[AstStructField]) -> Layout {
        let mut eligible = Vec::new();
        let mut total_size = 0;
        let mut all_fields_inlinable = true;
        for (index, field) in fields.iter().enumerate() {
            if let Some(size) = self.field_byte_size(field.type_id) {
                assert!(matches!(size, 1 | 2 | 4 | 8));
                eligible.push((index, size));
                total_size += size;
            } else {
                all_fields_inlinable = false;
            }
        }

        let (mode, payload_fields) = if all_fields_inlinable && total_size <= PAYLOAD_SIZE {
            eligible.sort_by_key(|&(_, size)| std::cmp::Reverse(size));
            (LayoutMode::Full, Self::place_fields(eligible, PAYLOAD_SIZE))
        } else {
            // Select small fields first to maximize the number stored in the
            // low 12 bytes, then place the selected fields largest first so
            // each field starts at an offset aligned to its size.
            eligible.sort_by_key(|&(index, size)| (size, index));
            let mut selected = Self::place_fields(eligible, EXTRA_START_OFFSET)
                .into_iter()
                .map(|(index, _, size)| (index, size))
                .collect::<Vec<_>>();
            assert!(!selected.is_empty(), "partial layout needs an inline field");
            selected.sort_by_key(|&(index, size)| (std::cmp::Reverse(size), index));
            (
                LayoutMode::Partial,
                Self::place_fields(selected, EXTRA_START_OFFSET),
            )
        };

        let mut locations = vec![None; fields.len()];
        for (index, offset, size) in payload_fields {
            locations[index] = Some(FieldLocation::Payload { offset, size });
        }
        let mut extra_count = 0;
        let fields = locations
            .into_iter()
            .map(|location| {
                location.unwrap_or_else(|| {
                    let index = extra_count;
                    extra_count += 1;
                    FieldLocation::Extra { index }
                })
            })
            .collect();
        debug_assert_eq!(mode == LayoutMode::Full, extra_count == 0);

        Layout {
            mode,
            fields,
            extra_count,
            extra_start_offset: (extra_count != 0).then_some(EXTRA_START_OFFSET),
        }
    }

    fn place_fields(fields: Vec<(usize, usize)>, capacity: usize) -> Vec<(usize, usize, usize)> {
        let mut placed = Vec::new();
        let mut used = 0;
        for (index, size) in fields {
            if used + size <= capacity {
                placed.push((index, used, size));
                used += size;
            }
        }
        placed
    }
}

fn validate_invalidations(schema: &Schema) {
    for ty in &schema.types {
        let AstType::Struct(ast) = ty else {
            continue;
        };
        let fields = ast
            .fields
            .iter()
            .enumerate()
            .map(|(index, field)| (field.name.as_str(), index))
            .collect::<HashMap<_, _>>();

        for field in &ast.fields {
            let mut targets = HashSet::new();
            for target_name in &field.attrs.invalidates {
                assert!(
                    targets.insert(target_name),
                    "`{}.{}` invalidates `{target_name}` more than once",
                    ast.name,
                    field.name
                );
                let Some(&target_index) = fields.get(target_name.as_str()) else {
                    panic!(
                        "`{}.{}` invalidates unknown field `{target_name}`",
                        ast.name, field.name
                    );
                };
                assert_ne!(
                    field.name, *target_name,
                    "`{}.{}` cannot invalidate itself",
                    ast.name, field.name
                );
                let target = &ast.fields[target_index];
                assert!(
                    matches!(schema.types[target.type_id], AstType::Option(_)),
                    "`{}.{}` invalidates `{target_name}`, which must be an Option field",
                    ast.name,
                    field.name
                );
            }
        }

        let mut states = vec![0_u8; ast.fields.len()];
        for field_index in 0..ast.fields.len() {
            validate_invalidation_cycles(ast, &fields, field_index, &mut states);
        }
    }
}

fn validate_invalidation_cycles(
    ast: &AstStruct,
    fields: &HashMap<&str, usize>,
    field_index: usize,
    states: &mut [u8],
) {
    match states[field_index] {
        1 => panic!(
            "invalidation dependency cycle involving `{}.{}`",
            ast.name, ast.fields[field_index].name
        ),
        2 => return,
        _ => {}
    }

    states[field_index] = 1;
    for target_name in &ast.fields[field_index].attrs.invalidates {
        let target_index = fields[target_name.as_str()];
        validate_invalidation_cycles(ast, fields, target_index, states);
    }
    states[field_index] = 2;
}
