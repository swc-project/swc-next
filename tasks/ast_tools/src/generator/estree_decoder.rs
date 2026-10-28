use std::fmt::Write;

use crate::{
    output::RawOutput,
    schema::{AstStruct, AstType, FieldLocation, Schema},
};

use super::{
    estree::{ENUMS, estree_field_name, estree_type, is_ts_field, ts_extra_fields},
    js_minifier::minify,
};

const OUTPUT_DIR: &str = "packages/decoder";

// TypeScript keeps common small node bodies in a hot dispatcher. TSX and empty
// programs use the complete dispatcher to avoid unnecessary calls or closures.
// Both dispatchers recurse through the grammar-selected node function.
// FormalParameter and Function stay out of the hot switch to limit frame size.
const HOT_TS_NODE_KINDS: &[&str] = &[
    "IdentifierReference",
    "IdentifierName",
    "MemberExpression",
    "BindingIdentifier",
    "ExpressionStatement",
    "CallExpression",
    "ThisExpression",
    "AssignmentExpression",
    "BinaryExpression",
    "VariableDeclarator",
    "StringLiteral",
    "NumericLiteral",
    "IfStatement",
    "BlockStatement",
    "VariableDeclaration",
    "FormalParameters",
    "FunctionBody",
    "ReturnStatement",
    "LogicalExpression",
    "ObjectProperty",
    "ArrayExpression",
    "ObjectExpression",
    "UnaryExpression",
    "TsTypeReference",
    "TsTypeAnnotation",
    "ImportSpecifier",
];

pub fn estree_decoder(schema: &Schema) -> Vec<RawOutput> {
    let mut outputs = Vec::new();
    for (is_ts, suffix, name) in [(false, "js", "decodeJs"), (true, "ts", "decodeTs")] {
        let pretty_path = format!("{OUTPUT_DIR}/decode-{suffix}.pretty.js");
        let minified_path = format!("{OUTPUT_DIR}/decode-{suffix}.js");
        let output = decoder_source(schema, is_ts, name);
        let minified = minify(&output, &pretty_path);
        outputs.push(RawOutput {
            path: pretty_path,
            content: output.into_bytes(),
        });
        outputs.push(RawOutput {
            path: minified_path,
            content: minified.into_bytes(),
        });
    }

    outputs
}

fn decoder_source(schema: &Schema, is_ts: bool, name: &str) -> String {
    let mut output = String::new();
    let decoder_enums: Vec<_> = ENUMS
        .iter()
        .filter(|(name, _)| {
            is_ts
                || !(*name == "Accessibility"
                    || *name == "ImportOrExportKind"
                    || name.starts_with("Ts"))
        })
        .collect();
    let mut enums = String::from("const ENUMS = Object.freeze([\n");
    for (name, values) in &decoder_enums {
        writeln!(enums, "  [{}], // {name}", values.join(", ")).unwrap();
    }
    enums.push_str("\n]);");
    output.push_str(&PREAMBLE.replace("__ENUMS__", &enums));
    if is_ts {
        output.push_str("const DEEP_AST = {};\n");
    }
    output.push_str(DECODE_HELPERS);
    writeln!(
        output,
        "export function {name}(buffer, source, sourceBuffer) {{"
    )
    .unwrap();
    output.push_str(DECODE_PREAMBLE);
    if is_ts {
        output.push_str(
            "  let node = nodeCount === 1 || ((flags >> 2) & 7) === 3 ? nodeAll : nodeHot;\n  let coldDepth = 0;\n",
        );
        output.push_str(&NODE_PREAMBLE.replace(
            "function node(index)",
            "function nodeHot(index, typeOverride)",
        ));
    } else {
        output.push_str(NODE_PREAMBLE);
    }

    let mut all_cases = String::new();
    for (kind, ty) in schema
        .types
        .iter()
        .filter_map(|ty| match ty {
            AstType::Struct(ast) => Some(ast),
            _ => None,
        })
        .enumerate()
    {
        if !is_ts && ty.name.starts_with("Ts") {
            continue;
        }
        if is_ts {
            write_node_case(&mut all_cases, kind, ty, schema, true);
            if HOT_TS_NODE_KINDS.contains(&ty.name.as_str()) {
                write_node_case(&mut output, kind, ty, schema, true);
            }
        } else {
            write_node_case(&mut output, kind, ty, schema, false);
        }
    }

    if is_ts {
        output.push_str(HOT_POSTAMBLE);
        output.push_str(&NODE_PREAMBLE.replace(
            "function node(index)",
            "function nodeAll(index, typeOverride)",
        ));
        output.push_str(&all_cases);
    }
    if is_ts {
        output.push_str(&POSTAMBLE.replace(
            "      return _program !== undefined\n        ? _program\n        : (_program = node(programIndex));",
            TS_PROGRAM_GETTER,
        ));
    } else {
        output.push_str(POSTAMBLE);
    }
    // Assign indices from the same filtered table used to emit the enum arrays.
    for (index, (name, _)) in decoder_enums.iter().enumerate() {
        for helper in ["enumValue", "optionalEnum"] {
            output = output.replace(
                &format!("{helper}(\"{name}\","),
                &format!("{helper}({index},"),
            );
        }
    }
    output
}

