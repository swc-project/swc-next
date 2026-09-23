use std::fmt::Write;

use convert_case::{Case, Casing};

use crate::{
    output::RawOutput,
    schema::{AstStruct, AstType, Schema},
    util::calculate_inline_layout,
};

use super::js_minifier::minify;

const OUTPUT_DIR: &str = "packages/decoder";

pub fn estree_decoder(schema: &Schema) -> Vec<RawOutput> {
    let mut outputs = Vec::new();
    for (is_ts, name) in [(false, "decode_js"), (true, "decode_ts")] {
        let pretty_path = format!("{OUTPUT_DIR}/{name}.pretty.js");
        let minified_path = format!("{OUTPUT_DIR}/{name}.js");
        let output = decoder_source(schema, is_ts, name);
        let minified = minify(&output, &pretty_path);
        outputs.push(RawOutput {
            path: pretty_path.into(),
            content: output.into_bytes(),
        });
        outputs.push(RawOutput {
            path: minified_path.into(),
            content: minified.into_bytes(),
        });
    }

    outputs
}

fn decoder_source(schema: &Schema, is_ts: bool, name: &str) -> String {
    let mut output = String::new();
    output.push_str(PREAMBLE);
    if is_ts {
        output.push_str(TS_ENUMS);
    }
    output.push_str(PREAMBLE_END);
    output.push_str(DECODE_HELPERS);
    writeln!(output, "export function {name}(buffer, source) {{").unwrap();
    output.push_str(DECODE_PREAMBLE);

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
        write_node_case(&mut output, kind, ty, schema, is_ts);
    }

    output.push_str(POSTAMBLE);
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

    let layout = calculate_inline_layout(ast, schema);
    let mut fields = Vec::new();

    let mut extra_offset = 0usize;
    for (field_index, field) in ast.fields.iter().enumerate() {
        let skip = field.attrs.estree_skip;
        let field_ty = &schema.types[field.type_id];
        let (location, aligned_u32) = if let Some((_, byte_offset, _)) =
            layout.as_ref().and_then(|layout| {
                layout
                    .fields
                    .iter()
                    .find(|(index, _, _)| *index == field_index)
            }) {
            (format!("payload + {byte_offset}"), byte_offset % 4 == 0)
        } else {
            let location = format!("extra + (extraId + {extra_offset}) * 8");
            extra_offset += 1;
            (location, true)
        };
        if skip {
            continue;
        }
        let expression = read_expression(field_ty, schema, &location, aligned_u32);
        let field_name = estree_field_name(&ast.name, &field.name);
        fields.push(DecodedField {
            name: field_name,
            expression,
        });
    }

    let mut body = String::new();
    if is_special_node(&ast.name) {
        write_special_node(&mut body, &ast.name, &fields, is_ts);
    } else {
        write_generic_node(&mut body, &ast.name, &fields, is_ts);
    }
    if body.contains("extraId") {
        writeln!(output, "        const extraId = _u32[(payload + 12) >> 2];").unwrap();
    }
    output.push_str(&body);
    writeln!(output, "        break;").unwrap();
    writeln!(output, "      }}").unwrap();
}

struct DecodedField {
    name: String,
    expression: String,
}

