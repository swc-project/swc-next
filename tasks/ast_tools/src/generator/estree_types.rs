//! Public ESTree declarations derived from Rust types and AST annotations.
use std::{collections::BTreeSet, fmt::Write};

use crate::{
    output::RawOutput,
    schema::{AstEnumVariant, AstStruct, AstType, Schema, TypeId},
};

pub fn estree_types(schema: &Schema) -> RawOutput {
    let mut output = String::from(PREAMBLE);
    for enumeration in &schema.scalar_enums {
        let name = type_name(&enumeration.name, schema);
        if let Some(alias) = &enumeration.attrs.ts_alias {
            writeln!(output, "export type {name} = {alias};\n").unwrap();
        } else {
            let variants: Vec<_> = enumeration
                .variants
                .iter()
                .map(|variant| variant_type(variant, schema))
                .collect();
            write_union(&mut output, &name, variants.iter().map(String::as_str));
        }
        if let Some(def) = &enumeration.attrs.add_ts_def {
            writeln!(output, "export {def}\n").unwrap();
        }
    }
    let mut nodes = BTreeSet::new();
    for ty in &schema.types {
        let (name, attrs) = match ty {
            AstType::Struct(ast) => (&ast.name, &ast._attrs.estree),
            AstType::Enum(ast) => (&ast.name, &ast._attrs.estree),
            _ => continue,
        };
        let name = type_name(name, schema);
        if let Some(alias) = &attrs.ts_alias {
            writeln!(output, "export type {name} = {alias};\n").unwrap();
            if alias
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
            {
                nodes.insert(alias.clone());
            }
        } else {
            match ty {
                AstType::Struct(ast) => {
                    writeln!(output, "export interface {name} extends BaseNode {{").unwrap();
                    for field in fields(ast, schema) {
                        writeln!(
                            output,
                            "  {}{}: {};",
                            field.name,
                            if field.optional { "?" } else { "" },
                            field.ts_type
                        )
                        .unwrap();
                    }
                    output.push_str("}\n\n");
                    nodes.insert(name.clone());
                }
                AstType::Enum(ast) => {
                    let variants: BTreeSet<_> = ast
                        .variants
                        .iter()
                        .map(|variant| variant_type(variant, schema))
                        .collect();
                    write_union(&mut output, &name, variants.iter().map(String::as_str));
                }
                _ => unreachable!(),
            }
        }
        if let Some(def) = &attrs.add_ts_def {
            writeln!(output, "export {def}\n").unwrap();
        }
    }
    write_union(&mut output, "Node", nodes.iter().map(String::as_str));
    output.truncate(output.trim_end().len());
    output.push('\n');
    RawOutput {
        path: "packages/types/index.d.ts".into(),
        content: output.into_bytes(),
    }
}

fn variant_type(variant: &AstEnumVariant, schema: &Schema) -> String {
    variant.estree.ts_type.clone().unwrap_or_else(|| {
        if let Some(id) = variant.type_id {
            variant
                .estree
                .rename
                .clone()
                .unwrap_or_else(|| reference(id, schema))
        } else {
            format!(
                "{:?}",
                variant.estree.rename.as_ref().unwrap_or(&variant.name)
            )
        }
    })
}

fn default_name(name: &str) -> String {
    if let Some(suffix) = name.strip_prefix("Ts") {
        format!("TS{}", default_name(suffix))
    } else if let Some(suffix) = name.strip_prefix("Jsx") {
        format!("JSX{suffix}")
    } else if let Some(suffix) = name.strip_prefix("Js") {
        format!("JS{suffix}")
    } else {
        name.into()
    }
}

fn type_name(name: &str, schema: &Schema) -> String {
    schema
        .types
        .iter()
        .find_map(|ty| match ty {
            AstType::Enum(ast) if ast.name == name => ast._attrs.estree.rename.clone(),
            _ => None,
        })
        .or_else(|| {
            schema
                .scalar_enums
                .iter()
                .find(|enumeration| enumeration.name == name)
                .and_then(|enumeration| enumeration.attrs.rename.clone())
        })
        .unwrap_or_else(|| default_name(name))
}

fn reference(id: TypeId, schema: &Schema) -> String {
    match &schema.types[id] {
        AstType::Struct(ast) => type_name(&ast.name, schema),
        AstType::Enum(ast) => type_name(&ast.name, schema),
        AstType::SubRange(range) => format!("Array<{}>", reference(range.inner_type_id, schema)),
        AstType::Option(option) => format!("{} | null", reference(option.inner_type_id, schema)),
        AstType::Primitive(primitive) => match primitive.name {
            "AstInternedUtf8" | "AstUtf8" | "AstWtf8" => "string".into(),
            "bool" => "boolean".into(),
            "Span" => "{ start: number; end: number }".into(),
            name if schema
                .scalar_enums
                .iter()
                .any(|enumeration| enumeration.name == name) =>
            {
                type_name(name, schema)
            }
            "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "i8" | "i16" | "i32" | "i64"
            | "i128" | "isize" | "f32" | "f64" => "number".into(),
            name => panic!("missing ESTree type mapping for {name}"),
        },
    }
}

struct Field {
    name: String,
    ts_type: String,
    optional: bool,
}

fn fields(ast: &AstStruct, schema: &Schema) -> Vec<Field> {
    use convert_case::{Case, Casing};

    let attrs = &ast._attrs.estree;
    let mut output = vec![Field {
        name: "type".into(),
        ts_type: attrs.ts_type.clone().unwrap_or_else(|| {
            format!(
                "{:?}",
                attrs
                    .rename
                    .clone()
                    .unwrap_or_else(|| default_name(&ast.name))
            )
        }),
        optional: false,
    }];
    for field in &ast.fields {
        if field.attrs.skip {
            continue;
        }
        output.push(Field {
            name: field.attrs.rename.clone().unwrap_or_else(|| {
                field
                    .name
                    .strip_prefix("r#")
                    .unwrap_or(&field.name)
                    .to_case(Case::Camel)
            }),
            ts_type: field
                .attrs
                .ts_type
                .clone()
                .unwrap_or_else(|| reference(field.type_id, schema)),
            optional: field.attrs.optional,
        });
    }
    output.extend(attrs.fields.iter().map(|field| Field {
        name: field.name.clone(),
        ts_type: field.ts_type.clone(),
        optional: field.optional,
    }));
    let mut names = BTreeSet::from(["start", "end"]);
    for field in &output {
        assert!(
            names.insert(&field.name),
            "duplicate ESTree field {}.{}",
            ast.name,
            field.name
        );
    }
    output
}

fn write_union<'a>(output: &mut String, name: &str, variants: impl Iterator<Item = &'a str>) {
    let variants: Vec<_> = variants.collect();
    let joined = variants.join(" | ");
    if name.len() + joined.len() < 80 {
        writeln!(output, "export type {name} = {joined};\n").unwrap();
    } else {
        writeln!(output, "export type {name} =").unwrap();
        for (index, variant) in variants.iter().enumerate() {
            writeln!(
                output,
                "  | {variant}{}",
                if index + 1 == variants.len() {
                    ";\n"
                } else {
                    ""
                }
            )
            .unwrap();
        }
    }
}

const PREAMBLE: &str = r#"// Generated by tasks/ast_tools. Do not edit.

export interface BaseNode {
  start: number;
  end: number;
}

"#;