fn write_node_case(
    output: &mut String,
    kind: usize,
    ast: &AstStruct,
    schema: &Schema,
    is_ts: bool,
) {
    writeln!(output, "      case {kind}: {{").unwrap();

    let layout = &ast.layout;
    let mut fields = Vec::new();

    for (field_index, field) in ast.fields.iter().enumerate() {
        let skip = field.attrs.estree_skip;
        let field_ty = &schema.types[field.type_id];
        let (location, aligned_i32) = match layout.fields[field_index] {
            FieldLocation::Payload { offset, .. } => {
                (format!("payload + {offset}"), offset % 4 == 0)
            }
            FieldLocation::Extra { index } => (format!("extra + (extraId + {index}) * 8"), true),
        };
        if skip {
            continue;
        }
        // These fields are source-backed, and node spans use UTF-16 offsets.
        let expression = match (ast.name.as_str(), field.name.as_str()) {
            ("StringLiteral", "raw") | ("JsxText", "value") => "source.slice(start, end)".into(),
            ("BigIntLiteral", "raw") => "source.slice(start, end - 1)".into(),
            ("NumericLiteral", "raw") => {
                let raw_start = read_i32(&location, aligned_i32);
                let raw_end = read_i32(&format!("{location} + 4"), aligned_i32);
                format!("{raw_start} === NULL && {raw_end} === 0 ? null : source.slice(start, end)")
            }
            _ => read_expression(field_ty, schema, &location, aligned_i32),
        };
        let field_name = estree_field_name(&ast.name, &field.name);
        let range = matches!(field_ty, AstType::SubRange(_)).then(|| {
            format!(
                "{}, {}",
                read_i32(&location, aligned_i32),
                read_i32(&format!("{location} + 4"), aligned_i32)
            )
        });
        fields.push(DecodedField {
            name: field_name,
            expression,
            range,
        });
    }

    let mut body = String::new();
    if is_special_node(&ast.name) {
        write_special_node(&mut body, &ast.name, &fields, is_ts);
    } else {
        write_generic_node(&mut body, &ast.name, &fields, is_ts);
    }
    if body.contains("extraId") {
        let offset = layout
            .extra_start_offset()
            .expect("extra fields need a payload pointer");
        writeln!(
            output,
            "        const extraId = _i32[(payload + {offset}) >> 2];"
        )
        .unwrap();
    }
    output.push_str(&body);
    writeln!(output, "        break;").unwrap();
    writeln!(output, "      }}").unwrap();
}

struct DecodedField {
    name: String,
    expression: String,
    range: Option<String>,
}

fn read_expression(ty: &AstType, schema: &Schema, location: &str, aligned_i32: bool) -> String {
    match ty {
        AstType::Struct(_) | AstType::Enum(_) => {
            format!("node({})", read_i32(location, aligned_i32))
        }
        AstType::SubRange(range) => {
            let inner = &schema.types[range.inner_type_id];
            let holes = matches!(inner, AstType::Option(_));
            let start = read_i32(location, aligned_i32);
            let end = read_i32(&format!("{location} + 4"), aligned_i32);
            format!("nodeArray({start}, {end}, {holes})")
        }
        AstType::Option(option) => match &schema.types[option.inner_type_id] {
            AstType::Struct(_) | AstType::Enum(_) => {
                format!("optionalNode({})", read_i32(location, aligned_i32))
            }
            AstType::Primitive(primitive) if primitive.name == "ImportPhase" => {
                format!("optionalEnum(\"ImportPhase\", _dv.getUint8({location}))")
            }
            AstType::Primitive(primitive) if matches!(primitive.name, "ScopeId" | "SymbolId") => {
                "undefined".into()
            }
            AstType::Primitive(primitive) if primitive.name == "Span" => {
                let start = read_i32(location, aligned_i32);
                let end = read_i32(&format!("{location} + 4"), aligned_i32);
                format!("{{ start: {start}, end: {end} }}")
            }
            AstType::Primitive(primitive) if primitive.name == "AstUtf8" => {
                let start = read_i32(location, aligned_i32);
                let end = read_i32(&format!("{location} + 4"), aligned_i32);
                format!("{start} === NULL && {end} === 0 ? null : str({start}, {end})")
            }
            inner => panic!("unsupported optional decoder type: {}", inner.name()),
        },
        AstType::Primitive(primitive) => match primitive.name {
            "AstUtf8" | "AstWtf8" => {
                let start = read_i32(location, aligned_i32);
                let end = read_i32(&format!("{location} + 4"), aligned_i32);
                format!("str({start}, {end})")
            }
            "bool" => format!("_dv.getUint8({location}) !== 0"),
            "u8" => format!("_dv.getUint8({location})"),
            "i8" => format!("_dv.getInt8({location})"),
            "u16" => format!("_dv.getUint16({location}, true)"),
            "i16" => format!("_dv.getInt16({location}, true)"),
            "u32" => read_u32(location, aligned_i32),
            "i32" => format!("_dv.getInt32({location}, true)"),
            "f64" => format!("_dv.getFloat64({location}, true)"),
            "Span" => {
                let start = read_i32(location, aligned_i32);
                let end = read_i32(&format!("{location} + 4"), aligned_i32);
                format!("{{ start: {start}, end: {end} }}")
            }
            "ScopeId" | "SymbolId" => "undefined".into(),
            name if schema.repr_sizes.contains_key(name)
                || matches!(
                    name,
                    "SourceType"
                        | "ImportPhase"
                        | "MethodDefinitionKind"
                        | "Accessibility"
                        | "ClassType"
                        | "BinaryOperator"
                        | "LogicalOperator"
                        | "UnaryOperator"
                        | "UpdateOperator"
                        | "AssignmentOperator"
                        | "VariableKind"
                        | "FunctionType"
                        | "FormalParameterKind"
                        | "PropertyKind"
                        | "ImportOrExportKind"
                        | "TsTypeOperatorKind"
                        | "TsMethodSignatureKind"
                        | "TsModuleDeclarationKind"
                        | "TsMappedTypeModifier"
                ) =>
            {
                format!("enumValue(\"{name}\", _dv.getUint8({location}))")
            }
            name => panic!("unsupported primitive decoder type: {name}"),
        },
    }
}