fn read_expression(ty: &AstType, schema: &Schema, location: &str, aligned_u32: bool) -> String {
    match ty {
        AstType::Struct(_) | AstType::Enum(_) => {
            format!("node({})", read_u32(location, aligned_u32))
        }
        AstType::SubRange(range) => {
            let inner = &schema.types[range.inner_type_id];
            let holes = matches!(inner, AstType::Option(_));
            let start = read_u32(location, aligned_u32);
            let end = read_u32(&format!("{location} + 4"), aligned_u32);
            format!("nodeArray({start}, {end}, {holes})")
        }
        AstType::Option(option) => match &schema.types[option.inner_type_id] {
            AstType::Struct(_) | AstType::Enum(_) => {
                format!("optionalNode({})", read_u32(location, aligned_u32))
            }
            AstType::Primitive(primitive) if primitive.name == "ImportPhase" => {
                format!("optionalEnum(\"ImportPhase\", _dv.getUint8({location}))")
            }
            AstType::Primitive(primitive) if matches!(primitive.name, "ScopeId" | "SymbolId") => {
                "undefined".into()
            }
            AstType::Primitive(primitive) if primitive.name == "Span" => {
                let start = read_u32(location, aligned_u32);
                let end = read_u32(&format!("{location} + 4"), aligned_u32);
                format!("{{ start: pos({start}), end: pos({end}) }}")
            }
            AstType::Primitive(primitive) if primitive.name == "AstUtf8" => {
                let start = read_u32(location, aligned_u32);
                let end = read_u32(&format!("{location} + 4"), aligned_u32);
                format!("{start} === NULL && {end} === 0 ? null : str({start}, {end})")
            }
            inner => panic!("unsupported optional decoder type: {}", inner.name()),
        },
        AstType::Primitive(primitive) => match primitive.name {
            "AstUtf8" | "AstWtf8" => {
                let start = read_u32(location, aligned_u32);
                let end = read_u32(&format!("{location} + 4"), aligned_u32);
                format!("str({start}, {end})")
            }
            "bool" => format!("_dv.getUint8({location}) !== 0"),
            "u8" => format!("_dv.getUint8({location})"),
            "i8" => format!("_dv.getInt8({location})"),
            "u16" => format!("_dv.getUint16({location}, true)"),
            "i16" => format!("_dv.getInt16({location}, true)"),
            "u32" => read_u32(location, aligned_u32),
            "i32" => format!("_dv.getInt32({location}, true)"),
            "f64" => format!("_dv.getFloat64({location}, true)"),
            "Span" => {
                let start = read_u32(location, aligned_u32);
                let end = read_u32(&format!("{location} + 4"), aligned_u32);
                format!("{{ start: pos({start}), end: pos({end}) }}")
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

fn read_u32(location: &str, aligned: bool) -> String {
    if aligned {
        format!("_u32[({location}) >> 2]")
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
            output.push_str("        const body = [...f_directives, ...f_body];\n");
            output.push_str(if is_ts {
                "        result = { type: \"Program\", start: body.length !== 0 ? body[0].start : end, end, sourceType, hashbang: f_hashbang, body };\n"
            } else {
                "        result = { type: \"Program\", start, end, sourceType, hashbang: f_hashbang, body };\n"
            });
        }
        "FunctionBody" | "TsModuleBlock" => {
            writeln!(
                output,
                "        result = {{ type: \"{}\", start, end, body: [...f_directives, ...f_body] }};",
                if name == "TsModuleBlock" {
                    "TSModuleBlock"
                } else {
                    "BlockStatement"
                }
            )
            .unwrap();
        }
        "FormalParameter" => {
            output.push_str("        const left = { ...f_pattern };\n");
            if is_ts {
                output.push_str("        left.optional = f_optional;\n        left.typeAnnotation = f_typeAnnotation;\n        left.end = f_typeAnnotation?.end ?? (f_optional ? left.end + 1 : left.end);\n");
            }
            output.push_str("        if (f_initializer === null) {\n          result = left;\n          result.end = end;\n");
            if is_ts {
                output.push_str("          result.decorators = f_decorators;\n");
            }
            output.push_str("        } else {\n          result = { type: \"AssignmentPattern\", start: left.start, end, left, right: f_initializer };\n");
            if is_ts {
                output.push_str("          result.decorators = f_decorators;\n          result.optional = false;\n          result.typeAnnotation = null;\n");
            }
            output.push_str("        }\n");
        }
        "FormalParameterRest" => {
            output.push_str(
                "        result = { type: \"RestElement\", start, end, argument: f_argument };\n",
            );
            if is_ts {
                output.push_str("        result.decorators = f_decorators;\n        result.optional = false;\n        result.typeAnnotation = f_typeAnnotation;\n        result.value = null;\n");
            }
        }
        "VariableDeclarator" => {
            output.push_str("        result = { type: \"VariableDeclarator\", start, end, id: f_id, init: f_init };\n");
            if is_ts {
                output.push_str("        result.definite = f_definite;\n        result.id = { ...f_id, typeAnnotation: f_typeAnnotation, end: f_typeAnnotation?.end ?? f_id.end };\n");
            }
        }
        "CatchClause" => {
            output.push_str("        result = { type: \"CatchClause\", start, end, param: f_param, body: f_body };\n");
            if is_ts {
                output.push_str("        if (f_param !== null) result.param = { ...f_param, typeAnnotation: f_typeAnnotation, end: f_typeAnnotation?.end ?? f_param.end };\n");
            }
        }
        "TsIndexSignatureParameter" => output.push_str(
            r#"        result = { ...f_name, start, end, typeAnnotation: f_typeAnnotation };
"#,
        ),
        "FormalParameters" => output.push_str(
            "        result = { params: f_rest === null ? f_items : [...f_items, f_rest] };\n",
        ),
        "Function" => {
            output.push_str(
                r#"        result = {
          type: f_functionType, start, end,
          id: f_id,
          generator: f_generator,
          async: f_async,
"#,
            );
            output.push_str(if is_ts {
                "          params: f_thisParam === null ? f_params.params : [f_thisParam, ...f_params.params],\n"
            } else {
                "          params: f_params.params,\n"
            });
            output.push_str(
                r#"          body: f_body,
          expression: false,
        };
"#,
            );
            if is_ts {
                output.push_str("        result.typeParameters = f_typeParameters;\n        result.returnType = f_returnType;\n        result.declare = false;\n");
            }
        }
        "TsFunction" => output.push_str(
            r#"        result = {
          type: "TSDeclareFunction",
          start, end,
          id: f_id,
          generator: f_generator,
          async: f_async,
          params: f_thisParam === null ? f_params.params : [f_thisParam, ...f_params.params],
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
        };
"#,
            );
            if is_ts {
                output.push_str("        result.typeParameters = f_typeParameters;\n        result.returnType = f_returnType;\n");
            }
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
            r#"        const raw = source.slice(start, end);
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
        };
"#,
            );
            if is_ts {
                output.push_str("        result.typeParameters = f_typeParameters;\n        result.superTypeArguments = f_superTypeArguments;\n        result.implements = f_implements;\n        result.abstract = f_abstract;\n        result.declare = f_declare;\n");
            }
        }
        "MethodDefinition" | "TsMethodDefinition" => {
            if name == "TsMethodDefinition" {
                output.push_str("        f_value.type = \"TSEmptyBodyFunctionExpression\";\n");
            }
            writeln!(
                output,
                "        result = {{\n          type: {},",
                if name == "TsMethodDefinition" {
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
        };
"#,
            );
            if is_ts {
                output.push_str("        result.override = f_override;\n        result.optional = f_optional;\n        result.accessibility = f_accessibility;\n        if (f_abstract) result.type = \"TSAbstractMethodDefinition\";\n");
            }
        }
        "PropertyDefinition" => {
            output.push_str(
                r#"        const accessor = f_accessor;
        result = {
          type: accessor ? "AccessorProperty" : "PropertyDefinition",
          start, end,
          decorators: f_decorators,
          key: f_key,
          value: f_value,
          computed: f_computed,
          static: f_static,
        };
"#,
            );
            if is_ts {
                output.push_str("        result.typeAnnotation = f_typeAnnotation;\n        result.declare = f_declare;\n        result.override = f_override;\n        result.optional = f_optional;\n        result.definite = f_definite;\n        result.readonly = f_readonly;\n        result.accessibility = f_accessibility;\n        if (f_abstract) result.type = accessor ? \"TSAbstractAccessorProperty\" : \"TSAbstractPropertyDefinition\";\n");
            }
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
        };
