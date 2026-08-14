use std::fmt::Write;

use convert_case::{Case, Casing};
use swc_core::{
    common::{FileName, GLOBALS, Globals, Mark, SourceMap, sync::Lrc},
    ecma::{
        ast::{EsVersion, Program},
        codegen::{Config, Emitter, text_writer::JsWriter},
        minifier::{
            optimize,
            option::{ExtraOptions, MinifyOptions},
        },
        parser::{EsSyntax, Parser, StringInput, Syntax, lexer::Lexer},
        transforms::base::{fixer::fixer, resolver},
        visit::VisitMutWith,
    },
};

use crate::{
    output::RawOutput,
    schema::{AstStruct, AstType, Schema},
    util::{InlineStorageMode, calculate_inline_layout},
};

const CORE_PRETTY_OUTPUT_PATH: &str = "napi/core/src-js/decode.pretty.js";
const CORE_MINIFIED_OUTPUT_PATH: &str = "napi/core/src-js/decode.js";
const PARSER_PRETTY_OUTPUT_PATH: &str = "napi/parser/src-js/decode.pretty.js";
const PARSER_MINIFIED_OUTPUT_PATH: &str = "napi/parser/src-js/decode.js";

pub fn estree_decoder(schema: &Schema) -> (RawOutput, RawOutput, RawOutput, RawOutput) {
    let mut output = String::new();
    output.push_str(PREAMBLE);

    for (kind, ty) in schema
        .types
        .iter()
        .filter_map(|ty| match ty {
            AstType::Struct(ast) => Some(ast),
            _ => None,
        })
        .enumerate()
    {
        write_node_case(&mut output, kind, ty, schema);
    }

    output.push_str(POSTAMBLE);
    let minified = minify(&output);
    (
        RawOutput {
            path: CORE_PRETTY_OUTPUT_PATH.into(),
            content: output.as_bytes().to_vec(),
        },
        RawOutput {
            path: CORE_MINIFIED_OUTPUT_PATH.into(),
            content: minified.as_bytes().to_vec(),
        },
        RawOutput {
            path: PARSER_PRETTY_OUTPUT_PATH.into(),
            content: output.into_bytes(),
        },
        RawOutput {
            path: PARSER_MINIFIED_OUTPUT_PATH.into(),
            content: minified.into_bytes(),
        },
    )
}

fn minify(source: &str) -> String {
    GLOBALS.set(&Globals::new(), || {
        let source_map = Lrc::new(SourceMap::default());
        let source_file = source_map.new_source_file(
            FileName::Custom(CORE_PRETTY_OUTPUT_PATH.into()).into(),
            source.to_owned(),
        );
        let lexer = Lexer::new(
            Syntax::Es(EsSyntax::default()),
            EsVersion::EsNext,
            StringInput::from(&*source_file),
            None,
        );
        let mut parser = Parser::new_from(lexer);
        let module = parser
            .parse_module()
            .unwrap_or_else(|error| panic!("failed to parse generated decoder: {error:?}"));
        let recoverable_errors = parser.take_errors();
        assert!(
            recoverable_errors.is_empty(),
            "generated decoder has parse errors: {recoverable_errors:?}"
        );

        let unresolved_mark = Mark::new();
        let top_level_mark = Mark::new();
        let mut program = Program::Module(module);
        program.visit_mut_with(&mut resolver(unresolved_mark, top_level_mark, false));
        let mut program = optimize(
            program,
            source_map.clone(),
            None,
            None,
            &MinifyOptions {
                compress: Some(Default::default()),
                mangle: Some(Default::default()),
                ..Default::default()
            },
            &ExtraOptions {
                unresolved_mark,
                top_level_mark,
                mangle_name_cache: None,
            },
        );
        program.visit_mut_with(&mut fixer(None));

        let mut output = Vec::new();
        Emitter {
            cfg: Config::default()
                .with_minify(true)
                .with_omit_last_semi(true),
            cm: source_map.clone(),
            comments: None,
            wr: JsWriter::new(source_map, "\n", &mut output, None),
        }
        .emit_program(&program)
        .expect("failed to print minified decoder");
        String::from_utf8(output).expect("SWC emitted invalid UTF-8")
    })
}