fn read_i32(location: &str, aligned: bool) -> String {
    if aligned {
        format!("_i32[({location}) >> 2]")
    } else {
        format!("_dv.getInt32({location}, true)")
    }
}

fn read_u32(location: &str, aligned: bool) -> String {
    if aligned {
        format!("({} >>> 0)", read_i32(location, true))
    } else {
        format!("_dv.getUint32({location}, true)")
    }
}

fn write_generic_node(output: &mut String, name: &str, fields: &[DecodedField], is_ts: bool) {
    write_object_literal(
        output,
        name,
        fields
            .iter()
            .filter(|field| is_ts || !is_ts_field(name, &field.name)),
        if is_ts { ts_extra_fields(name) } else { &[] },
        8,
    );
}

fn write_object_literal<'a>(
    output: &mut String,
    name: &str,
    fields: impl Iterator<Item = &'a DecodedField>,
    extras: &[(&str, &str)],
    indent: usize,
) {
    let padding = " ".repeat(indent);
    writeln!(
        output,
        "{padding}result = {{ type: \"{}\", start, end,",
        estree_type(name)
    )
    .unwrap();
    for field in fields {
        writeln!(output, "{padding}  {}: {},", field.name, field.expression).unwrap();
    }
    for (field, value) in extras {
        writeln!(output, "{padding}  {field}: {value},").unwrap();
    }
    writeln!(output, "{padding}}};").unwrap();
}

fn is_special_node(name: &str) -> bool {
    matches!(
        name,
        "Program"
            | "FunctionBody"
            | "TsModuleBlock"
            | "FormalParameter"
            | "FormalParameterRest"
            | "VariableDeclarator"
            | "CatchClause"
            | "TsIndexSignatureParameter"
            | "FormalParameters"
            | "Function"
            | "TsFunction"
            | "ArrowFunctionExpression"
            | "Directive"
            | "StringLiteral"
            | "NumericLiteral"
            | "BigIntLiteral"
            | "BooleanLiteral"
            | "NullLiteral"
            | "RegExpLiteral"
            | "TemplateElement"
            | "Class"
            | "MethodDefinition"
            | "TsMethodDefinition"
            | "PropertyDefinition"
            | "UnaryExpression"
            | "BindingProperty"
            | "ArrayPattern"
            | "ObjectPattern"
            | "JsxText"
            | "TsFunctionType"
            | "TsConstructorType"
            | "TsMethodSignature"
            | "TsCallSignatureDeclaration"
            | "TsConstructSignatureDeclaration"
            | "TsMappedType"
            | "TsModuleDeclaration"
            | "TsGlobalDeclaration"
            | "TsThisParameter"
    )
}