"#,
            );
            if is_ts {
                output.push_str("        result.optional = false;\n");
            }
        }
        "ArrayPattern" => {
            output.push_str(
                r#"        const elements = f_rest === null ? f_elements : [...f_elements, f_rest];
        result = { type: "ArrayPattern", start, end, elements };
"#,
            );
            if is_ts {
                output.push_str("        result.decorators = [];\n        result.optional = false;\n        result.typeAnnotation = null;\n");
            }
        }
        "ObjectPattern" => {
            output.push_str(r#"        const properties = f_rest === null ? f_properties : [...f_properties, f_rest];
        result = { type: "ObjectPattern", start, end, properties };
"#);
            if is_ts {
                output.push_str("        result.decorators = [];\n        result.optional = false;\n        result.typeAnnotation = null;\n");
            }
        }
        "JsxText" => output.push_str(
            "        result = { type: \"JSXText\", start, end, value: f_value, raw: f_value };\n",
        ),
        "TsFunctionType"
        | "TsConstructorType"
        | "TsMethodSignature"
        | "TsCallSignatureDeclaration"
        | "TsConstructSignatureDeclaration" => {
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
        };
"#,
            );
            if matches!(
                name,
                "TsFunctionType" | "TsMethodSignature" | "TsCallSignatureDeclaration"
            ) {
                output.push_str(
                    "        if (f_thisParam !== null) result.params.unshift(f_thisParam);\n",
                );
            }
            if name == "TsMethodSignature" {
                output.push_str(
                    r#"        result.accessibility = null;
        result.readonly = false;
        result.static = false;
"#,
                );
            }
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
            r#"        result = {
          type: "TSModuleDeclaration", start, end,
          id: f_id, kind: f_kind, declare: f_declare, global: false,
        };
        if (f_body !== null) result.body = f_body;
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