fn write_node_case(output: &mut String, kind: usize, ast: &AstStruct, schema: &Schema) {
    writeln!(output, "      case {kind}: {{").unwrap();

    let layout = calculate_inline_layout(ast, schema);
    if layout
        .as_ref()
        .is_none_or(|layout| layout.mode == InlineStorageMode::Partial)
    {
        writeln!(output, "        const extraId = _u32[(payload + 12) >> 2];").unwrap();
    }
    let mut fields = Vec::new();

    let mut extra_offset = 0usize;
    for (field_index, field) in ast.fields.iter().enumerate() {
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
        let expression = read_expression(field_ty, schema, &location, aligned_u32);
        let field_name = estree_field_name(&ast.name, &field.name);
        fields.push(DecodedField {
            name: field_name,
            expression,
        });
    }

    if is_special_node(&ast.name) {
        write_special_node(output, &ast.name, &fields);
    } else {
        write_generic_node(output, &ast.name, &fields);
    }
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
        AstType::Vec(vec) => {
            let inner = &schema.types[vec.inner_type_id];
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
                        | "NumericLiteralKind"
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

fn write_generic_node(output: &mut String, name: &str, fields: &[DecodedField]) {
    let has_ts_shape = fields.iter().any(|field| is_ts_field(name, &field.name))
        || !ts_extra_fields(name).is_empty();

    if has_ts_shape {
        writeln!(output, "        if (isTs) {{").unwrap();
        write_object_literal(output, name, fields.iter(), ts_extra_fields(name), 10);
        writeln!(output, "        }} else {{").unwrap();
        write_object_literal(
            output,
            name,
            fields
                .iter()
                .filter(|field| !is_ts_field(name, &field.name)),
            &[],
            10,
        );
        writeln!(output, "        }}").unwrap();
    } else {
        write_object_literal(output, name, fields.iter(), &[], 8);
    }
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

fn write_special_node(output: &mut String, name: &str, fields: &[DecodedField]) {
    for field in fields {
        writeln!(
            output,
            "        const f_{} = {};",
            field.name, field.expression
        )
        .unwrap();
    }

    match name {
        "Program" => output.push_str(
            r#"        const body = [...f_directives, ...f_body];
        result = {
          type: "Program", start: isTs ? (body.length !== 0 ? body[0].start : end) : start, end,
          sourceType,
          hashbang: f_hashbang,
          body,
        };
"#,
        ),
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
        "FormalParameter" => output.push_str("        result = f_pattern;\n"),
        "FormalParameters" => output.push_str(
            "        result = { params: f_rest === null ? f_items : [...f_items, f_rest] };\n",
        ),
        "Function" => output.push_str(
            r#"        result = {
          type: f_functionType, start, end,
          id: f_id,
          generator: f_generator,
          async: f_async,
          params: f_params.params,
          body: f_body,
          expression: false,
        };
        if (isTs) {
          result.typeParameters = f_typeParameters;
          result.returnType = f_returnType;
          result.declare = false;
        }
"#,
        ),
        "TsFunction" => output.push_str(
            r#"        result = {
          type: "TSDeclareFunction",
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
        "ArrowFunctionExpression" => output.push_str(
            r#"        result = {
          type: "ArrowFunctionExpression", start, end,
          id: null, generator: false, async: f_async,
          params: f_params.params,
          body: f_body, expression: f_expression,
        };
        if (isTs) {
          result.typeParameters = f_typeParameters;
          result.returnType = f_returnType;
        }
"#,
        ),
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
            r#"        const raw = f_raw;
        const normalized = raw.includes("_") ? raw.replaceAll("_", "") : raw;
        const value =
          f_kind === "octal" && normalized[1]?.toLowerCase() !== "o"
            ? parseInt(normalized.slice(1), 8)
            : +normalized;
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
        "TemplateElement" => output.push_str(
            r#"        result = {
          type: "TemplateElement",
          start: isTs ? start - 1 : start,
          end: isTs ? (f_tail ? end + 1 : end + 2) : end,
          value: { raw: f_raw, cooked: f_isCookedUndefined ? null : f_cooked },
          tail: f_tail,
        };
"#,
        ),
        "Class" => output.push_str(
            r#"        result = {
          type: f_classType, start, end,
          decorators: f_decorators,
          id: f_id,
          superClass: f_superClass,
          body: f_body,
        };
        if (isTs) {
          result.typeParameters = f_typeParameters;
          result.superTypeArguments = f_superTypeArguments;
          result.implements = f_implements;
          result.abstract = f_abstract;
          result.declare = f_declare;
        }
"#,
        ),
        "MethodDefinition" | "TsMethodDefinition" => {
            if name == "TsMethodDefinition" {
                output.push_str(
                    "        f_value.type = \"TSEmptyBodyFunctionExpression\";\n",
                );
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
        if (isTs) {
          result.override = f_override;
          result.optional = f_optional;
          result.accessibility = f_accessibility;
          if (f_abstract) result.type = "TSAbstractMethodDefinition";
        }
"#,
            );
        }
        "PropertyDefinition" => output.push_str(
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
        if (isTs) {
          result.typeAnnotation = f_typeAnnotation;
          result.declare = f_declare;
          result.override = f_override;
          result.optional = f_optional;
          result.definite = f_definite;
          result.readonly = f_readonly;
          result.accessibility = f_accessibility;
          if (f_abstract) {
            result.type = accessor
              ? "TSAbstractAccessorProperty"
              : "TSAbstractPropertyDefinition";
          }
        }
"#,
        ),
        "UnaryExpression" => output.push_str(
            r#"        result = {
          type: "UnaryExpression", start, end,
          operator: f_operator, prefix: true, argument: f_argument,
        };
"#,
        ),
        "BindingProperty" => output.push_str(
            r#"        result = {
          type: "Property", start, end, kind: "init",
          key: f_key, value: f_value,
          method: false, shorthand: f_shorthand, computed: f_computed,
        };
        if (isTs) result.optional = false;
"#,
        ),
        "ArrayPattern" => output.push_str(
            r#"        const elements = f_rest === null ? f_elements : [...f_elements, f_rest];
        result = { type: "ArrayPattern", start, end, elements };
        if (isTs) {
          result.decorators = f_decorators;
          result.optional = f_optional;
          result.typeAnnotation = f_typeAnnotation;
        }
"#,
        ),
        "ObjectPattern" => output.push_str(
            r#"        const properties = f_rest === null ? f_properties : [...f_properties, f_rest];
        result = { type: "ObjectPattern", start, end, properties };
        if (isTs) {
          result.decorators = f_decorators;
          result.optional = f_optional;
          result.typeAnnotation = f_typeAnnotation;
        }
"#,
        ),
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