fn write_special_node(output: &mut String, name: &str, fields: &[DecodedField], is_ts: bool) {
    let mut body = String::new();
    {
        let output = &mut body;
        match name {
        "Program" => {
            writeln!(output, "        const body = {};", combined_body_expression(fields)).unwrap();
            output.push_str(if is_ts {
                "        const first = body[0];\n        const declaration = first?.declaration ?? first;\n        const firstStart = first?.start ?? end;\n        const programStart = Math.min(firstStart, declaration?.decorators?.[0]?.start ?? firstStart);\n        result = { type: \"Program\", start: programStart, end, sourceType, hashbang: f_hashbang, body };\n"
            } else {
                "        result = { type: \"Program\", start, end, sourceType, hashbang: f_hashbang, body };\n"
            });
        }
        "FunctionBody" | "TsModuleBlock" => {
            writeln!(
                output,
                "        result = {{ type: \"{}\", start, end, body: {} }};",
                if name == "TsModuleBlock" {
                    "TSModuleBlock"
                } else {
                    "BlockStatement"
                },
                combined_body_expression(fields)
            )
            .unwrap();
        }
        "FormalParameter" => {
            if is_ts {
                output.push_str(
                    r#"        if (f_initializer === null) {
          f_pattern.optional = f_optional;
          f_pattern.typeAnnotation = f_typeAnnotation;
          f_pattern.end = end;
          f_pattern.decorators = f_decorators;
          result = f_pattern;
        } else {
          const left = f_pattern;
          left.optional = f_optional;
          left.typeAnnotation = f_typeAnnotation;
          left.end = f_typeAnnotation?.end ?? (f_optional ? left.end + 1 : left.end);
          result = {
            type: "AssignmentPattern", start: left.start, end,
            left, right: f_initializer,
            decorators: f_decorators, optional: false, typeAnnotation: null,
          };
        }
"#,
                );
            } else {
                output.push_str("        const left = f_pattern;\n");
                output.push_str("        if (f_initializer === null) {\n          result = left;\n          result.end = end;\n");
                output.push_str("        } else {\n          result = { type: \"AssignmentPattern\", start: left.start, end, left, right: f_initializer };\n");
                output.push_str("        }\n");
            }
        }
        "FormalParameterRest" => {
            output.push_str(if is_ts {
                "        result = { type: \"RestElement\", start, end, argument: f_argument, decorators: f_decorators, optional: false, typeAnnotation: f_typeAnnotation, value: null };\n"
            } else {
                "        result = { type: \"RestElement\", start, end, argument: f_argument };\n"
            });
        }
        "VariableDeclarator" => {
            output.push_str(if is_ts {
                "        f_id.typeAnnotation = f_typeAnnotation;\n        f_id.end = f_typeAnnotation?.end ?? f_id.end;\n        result = { type: \"VariableDeclarator\", start, end, id: f_id, init: f_init, definite: f_definite };\n"
            } else {
                "        result = { type: \"VariableDeclarator\", start, end, id: f_id, init: f_init };\n"
            });
        }
        "CatchClause" => {
            output.push_str(if is_ts {
                "        if (f_param !== null) {\n          f_param.typeAnnotation = f_typeAnnotation;\n          f_param.end = f_typeAnnotation?.end ?? f_param.end;\n        }\n        result = { type: \"CatchClause\", start, end, param: f_param, body: f_body };\n"
            } else {
                "        result = { type: \"CatchClause\", start, end, param: f_param, body: f_body };\n"
            });
        }
        "TsIndexSignatureParameter" => output.push_str(
            r#"        f_name.start = start;
        f_name.end = end;
        f_name.typeAnnotation = f_typeAnnotation;
        result = f_name;
"#,
        ),
        "FormalParameters" => output.push_str(
            "        if (f_rest !== null) f_items.push(f_rest);\n        result = { params: f_items };\n",
        ),
        "Function" => {
            if is_ts {
                output.push_str("        if (f_thisParam !== null) f_params.params.unshift(f_thisParam);\n");
            }
            output.push_str(
                r#"        result = {
          type: f_functionType, start, end,
          id: f_id,
          generator: f_generator,
          async: f_async,
"#,
            );
            output.push_str("          params: f_params.params,\n");
            output.push_str(
                r#"          body: f_body,
          expression: false,
"#,
            );
            if is_ts {
                output.push_str("          typeParameters: f_typeParameters,\n          returnType: f_returnType,\n          declare: false,\n");
            }
            output.push_str("        };\n");
        }
        "TsFunction" => output.push_str(
            r#"        if (f_thisParam !== null) f_params.params.unshift(f_thisParam);
        result = {
          type: typeOverride ?? "TSDeclareFunction",
          start, end,
          id: f_id,
          generator: f_generator,
          async: f_async,
          params: f_params.params,
          body: null,
          expression: false,
          typeParameters: f_typeParameters,
          returnType: f_returnType,
          declare: f_declare,
        };
"#,
        ),
        "ArrowFunctionExpression" => {
            output.push_str(
                r#"        result = {
          type: "ArrowFunctionExpression", start, end,
          id: null, generator: false, async: f_async,
          params: f_params.params,
          body: f_body, expression: f_expression,
"#,
            );
            if is_ts {
                output.push_str("          typeParameters: f_typeParameters,\n          returnType: f_returnType,\n");
            }
            output.push_str("        };\n");
        }
        "Directive" => output.push_str(
            r#"        result = {
          type: "ExpressionStatement", start, end,
          expression: f_expression, directive: f_value,
        };
"#,
        ),
        "StringLiteral" => output.push_str(
            "        result = { type: \"Literal\", start, end, value: f_value, raw: f_raw };\n",
        ),
        "NumericLiteral" => output.push_str(
            r#"        const value = f_value;
        const raw = f_raw ?? String(value);
        result = { type: "Literal", start, end, value, raw };
"#,
        ),
        "BigIntLiteral" => output.push_str(
            r#"        const raw = `${f_raw}n`;
        const value = BigInt(f_raw.replaceAll("_", ""));
        result = { type: "Literal", start, end, value, raw, bigint: value.toString() };
"#,
        ),
        "BooleanLiteral" => output.push_str(
            r#"        result = {
          type: "Literal", start, end,
          value: f_value, raw: f_value ? "true" : "false",
        };
"#,
        ),
        "NullLiteral" => output.push_str(
            "        result = { type: \"Literal\", start, end, value: null, raw: \"null\" };\n",
        ),
        "RegExpLiteral" => output.push_str(
            r#"        let value = null;
        try {
          value = new RegExp(f_pattern, f_flags);
        } catch {}
        result = {
          type: "Literal", start, end, value,
          raw: `/${f_pattern}/${f_flags}`,
          regex: { pattern: f_pattern, flags: [...f_flags].sort().join("") },
        };
"#,
        ),
        "TemplateElement" => {
            output.push_str(if is_ts {
                "        result = { type: \"TemplateElement\", start: start - 1, end: f_tail ? end + 1 : end + 2, value: { raw: f_raw, cooked: f_isCookedUndefined ? null : f_cooked }, tail: f_tail };\n"
            } else {
                "        result = { type: \"TemplateElement\", start, end, value: { raw: f_raw, cooked: f_isCookedUndefined ? null : f_cooked }, tail: f_tail };\n"
            });
        }
        "Class" => {
            output.push_str(
                r#"        result = {
          type: f_classType, start, end,
          decorators: f_decorators,
          id: f_id,
          superClass: f_superClass,
          body: f_body,
"#,
            );
            if is_ts {
                output.push_str("          typeParameters: f_typeParameters,\n          superTypeArguments: f_superTypeArguments,\n          implements: f_implements,\n          abstract: f_abstract,\n          declare: f_declare,\n");
            }
            output.push_str("        };\n");
        }
        "MethodDefinition" | "TsMethodDefinition" => {
            writeln!(
                output,
                "        result = {{\n          type: {},",
                if is_ts {
                    "f_abstract ? \"TSAbstractMethodDefinition\" : \"MethodDefinition\""
                } else {
                    "\"MethodDefinition\""
                }
            )
            .unwrap();
            output.push_str(
                r#"          start, end,
          decorators: f_decorators,
          key: f_key,
          value: f_value,
          kind: f_kind,
          computed: f_computed,
          static: f_static,
"#,
            );
            if is_ts {
                output.push_str("          override: f_override,\n          optional: f_optional,\n          accessibility: f_accessibility,\n");
            }
            output.push_str("        };\n");
        }
        "PropertyDefinition" => {
            output.push_str("        const accessor = f_accessor;\n        result = {\n");
            output.push_str(if is_ts {
                "          type: f_abstract ? (accessor ? \"TSAbstractAccessorProperty\" : \"TSAbstractPropertyDefinition\") : (accessor ? \"AccessorProperty\" : \"PropertyDefinition\"),\n"
            } else {
                "          type: accessor ? \"AccessorProperty\" : \"PropertyDefinition\",\n"
            });
            output.push_str(
                r#"          start, end,
          decorators: f_decorators,
          key: f_key,
          value: f_value,
          computed: f_computed,
          static: f_static,
"#,
            );
            if is_ts {
                output.push_str("          typeAnnotation: f_typeAnnotation,\n          declare: f_declare,\n          override: f_override,\n          optional: f_optional,\n          definite: f_definite,\n          readonly: f_readonly,\n          accessibility: f_accessibility,\n");
            }
            output.push_str("        };\n");
        }
        "UnaryExpression" => output.push_str(
            r#"        result = {
          type: "UnaryExpression", start, end,
          operator: f_operator, prefix: true, argument: f_argument,
        };
