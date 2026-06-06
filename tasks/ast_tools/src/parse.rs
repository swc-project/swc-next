use std::fs;

use indexmap::IndexSet;
use oxc_index::IndexVec;
use quote::ToTokens;
use syn::{
    Attribute, Field, GenericArgument, Ident, Item, ItemEnum, ItemStruct, Meta, PathArguments,
    Type, Variant,
};

use crate::schema::{
    AstBox, AstEnum, AstEnumVariant, AstOption, AstPrimitive, AstStruct, AstStructField, AstType,
    AstVec, Schema, SpanKind, TypeId,
};

struct Parser {
    /// Mapping all type names to their type ids
    type_names: IndexSet<String>,
    top_level_type_count: usize,
    /// Types exclude top level structs and enums
    extra_types: IndexVec<TypeId, AstType>,
}

pub fn parse_files(file_paths: &[&str]) -> Schema {
    struct PrototypeStruct {
        type_id: TypeId,
        item: ItemStruct,
    }

    struct PrototypeEnum {
        type_id: TypeId,
        item: ItemEnum,
    }

    enum PrototypeStructOrEnum {
        Struct(PrototypeStruct),
        Enum(PrototypeEnum),
    }

    // Collect declared enums and structs ahead of time and create their types
    let mut type_names = IndexSet::new();
    let mut prototypes = Vec::new();

    for source in file_paths {
        let content =
            fs::read_to_string(source).unwrap_or_else(|_| panic!("Cannot not find {source}"));
        let file = syn::parse_file(&content).unwrap();
        for item in file.items {
            match item {
                Item::Struct(item) => {
                    // Filter structs with #[ast]
                    if !has_ast_attr(&item.attrs) {
                        continue;
                    }

                    let name = item.ident.to_string();
                    let type_id = TypeId::from_usize(type_names.len());
                    type_names.insert(name);
                    prototypes.push(PrototypeStructOrEnum::Struct(PrototypeStruct {
                        type_id,
                        item,
                    }));
                }
                Item::Enum(item) => {
                    // Filter enums with #[ast]
                    if !has_ast_attr(&item.attrs) {
                        continue;
                    }

                    let name = item.ident.to_string();
                    let type_id = TypeId::from_usize(type_names.len());
                    type_names.insert(name);
                    prototypes.push(PrototypeStructOrEnum::Enum(PrototypeEnum { type_id, item }));
                }
                _ => continue,
            }
        }
    }

    let mut parser = Parser {
        top_level_type_count: type_names.len(),
        type_names,
        extra_types: IndexVec::default(),
    };

    // Collect inner types like struct fields and enum variants
    let mut types = IndexVec::new();
    for prototype in prototypes {
        match prototype {
            PrototypeStructOrEnum::Struct(prototype_struct) => {
                types.push(parser.parse_struct(prototype_struct.type_id, prototype_struct.item));
            }
            PrototypeStructOrEnum::Enum(prototype_enum) => {
                types.push(parser.parse_enum(prototype_enum.type_id, prototype_enum.item));
            }
        }
    }

    types.extend(parser.extra_types);
    Schema { types }
}

impl Parser {
    fn create_new_type(&mut self, mut item: AstType) -> TypeId {
        let type_id = TypeId::from_usize(self.type_names.len());
        match &mut item {
            AstType::Struct(ast) => ast.type_id = type_id,
            AstType::Enum(ast) => ast.type_id = type_id,
            AstType::Box(ast) => ast.type_id = type_id,
            AstType::Vec(ast) => ast.type_id = type_id,
            AstType::Option(ast) => ast.type_id = type_id,
            AstType::Primitive(ast) => ast.type_id = type_id,
        }

        let not_contained = self.type_names.insert(item.name().to_string());
        assert!(not_contained, "{} is duplicated", item.name());

        self.extra_types.push(item);
        type_id
    }

    /// Get type id of the simple type which is not wrapped by `Option`, `Box` or `Vec`
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
            "AtomRef" => primitive("AtomRef"),
            "Atom" => primitive("Atom"),
            "Wtf8Atom" => primitive("Wtf8Atom"),
            "swc_experimental_num_bigint::BigInt" => {
                primitive("swc_experimental_num_bigint::BigInt")
            }
            "ImportPhase" => primitive("ImportPhase"),
            "VarDeclKind" => primitive("VarDeclKind"),
            "UnaryOp" => primitive("UnaryOp"),
            "BinaryOp" => primitive("BinaryOp"),
            "AssignOp" => primitive("AssignOp"),
            "UpdateOp" => primitive("UpdateOp"),
            "MetaPropKind" => primitive("MetaPropKind"),
            "MethodKind" => primitive("MethodKind"),
            _ => panic!("Unknown primitive {name}"),
        };
        self.create_new_type(type_def)
    }
}