fn estree_type(name: &str) -> String {
    match name {
        "BindingRestElement" => "RestElement".into(),
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
const BINDING_REST_TS_EXTRAS: &[(&str, &str)] = &[("value", "null")];
const EXPORT_DEFAULT_TS_EXTRAS: &[(&str, &str)] = &[("exportKind", "\"value\"")];
const TS_PROPERTY_SIGNATURE_EXTRAS: &[(&str, &str)] =
    &[("accessibility", "null"), ("static", "false")];
const TS_INDEX_SIGNATURE_EXTRAS: &[(&str, &str)] = &[("accessibility", "null")];
const TS_PARAMETER_PROPERTY_EXTRAS: &[(&str, &str)] = &[("static", "false")];

fn ts_extra_fields(name: &str) -> &'static [(&'static str, &'static str)] {
    match name {
        "IdentifierReference" | "IdentifierName" | "LabelIdentifier" => IDENTIFIER_TS_EXTRAS,
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
const textDecoder = new TextDecoder("utf-8", { ignoreBOM: true });

const ENUMS = Object.freeze({
  SourceType: ["module", "script", "commonjs"],
  MethodDefinitionKind: ["constructor", "method", "get", "set"],
  Accessibility: [null, "public", "private", "protected"],
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
  NumericLiteralKind: ["decimal", "hex", "octal", "binary"],
  ImportOrExportKind: ["value", "type"],
  ImportPhase: ["source", "defer"],
  TsTypeOperatorKind: ["keyof", "unique", "readonly"],
  TsMethodSignatureKind: ["method", "get", "set"],
  TsModuleDeclarationKind: ["namespace", "module"],
  TsMappedTypeModifier: [null, true, "+", "-"],
});

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

export function decode(buffer, source) {
  const arrayBuffer = buffer instanceof ArrayBuffer ? buffer : buffer.buffer;
  const byteOffset = buffer.byteOffset ?? 0;
  const byteLength = buffer.byteLength;
  const bytes = new Uint8Array(arrayBuffer, byteOffset, byteLength);
  const _u32 = new Uint32Array(arrayBuffer, byteOffset, byteLength >> 2);
  const _dv = new DataView(arrayBuffer, byteOffset, byteLength);
  const nodeCount = _u32[0];
  const extraCount = _u32[1];
  const stringPoolLength = _u32[2];
  const sourceLength = _u32[3];
  const programIndex = _u32[7];
  const isTs = (_u32[8] & 1) !== 0;

  const spans = HEADER_SIZE;
  const kinds = spans + nodeCount * 8;
  const payloads = (kinds + nodeCount + 3) & ~3;
  const extra = payloads + nodeCount * 16;
  const strings = extra + extraCount * 8;
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
  const sourceType = enumValue("SourceType", (_u32[8] >> 1) & 3);
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

  const root = node(programIndex);
  return root;
}
"#;