"#,
        ),
        "BindingProperty" => {
            output.push_str(
                r#"        result = {
          type: "Property", start, end, kind: "init",
          key: f_key, value: f_value,
          method: false, shorthand: f_shorthand, computed: f_computed,
"#,
            );
            if is_ts {
                output.push_str("          optional: false,\n");
            }
            output.push_str("        };\n");
        }
        "ArrayPattern" => {
            output.push_str(
                r#"        if (f_rest !== null) f_elements.push(f_rest);
"#,
            );
            output.push_str(if is_ts {
                "        result = { type: \"ArrayPattern\", start, end, elements: f_elements, decorators: [], optional: false, typeAnnotation: null };\n"
            } else {
                "        result = { type: \"ArrayPattern\", start, end, elements: f_elements };\n"
            });
        }
        "ObjectPattern" => {
            output.push_str("        if (f_rest !== null) f_properties.push(f_rest);\n");
            output.push_str(if is_ts {
                "        result = { type: \"ObjectPattern\", start, end, properties: f_properties, decorators: [], optional: false, typeAnnotation: null };\n"
            } else {
                "        result = { type: \"ObjectPattern\", start, end, properties: f_properties };\n"
            });
        }
        "JsxText" => output.push_str(
            "        result = { type: \"JSXText\", start, end, value: f_value, raw: f_value };\n",
        ),
        "TsFunctionType"
        | "TsConstructorType"
        | "TsMethodSignature"
        | "TsCallSignatureDeclaration"
        | "TsConstructSignatureDeclaration" => {
            if matches!(
                name,
                "TsFunctionType" | "TsMethodSignature" | "TsCallSignatureDeclaration"
            ) {
                output.push_str(
                    "        if (f_thisParam !== null) f_params.params.unshift(f_thisParam);\n",
                );
            }
            writeln!(
                output,
                "        result = {{\n          type: \"{}\", start, end,",
                estree_type(name)
            )
            .unwrap();
            if name == "TsConstructorType" {
                output.push_str("          abstract: f_abstract,\n");
            }
            if name == "TsMethodSignature" {
                output.push_str(
                    r#"          key: f_key,
          computed: f_computed,
          optional: f_optional,
          kind: f_kind,
"#,
                );
            }
            output.push_str(
                r#"          typeParameters: f_typeParameters,
          params: f_params.params,
          returnType: f_returnType,
"#,
            );
            if name == "TsMethodSignature" {
                output.push_str("          accessibility: null,\n          readonly: false,\n          static: false,\n");
            }
            output.push_str("        };\n");
        }
        "TsMappedType" => output.push_str(
            r#"        result = {
          type: "TSMappedType", start, end,
          key: f_key, constraint: f_constraint,
          nameType: f_nameType, typeAnnotation: f_typeAnnotation,
          optional: f_optional === null ? false : f_optional,
          readonly: f_readonly,
        };