fn references_field(body: &str, field: &str) -> bool {
    let identifier = format!("f_{field}");
    body.match_indices(&identifier).any(|(start, _)| {
        body.as_bytes()
            .get(start + identifier.len())
            .is_none_or(|next| !next.is_ascii_alphanumeric() && *next != b'_' && *next != b'$')
    })
}

fn estree_type(name: &str) -> String {
    match name {
        "BindingRestElement" | "FormalParameterRest" => "RestElement".into(),
        "ObjectProperty" => "Property".into(),
        "IdentifierReference" | "BindingIdentifier" | "IdentifierName" | "LabelIdentifier" => {
            "Identifier".into()
        }
        "TsBigIntKeyword" => "TSBigIntKeyword".into(),
        "TsJsDocNullableType" => "TSJSDocNullableType".into(),
        "TsJsDocNonNullableType" => "TSJSDocNonNullableType".into(),
        "TsJsDocUnknownType" => "TSJSDocUnknownType".into(),
        _ if name.starts_with("Jsx") => format!("JSX{}", &name[3..]),
        _ if name.starts_with("Ts") => format!("TS{}", &name[2..]),
        _ => name.into(),
    }
}

fn is_ts_field(node: &str, field: &str) -> bool {
    matches!(
        (node, field),
        ("VariableDeclaration", "declare")
            | ("VariableDeclarator", "definite")
            | ("CallExpression", "typeArguments")
            | ("NewExpression", "typeArguments")
            | ("TaggedTemplateExpression", "typeArguments")
            | ("JsxOpeningElement", "typeArguments")
            | ("ImportDeclaration", "importKind")
            | ("ImportSpecifier", "importKind")
            | ("ExportNamedDeclaration", "exportKind")
            | ("ExportAllDeclaration", "exportKind")
            | ("ExportSpecifier", "exportKind")
            | (
                "BindingRestElement",
                "decorators" | "optional" | "typeAnnotation"
            )
            | (
                "BindingIdentifier",
                "decorators" | "optional" | "typeAnnotation"
            )
            | (
                "AssignmentPattern",
                "decorators" | "optional" | "typeAnnotation"
            )
            | (
                "ObjectPattern",
                "decorators" | "optional" | "typeAnnotation"
            )
            | ("ArrayPattern", "decorators" | "optional" | "typeAnnotation")
            | ("TsIndexSignature", "static")
    )
}

const IDENTIFIER_TS_EXTRAS: &[(&str, &str)] = &[
    ("decorators", "[]"),
    ("optional", "false"),
    ("typeAnnotation", "null"),
];
const OBJECT_PROPERTY_TS_EXTRAS: &[(&str, &str)] = &[("optional", "false")];
const EXPRESSION_STATEMENT_TS_EXTRAS: &[(&str, &str)] = &[("directive", "null")];
const BINDING_REST_TS_EXTRAS: &[(&str, &str)] = &[
    ("value", "null"),
    ("decorators", "[]"),
    ("optional", "false"),
    ("typeAnnotation", "null"),
];
const EXPORT_DEFAULT_TS_EXTRAS: &[(&str, &str)] = &[("exportKind", "\"value\"")];
const TS_PROPERTY_SIGNATURE_EXTRAS: &[(&str, &str)] =
    &[("accessibility", "null"), ("static", "false")];
const TS_INDEX_SIGNATURE_EXTRAS: &[(&str, &str)] = &[("accessibility", "null")];
const TS_PARAMETER_PROPERTY_EXTRAS: &[(&str, &str)] = &[("static", "false")];