fn has_ast_attr(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| match &attr.meta {
        Meta::Path(path) => path.is_ident("ast"),
        Meta::List(meta_list) => meta_list.path.is_ident("ast"),
        _ => false,
    })
}

impl Parser {
    fn parse_struct(&mut self, type_id: TypeId, item: ItemStruct) -> AstType {
        let name = item.ident.to_string();
        let fields = item
            .fields
            .into_iter()
            .map(|field| self.parse_struct_field(field))
            .collect();
        AstType::Struct(AstStruct {
            type_id,
            name,
            fields,
        })
    }

    fn parse_struct_field(&mut self, field: Field) -> AstStructField {
        let name = field.ident.unwrap().to_string();
        let span_kind = parse_span_attr(&field.attrs);
        let type_id = self
            .parse_type_name(&field.ty)
            .unwrap_or_else(|| panic!("Cannot parse type {}", field.ty.to_token_stream()));
        AstStructField {
            type_id,
            name,
            span_kind,
        }
    }

    fn parse_enum(&mut self, type_id: TypeId, item: ItemEnum) -> AstType {
        let name = item.ident.to_string();
        let variants = item
            .variants
            .into_iter()
            .map(|variant| self.parse_enum_variant(variant))
            .collect();
        AstType::Enum(AstEnum {
            type_id,
            name,
            variants,
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

        AstEnumVariant { type_id, name }
    }

    fn parse_type_name(&mut self, ty: &Type) -> Option<TypeId> {
        let Type::Path(ty) = ty else {
            return None;
        };

        if ty.qself.is_some() || ty.path.leading_colon.is_some() {
            return None;
        }

        let segment = ty.path.segments.last().unwrap();
        let name = segment.ident.to_string();
        match &segment.arguments {
            PathArguments::None => {
                let name = if ty.path.segments.len() == 1 {
                    name
                } else {
                    ty.path
                        .segments
                        .iter()
                        .map(|segment| segment.ident.to_string())
                        .collect::<Vec<_>>()
                        .join("::")
                };
                Some(self.simple_type_id(&name))
            }
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
                    let name = if ty.path.segments.len() == 1 {
                        name
                    } else {
                        ty.path
                            .segments
                            .iter()
                            .map(|segment| segment.ident.to_string())
                            .collect::<Vec<_>>()
                            .join("::")
                    };
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
            "Box" => {
                if inner_type_id.index() < self.top_level_type_count {
                    return Some(inner_type_id);
                }

                let name = format!(
                    "{}<{}>",
                    wrapper_name,
                    self.type_names[inner_type_id.index()]
                );
                if let Some(type_id) = self.type_names.get_index_of(&name) {
                    return Some(TypeId::from(type_id));
                }

                let ast_type = AstType::Box(AstBox {
                    type_id: TypeId::DUMMY,
                    name,
                    inner_type_id,
                });
                self.create_new_type(ast_type)
            }
            "Vec" => {
                let name = format!(
                    "{}<{}>",
                    wrapper_name,
                    self.type_names[inner_type_id.index()]
                );
                if let Some(type_id) = self.type_names.get_index_of(&name) {
                    return Some(TypeId::from(type_id));
                }

                let ast_type = AstType::Vec(AstVec {
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
            "Cell" => inner_type_id,
            _ => return None,
        };
        Some(type_id)
    }
}

fn parse_span_attr(attrs: &[Attribute]) -> Option<SpanKind> {
    attrs.iter().find_map(|attr| {
        if !attr.path().is_ident("span") {
            return None;
        }

        match &attr.meta {
            Meta::Path(_) => Some(SpanKind::Full),
            Meta::List(_) => match attr.parse_args::<Ident>() {
                Ok(ident) if ident == "lo" => Some(SpanKind::Lo),
                Ok(ident) if ident == "hi" => Some(SpanKind::Hi),
                Ok(ident) => panic!("Unknown span attribute argument {ident}"),
                Err(err) => panic!("Cannot parse span attribute: {err}"),
            },
            _ => panic!("Unsupported span attribute"),
        }
    })
}