"#,
        ),
        "TsModuleDeclaration" => output.push_str(
            r#"        result = f_body === null
          ? { type: "TSModuleDeclaration", start, end, id: f_id, kind: f_kind, declare: f_declare, global: false }
          : { type: "TSModuleDeclaration", start, end, id: f_id, kind: f_kind, declare: f_declare, global: false, body: f_body };
"#,
        ),
        "TsGlobalDeclaration" => output.push_str(
            r#"        result = {
          type: "TSModuleDeclaration", start, end,
          id: f_id, body: f_body, kind: "global",
          declare: f_declare, global: true,
        };
"#,
        ),
        "TsThisParameter" => output.push_str(
            r#"        result = {
          type: "Identifier", start, end,
          decorators: [], name: "this", optional: false,
          typeAnnotation: f_typeAnnotation,
        };
"#,
        ),
        _ => unreachable!("missing special ESTree decoder for {name}"),
    }
    }
    for field in fields {
        if references_field(&body, &field.name) {
            if name == "TsMethodDefinition" && field.name == "value" {
                let value_index = field
                    .expression
                    .strip_prefix("node(")
                    .and_then(|expression| expression.strip_suffix(')'))
                    .expect("TS method value must be a node");
                writeln!(
                    output,
                    "        const f_value = node({value_index}, \"TSEmptyBodyFunctionExpression\");"
                )
                .unwrap();
                continue;
            }
            writeln!(
                output,
                "        const f_{} = {};",
                field.name, field.expression
            )
            .unwrap();
        }
    }
    output.push_str(&body);
}

fn combined_body_expression(fields: &[DecodedField]) -> String {
    let range = |name| {
        fields
            .iter()
            .find(|field| field.name == name)
            .and_then(|field| field.range.as_deref())
            .unwrap_or_else(|| panic!("missing {name} subrange"))
    };
    format!("nodeArrayPair({}, {})", range("directives"), range("body"))
}

fn references_field(body: &str, field: &str) -> bool {
    let identifier = format!("f_{field}");
    body.match_indices(&identifier).any(|(start, _)| {
        body.as_bytes()
            .get(start + identifier.len())
            .is_none_or(|next| !next.is_ascii_alphanumeric() && *next != b'_' && *next != b'$')
    })
}

const PREAMBLE: &str = r#"// generated by tasks/ast_tools, do not edit

const NULL = -1;
const HEADER_SIZE = 40;
const NATIVE_LITTLE_ENDIAN =
  new Uint8Array(new Int32Array([1]).buffer)[0] === 1;
const utf8Slice = typeof Buffer !== "undefined" &&
  typeof Buffer.prototype.utf8Slice === "function"
  ? Buffer.prototype.utf8Slice
  : (() => {
      const decoder = new TextDecoder("utf-8", { ignoreBOM: true });
      return function(start, end) {
        return decoder.decode(this.subarray(start, end));
      };
    })();