fn ts_extra_fields(name: &str) -> &'static [(&'static str, &'static str)] {
    match name {
        "IdentifierReference"
        | "IdentifierName"
        | "LabelIdentifier"
        | "BindingIdentifier"
        | "AssignmentPattern" => IDENTIFIER_TS_EXTRAS,
        "ObjectProperty" => OBJECT_PROPERTY_TS_EXTRAS,
        "ExpressionStatement" => EXPRESSION_STATEMENT_TS_EXTRAS,
        "BindingRestElement" => BINDING_REST_TS_EXTRAS,
        "ExportDefaultDeclaration" => EXPORT_DEFAULT_TS_EXTRAS,
        "TsPropertySignature" => TS_PROPERTY_SIGNATURE_EXTRAS,
        "TsIndexSignature" => TS_INDEX_SIGNATURE_EXTRAS,
        "TsParameterProperty" => TS_PARAMETER_PROPERTY_EXTRAS,
        _ => &[],
    }
}

fn estree_field_name(node: &str, field: &str) -> String {
    let field = field.strip_prefix("r#").unwrap_or(field);
    if node == "VariableDeclaration" && field == "declarators" {
        return "declarations".into();
    }
    if node == "TsEnumDeclaration" && field == "is_const" {
        return "const".into();
    }
    field.to_case(Case::Camel)
}

const PREAMBLE: &str = r#"// generated by tasks/ast_tools, do not edit

const NULL = 0xffffffff;
const HEADER_SIZE = 36;
const NATIVE_LITTLE_ENDIAN =
  new Uint8Array(new Uint32Array([1]).buffer)[0] === 1;
const textDecoder = new TextDecoder("utf-8", { ignoreBOM: true });

const ENUMS = Object.freeze({
  SourceType: ["module", "script", "commonjs"],
  MethodDefinitionKind: ["constructor", "method", "get", "set"],
  ClassType: ["ClassDeclaration", "ClassExpression"],
  BinaryOperator: ["==", "!=", "===", "!==", "<", "<=", ">", ">=", "+", "-", "*", "/", "%", "**", "|", "^", "&", "<<", ">>", ">>>", "in", "instanceof"],
  LogicalOperator: ["&&", "||", "??"],
  UnaryOperator: ["-", "+", "!", "~", "typeof", "void", "delete"],
  UpdateOperator: ["++", "--"],
  AssignmentOperator: ["=", "+=", "-=", "*=", "/=", "%=", "**=", "<<=", ">>=", ">>>=", "|=", "^=", "&=", "||=", "&&=", "??="],
  VariableKind: ["var", "let", "const", "using", "await using"],
  FunctionType: ["FunctionDeclaration", "FunctionExpression"],
  FormalParameterKind: ["formal", "unique", "arrow", "signature"],
  PropertyKind: ["init", "get", "set"],
  ImportPhase: ["source", "defer"],
"#;

const TS_ENUMS: &str = r#"  Accessibility: [null, "public", "private", "protected"],
  ImportOrExportKind: ["value", "type"],
  TsTypeOperatorKind: ["keyof", "unique", "readonly"],
  TsMethodSignatureKind: ["method", "get", "set"],
  TsModuleDeclarationKind: ["namespace", "module"],
  TsMappedTypeModifier: [null, true, "+", "-"],
"#;

const PREAMBLE_END: &str = r#"
});
"#;

const DECODE_HELPERS: &str = r#"
function buildPosMap(source, byteLength) {
  const map = new Uint32Array(byteLength + 1);
  let byte = 0;
  for (let utf16 = 0; utf16 < source.length; utf16++) {
    const first = source.charCodeAt(utf16);
    const isPair =
      first >= 0xd800 &&
      first <= 0xdbff &&
      utf16 + 1 < source.length &&
      source.charCodeAt(utf16 + 1) >= 0xdc00 &&
      source.charCodeAt(utf16 + 1) <= 0xdfff;
    const width =
      first < 0x80 ? 1 :
      first < 0x800 ? 2 :
      isPair ? 4 : 3;
    for (let i = 0; i < width; i++) map[byte + i] = utf16;
    byte += width;
    if (width === 4) {
      map[byte - 1] = utf16 + 1;
      utf16++;
    }
  }
  map[byteLength] = source.length;
  return map;
}

