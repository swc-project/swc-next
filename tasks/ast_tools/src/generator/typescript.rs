//! Public ESTree declarations. Rust types supply defaults; ESTree annotations
//! describe the JS projection and deliberately wider input types.
use std::{collections::BTreeSet, fmt::Write};

use super::estree::{ENUMS, estree_field_name, estree_type, is_ts_field, ts_extra_fields};
use crate::{
    output::RawOutput,
    schema::{AstStruct, AstType, Schema, TypeId},
};

pub fn typescript(schema: &Schema) -> RawOutput {
    let mut output = String::from(PREAMBLE);
    for (name, values) in ENUMS {
        write_union(&mut output, &type_name(name), values.iter().copied());
    }
    let mut nodes = BTreeSet::new();
    for ty in &schema.types {
        let (name, attrs) = match ty {
            AstType::Struct(ast) => (&ast.name, &ast._attrs.estree),
            AstType::Enum(ast) => (&ast.name, &ast._attrs.estree),
            _ => continue,
        };
        let name = type_name(name);
        if let Some(alias) = &attrs.ts_alias {
            writeln!(output, "export type {name} = {alias};\n").unwrap();
        } else {
            match ty {
                AstType::Struct(ast) => {
                    write_struct(&mut output, ast, schema);
                    nodes.insert(name.clone());
                }
                AstType::Enum(ast) => {
                    let variants: BTreeSet<_> = ast
                        .variants
                        .iter()
                        .map(|variant| {
                            variant.type_id.map_or_else(
                                || format!("\"{}\"", variant.name),
                                |id| reference(id, schema),
                            )
                        })
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
    // Aliases for transparent Rust nodes are not nodes in their own right.
    // Their ESTree targets are already reachable through Expression/Statement.
    output.push_str("export type Expression = Expr;\nexport type Statement = Stmt;\nexport type Declaration = Decl;\nexport type Pattern = BindingPattern;\n\n");
    nodes.insert("Expression".into());
    nodes.insert("Statement".into());
    write_union(&mut output, "Node", nodes.iter().map(String::as_str));
    output.truncate(output.trim_end().len());
    output.push('\n');
    RawOutput {
        path: "packages/types/index.d.ts".into(),
        content: output.into_bytes(),
    }
}

fn write_struct(output: &mut String, ast: &AstStruct, schema: &Schema) {
    let attrs = &ast._attrs.estree;
    let name = type_name(&ast.name);
    writeln!(output, "export interface {name} extends BaseNode {{").unwrap();
    let discriminator = attrs
        .ts_type
        .clone()
        .unwrap_or_else(|| format!("\"{}\"", estree_type(&ast.name)));
    writeln!(output, "  type: {discriminator};").unwrap();
    let mut field_names = BTreeSet::from(["type".to_string()]);
    for field in &ast.fields {
        if field.attrs.estree_skip || field.attrs.ts_skip {
            continue;
        }
        let name = field
            .attrs
            .ts_rename
            .clone()
            .unwrap_or_else(|| estree_field_name(&ast.name, &field.name));
        assert!(
            field_names.insert(name.clone()),
            "Duplicate ESTree field {}.{name}",
            ast.name
        );
        let ty = field
            .attrs
            .ts_type
            .clone()
            .unwrap_or_else(|| reference(field.type_id, schema));
        let optional = field.attrs.ts_optional || is_ts_field(&ast.name, &name);
        writeln!(output, "  {name}{}: {ty};", if optional { "?" } else { "" }).unwrap();
    }
    for (name, value) in ts_extra_fields(&ast.name) {
        if field_names.insert((*name).to_string()) {
            let ty = match *value {
                "[]" => "Array<Decorator>",
                "false" | "true" => "boolean",
                "null" if *name == "directive" => "string | null",
                value => value,
            };
            writeln!(output, "  {name}?: {ty};").unwrap();
        }
    }
    for (name, converter) in &attrs.add_fields {
        assert!(
            field_names.insert(name.clone()),
            "Duplicate ESTree field {}.{name}",
            ast.name
        );
        let (ty, optional) = added_field_type(converter);
        writeln!(output, "  {name}{}: {ty};", if optional { "?" } else { "" }).unwrap();
    }
    output.push_str("}\n\n");
}

fn added_field_type(converter: &str) -> (&str, bool) {
    match converter {
        "Null" => ("null", false),
        "False" | "True" => ("boolean", false),
        "TsFalse" => ("boolean", true),
        "SourceType" => ("SourceType", false),
        "Init" => ("\"init\"", false),
        "Global" => ("\"global\"", false),
        "This" => ("\"this\"", false),
        "EmptyArray" => ("Array<Decorator>", true),
        "Raw" | "StringValue" => ("string", false),
        "BigIntValue" => ("bigint", false),
        "NullRaw" => ("\"null\"", false),
        "RegExpValue" => ("RegExp | null", false),
        "RegExpPattern" => ("{ pattern: string; flags: string }", false),
        "TemplateValue" => ("{ raw: string; cooked: string | null }", false),
        _ => panic!("Unknown ESTree field converter {converter}"),
    }
}

fn reference(id: TypeId, schema: &Schema) -> String {
    match &schema.types[id] {
        AstType::Struct(ast) => type_name(&ast.name),
        AstType::Enum(ast) => type_name(&ast.name),
        AstType::SubRange(range) => format!("Array<{}>", reference(range.inner_type_id, schema)),
        AstType::Option(option) => format!("{} | null", reference(option.inner_type_id, schema)),
        AstType::Primitive(primitive) => match primitive.name {
            "AstUtf8" | "AstWtf8" => "string".into(),
            "bool" => "boolean".into(),
            "Span" => "{ start: number; end: number }".into(),
            name if ENUMS.iter().any(|(enum_name, _)| *enum_name == name) => type_name(name),
            "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "i8" | "i16" | "i32" | "i64"
            | "i128" | "isize" | "f32" | "f64" => "number".into(),
            name => panic!("Missing ESTree TypeScript mapping for {name}"),
        },
    }
}

fn type_name(name: &str) -> String {
    if name.starts_with("Ts") || name.starts_with("Jsx") {
        estree_type(name)
    } else {
        name.to_string()
    }
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
  range?: [number, number];
  loc?: {
    start: { line: number; column: number };
    end: { line: number; column: number };
    source?: string | null;
  } | null;
}

"#;