const latin1Slice = typeof Buffer !== "undefined" &&
  typeof Buffer.prototype.latin1Slice === "function"
  ? Buffer.prototype.latin1Slice
  : null;
const fromCharCode = String.fromCharCode;
const shortStringChars = Array(65);
for (let length = 0; length <= 64; length++) {
  shortStringChars[length] = Array(length).fill(0);
}

__ENUMS__
"#;

const DECODE_HELPERS: &str = r#"
function decodePool(bytes) {
  const length = bytes.length;
  if (length <= 64) {
    const chars = shortStringChars[length];
    let i = 0;
    for (; i < length; i++) {
      const byte = bytes[i];
      if (byte >= 0x80) break;
      chars[i] = byte;
    }
    if (i === length) return fromCharCode.apply(null, chars);
  }
  let hasSurrogate = false;
  for (const byte of bytes) {
    if (byte === 0xed) {
      hasSurrogate = true;
      break;
    }
  }
  if (!hasSurrogate) return utf8Slice.call(bytes, 0, length);

  let result = "";
  for (let i = 0; i < bytes.length;) {
    const first = bytes[i];
    if (first < 0x80) {
      result += String.fromCharCode(first);
      i++;
    } else if (first < 0xe0) {
      result += String.fromCharCode(((first & 0x1f) << 6) | (bytes[i + 1] & 0x3f));
      i += 2;
    } else if (first < 0xf0) {
      result += String.fromCharCode(
        ((first & 0x0f) << 12) |
        ((bytes[i + 1] & 0x3f) << 6) |
        (bytes[i + 2] & 0x3f),
      );
      i += 3;
    } else {
      result += String.fromCodePoint(
        ((first & 0x07) << 18) |
        ((bytes[i + 1] & 0x3f) << 12) |
        ((bytes[i + 2] & 0x3f) << 6) |
        (bytes[i + 3] & 0x3f),
      );
      i += 4;
    }
  }
  return result;
}

function copyLittleEndianI32(view) {
  const i32 = new Int32Array(view.byteLength >> 2);
  for (let offset = 0; offset + 4 <= view.byteLength; offset += 4) {
    i32[offset >> 2] = view.getInt32(offset, true);
  }
  return i32;
}
"#;