function decodePool(bytes) {
  let hasSurrogate = false;
  for (const byte of bytes) {
    if (byte === 0xed) {
      hasSurrogate = true;
      break;
    }
  }
  if (!hasSurrogate) return textDecoder.decode(bytes);

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

function copyLittleEndianU32(view) {
  const u32 = new Uint32Array(view.byteLength >> 2);
  for (let offset = 0; offset + 4 <= view.byteLength; offset += 4) {
    u32[offset >> 2] = view.getUint32(offset, true);
  }
  return u32;
}
"#;

const DECODE_PREAMBLE: &str = r#"
  const arrayBuffer = buffer instanceof ArrayBuffer ? buffer : buffer.buffer;
  const byteOffset = buffer.byteOffset ?? 0;
  const byteLength = buffer.byteLength;
  const bytes = new Uint8Array(arrayBuffer, byteOffset, byteLength);
  const _dv = new DataView(arrayBuffer, byteOffset, byteLength);
  const _u32 = NATIVE_LITTLE_ENDIAN && (byteOffset & 3) === 0
    ? new Uint32Array(arrayBuffer, byteOffset, byteLength >> 2)
    : copyLittleEndianU32(_dv);
  const nodeCount = _u32[0];
  const extraCount = _u32[1];
  const stringPoolLength = _u32[2];
  const sourceLength = _u32[3];
  const commentCount = _u32[4];
  const diagnosticCount = _u32[6];
  const programIndex = _u32[7];
  const flags = _u32[8];

  const spans = HEADER_SIZE;
  const kinds = spans + nodeCount * 8;
  const payloads = (kinds + nodeCount + 3) & ~3;
  const extra = payloads + nodeCount * 16;
  const strings = extra + extraCount * 8;
  const comments = strings + ((stringPoolLength + 3) & ~3);
  const diagnostics = comments + commentCount * 16;
  const positions =
    source.length === sourceLength ? null : buildPosMap(source, sourceLength);

  const pos = positions === null ? byte => byte : byte => positions[byte];
  const str = (start, end) => {
    if (start === end) return "";
    if (start < sourceLength) return source.slice(pos(start), pos(end));
    return decodePool(bytes.subarray(
      strings + start - sourceLength,
      strings + end - sourceLength,
    ));
  };
  const enumValue = (name, value) => {
    const values = ENUMS[name];
    return values !== undefined && value < values.length ? values[value] : value;
  };
  const sourceType = enumValue("SourceType", (flags >> 1) & 3);
  const optionalEnum = (name, value) =>
    value === 0xff ? null : enumValue(name, value);
  const optionalNode = index => index === NULL ? null : node(index);
  const nodeArray = (start, end, holes) => {
    const result = new Array(end - start);
    for (let i = start; i < end; i++) {
      const index = _u32[(extra >> 2) + i * 2];
      result[i - start] = holes && index === NULL ? null : node(index);
    }
    return result;
  };

  const _decodeComments = () => {
    const result = new Array(commentCount);
    for (let index = 0; index < commentCount; index++) {
      const offset = comments + index * 16;
      const start = _u32[offset >> 2];
      const end = _u32[(offset >> 2) + 1];
      const kind = _dv.getUint16(offset + 12, true);
      const first = source.charCodeAt(pos(start));
      const valueStart = start + (first === 0x3c ? 4 : first === 0x2d ? 3 : 2);
      const valueEnd = end - (kind === 1 ? 2 : 0);
      result[index] = {
        type: kind === 1 ? "Block" : "Line",
        value: str(valueStart, valueEnd),
        start: pos(start),
        end: pos(end),
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
      const value = textDecoder.decode(bytes.subarray(offset, offset + length));
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
      const start = pos(readU32());
      const end = pos(readU32());
      const message = readDiagnosticString();
      const help = readU8() === 0 ? undefined : readDiagnosticString();
      const labelCount = readU32();
      const labels = new Array(labelCount);
      for (let label = 0; label < labelCount; label++) {
        labels[label] = {
          start: pos(readU32()),
          end: pos(readU32()),
          message: readDiagnosticString(),
        };
      }
      result[index] = { severity, message, start, end, help, labels };
    }
    return result;
  };

  function node(index) {
    const span = spans + index * 8;
    const start = pos(_u32[span >> 2]);
    const end = pos(_u32[(span >> 2) + 1]);
    const payload = payloads + index * 16;
    let result;
    switch (bytes[kinds + index]) {
"#;

const POSTAMBLE: &str = r#"      default:
        throw new RangeError(`Unknown AST node kind ${bytes[kinds + index]}`);
    }
    return result;
  }

  let _program, _comments, _diagnostics;
  return {
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