const DECODE_PREAMBLE: &str = r#"
  const arrayBuffer = buffer instanceof ArrayBuffer ? buffer : buffer.buffer;
  const byteOffset = buffer.byteOffset ?? 0;
  const byteLength = buffer.byteLength;
  if (byteLength > 0x7fffffff) {
    throw new RangeError("AST buffer exceeds signed 32-bit range");
  }
  const bytes = new Uint8Array(arrayBuffer, byteOffset, byteLength);
  const _dv = new DataView(arrayBuffer, byteOffset, byteLength);
  const _i32 = NATIVE_LITTLE_ENDIAN && (byteOffset & 3) === 0
    ? new Int32Array(arrayBuffer, byteOffset, byteLength >> 2)
    : copyLittleEndianI32(_dv);
  const nodeCount = _i32[0];
  const extraCount = _i32[1];
  const stringPoolLength = _dv.getUint32(8, true);
  const sourceLength = _dv.getUint32(12, true);
  if (sourceLength + stringPoolLength > 0x7fffffff) {
    throw new RangeError("AST string offsets exceed signed 32-bit range");
  }
  const commentCount = _i32[4];
  const diagnosticCount = _i32[6];
  const programIndex = _i32[7];
  const flags = _i32[8];
  const firstNonAsciiPos = _i32[9];

  const spans = HEADER_SIZE;
  const kinds = spans + nodeCount * 8;
  const payloads = (kinds + nodeCount + 7) & ~7;
  const extra = payloads + nodeCount * 16;
  const strings = extra + extraCount * 8;
  const comments = strings + ((stringPoolLength + 3) & ~3);
  const diagnostics = comments + commentCount * 16;
  let sourceLatin;
  if (source.length !== sourceLength) {
    sourceLatin = latin1Slice === null
      ? new TextDecoder("latin1").decode(sourceBuffer.subarray(0, sourceLength))
      : latin1Slice.call(sourceBuffer, 0, sourceLength);
  }
  const str = (start, end) => {
    if (start === end) return "";
    if (start < sourceLength) {
      if (end <= firstNonAsciiPos) return source.slice(start, end);
      const length = end - start;
      if (length <= 64) {
        for (let i = 0; i < length; i++) {
          if (sourceBuffer[start + i] >= 0x80) {
            return utf8Slice.call(sourceBuffer, start, end);
          }
        }
        return sourceLatin.slice(start, end);
      }
      return utf8Slice.call(sourceBuffer, start, end);
    }
    return decodePool(bytes.subarray(
      strings + start - sourceLength,
      strings + end - sourceLength,
    ));
  };
  const enumValue = (index, value) => {
    const values = ENUMS[index];
    return values !== undefined && value < values.length ? values[value] : value;
  };
  const sourceType = enumValue("SourceType", flags & 3);
  const optionalEnum = (index, value) =>
    value === 0xff ? null : enumValue(index, value);
  const optionalNode = index => index === NULL ? null : node(index);
  const nodeArray = (start, end, holes) => {
    const result = new Array(end - start);
    for (let i = start; i < end; i++) {
      const index = _i32[(extra >> 2) + i * 2];
      result[i - start] = holes && index === NULL ? null : node(index);
    }
    return result;
  };
  const nodeArrayPair = (firstStart, firstEnd, secondStart, secondEnd) => {
    const result = new Array(firstEnd - firstStart + secondEnd - secondStart);
    let out = 0;
    for (let i = firstStart; i < firstEnd; i++) {
      result[out++] = node(_i32[(extra >> 2) + i * 2]);
    }
    for (let i = secondStart; i < secondEnd; i++) {
      result[out++] = node(_i32[(extra >> 2) + i * 2]);
    }
    return result;
  };

  const _decodeComments = () => {
    const result = new Array(commentCount);
    for (let index = 0; index < commentCount; index++) {
      const offset = comments + index * 16;
      const start = _i32[offset >> 2];
      const end = _i32[(offset >> 2) + 1];
      const byteStart = _i32[(offset >> 2) + 2];
      const byteEnd = _i32[(offset >> 2) + 3];
      const first = sourceBuffer[byteStart];
      const kind = sourceBuffer[byteStart + 1] === 0x2a ? 1 : 0;
      const valueStart = byteStart + (first === 0x3c ? 4 : first === 0x2d ? 3 : 2);
      const valueEnd = byteEnd - (kind === 1 ? 2 : 0);
      result[index] = {
        type: kind === 1 ? "Block" : "Line",
        value: str(valueStart, valueEnd),
        start,
        end,
      };
    }
    return result;
  };

  const _decodeDiagnostics = () => {
    let offset = diagnostics;
    const readU8 = () => bytes[offset++];
    const readU32 = () => {
      const value = _dv.getUint32(offset, true);
      offset += 4;
      return value;
    };
    const readDiagnosticString = () => {
      const length = readU32();
      const value = utf8Slice.call(bytes, offset, offset + length);
      offset += length;
      return value;
    };
    const severities = ["error", "warning", "hint", "info"];
    const result = new Array(diagnosticCount);
    for (let index = 0; index < diagnosticCount; index++) {
      const severityValue = readU8();
      const severity = severities[severityValue];
      if (severity === undefined) {
        throw new RangeError(`Unknown diagnostic severity ${severityValue}`);
      }
      const start = readU32();
      const end = readU32();
      const message = readDiagnosticString();
      const help = readU8() === 0 ? undefined : readDiagnosticString();
      const labelCount = readU32();
      const labels = new Array(labelCount);
      for (let label = 0; label < labelCount; label++) {
        labels[label] = {
          start: readU32(),
          end: readU32(),
          message: readDiagnosticString(),
        };
      }
      result[index] = { severity, message, start, end, help, labels };
    }
    return result;
  };

  // Each reference is decoded into a fresh value, so parents can finish their
  // owned children in place before exposing the result.
"#;

const NODE_PREAMBLE: &str = r#"  function node(index) {
    const span = spans + index * 8;
    const start = _i32[span >> 2];
    const end = _i32[(span >> 2) + 1];
    const payload = payloads + index * 16;
    let result;
    switch (bytes[kinds + index]) {
"#;

const HOT_POSTAMBLE: &str = r#"      default:
        // Retry deep trees without the extra hot-dispatch frames.
        if (coldDepth === 64) throw DEEP_AST;
        coldDepth++;
        try {
          return /*#__NOINLINE__*/ nodeAll(index, typeOverride);
        } finally {
          coldDepth--;
        }
    }
    return result;
  }

"#;

const TS_PROGRAM_GETTER: &str = r#"      if (_program !== undefined) return _program;
      try {
        return (_program = nodeAll(programIndex));
      } catch (error) {
        if (error !== DEEP_AST) throw error;
        node = nodeAll;
        return (_program = nodeAll(programIndex));
      }"#;

const POSTAMBLE: &str = r#"      default:
        throw new RangeError(`Unknown AST node kind ${bytes[kinds + index]}`);
    }
    return result;
  }

  let _program, _comments, _diagnostics;
  return {
    meta: { lang: enumValue("Lang", (flags >> 2) & 7), sourceType },
    get program() {
      return _program !== undefined
        ? _program
        : (_program = node(programIndex));
    },
    get comments() {
      return _comments !== undefined
        ? _comments
        : (_comments = _decodeComments());
    },
    get diagnostics() {
      return _diagnostics !== undefined
        ? _diagnostics
        : (_diagnostics = _decodeDiagnostics());
    },
  };
}
"#;
