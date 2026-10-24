use std::{
    collections::{HashMap, HashSet},
    fmt::Write,
};

use convert_case::{Case, Casing};

use crate::{
    output::RawOutput,
    schema::{AstStruct, AstType, FieldLocation, Schema, TypeId},
};

use super::js_minifier::minify_encoder;

const OUTPUT_DIR: &str = "packages/encoder";

// Switch hit frequencies from the 25 codec benchmark fixtures, normalized per
// fixture and combined across JS/TS. Unobserved cases follow the profiled cases.
const ESTREE_TYPE_ORDER: &[&str] = &[
    "Identifier",
    "MemberExpression",
    "Literal",
    "CallExpression",
    "VariableDeclaration",
    "ExpressionStatement",
    "TSTypeReference",
    "BinaryExpression",
    "Property",
    "AssignmentExpression",
    "BlockStatement",
    "JSXIdentifier",
    "IfStatement",
    "LogicalExpression",
    "ReturnStatement",
    "ThisExpression",
    "ObjectExpression",
    "FunctionDeclaration",
    "JSXAttribute",
    "UnaryExpression",
    "JSXText",
    "ParenthesizedExpression",
    "FunctionExpression",
    "ImportSpecifier",
    "ConditionalExpression",
    "JSXElement",
    "JSXExpressionContainer",
    "ArrowFunctionExpression",
    "ImportDeclaration",
    "ArrayExpression",
    "TemplateLiteral",
    "ObjectPattern",
    "TSAsExpression",
    "ChainExpression",
    "NewExpression",
    "ImportDefaultSpecifier",
    "UpdateExpression",
    "TSStringKeyword",
    "TSPropertySignature",
    "MethodDefinition",
    "AwaitExpression",
    "TSUnionType",
    "PrivateIdentifier",
    "ExportNamedDeclaration",
    "TSAnyKeyword",
    "TSIntersectionType",
    "TSTupleType",
    "TSLiteralType",
    "SpreadElement",
    "ForStatement",
    "TSIndexedAccessType",
    "TSVoidKeyword",
    "BreakStatement",
    "TSNonNullExpression",
    "TSTypeLiteral",
    "TSTypeAliasDeclaration",
    "TSUndefinedKeyword",
    "ThrowStatement",
    "TSBooleanKeyword",
    "SequenceExpression",
    "TSNumberKeyword",
    "PropertyDefinition",
    "WhileStatement",
    "TSArrayType",
    "TSTypeQuery",
    "TSFunctionType",
    "TSInterfaceDeclaration",
    "TSNullKeyword",
    "TSMethodSignature",
    "TSConditionalType",
    "TSQualifiedName",
    "TSImportType",
    "ForOfStatement",
    "TaggedTemplateExpression",
    "SwitchStatement",
    "TryStatement",
    "ArrayPattern",
    "ClassDeclaration",
    "JSXMemberExpression",
    "ForInStatement",
    "AssignmentPattern",
    "TSCallSignatureDeclaration",
    "TSNeverKeyword",
    "ExportDefaultDeclaration",
    "TSUnknownKeyword",
    "ContinueStatement",
    "JSXFragment",
    "TSEnumDeclaration",
    "TSIndexSignature",
    "TSInferType",
    "TSTypeOperator",
    "JSXSpreadAttribute",
    "TSTemplateLiteralType",
    "ImportNamespaceSpecifier",
    "TSParenthesizedType",
    "ImportExpression",
    "TSInstantiationExpression",
    "TSMappedType",
    "Super",
    "DoWhileStatement",
    "TSModuleDeclaration",
    "EmptyStatement",
    "JSXEmptyExpression",
    "TSNamedTupleMember",
    "LabeledStatement",
    "TSDeclareFunction",
    "TSTypePredicate",
    "TSAbstractMethodDefinition",
    "YieldExpression",
    "TSRestType",
    "TSObjectKeyword",
    "MetaProperty",
    "TSAbstractPropertyDefinition",
    "TSSymbolKeyword",
    "TSConstructorType",
    "TSSatisfiesExpression",
    "ClassExpression",
    "TSModuleBlock",
    "DebuggerStatement",
    "WithStatement",
    "ExportAllDeclaration",
    "RestElement",
    "AccessorProperty",
    "StaticBlock",
    "JSXNamespacedName",
    "JSXSpreadChild",
    "TSExportAssignment",
    "TSNamespaceExportDeclaration",
    "TSImportEqualsDeclaration",
    "TSTypeAssertion",
    "TSParameterProperty",
    "TSAbstractAccessorProperty",
    "TSBigIntKeyword",
    "TSIntrinsicKeyword",
    "TSThisType",
    "TSOptionalType",
    "TSJSDocNullableType",
    "TSJSDocNonNullableType",
    "TSJSDocUnknownType",
    "TSConstructSignatureDeclaration",
    "TSExternalModuleReference",
];

pub fn estree_encoder(schema: &Schema) -> Vec<RawOutput> {
    let mut outputs = Vec::new();
    for (is_ts, suffix, name) in [(false, "js", "encodeJs"), (true, "ts", "encodeTs")] {
        let pretty_path = format!("{OUTPUT_DIR}/encode-{suffix}.pretty.js");
        let minified_path = format!("{OUTPUT_DIR}/encode-{suffix}.js");
        let output = encoder_source(schema, is_ts, name);
        let minified = minify_encoder(&output, &pretty_path);
        outputs.push(RawOutput {
            path: pretty_path,
            content: output.into_bytes(),
        });
        outputs.push(RawOutput {
            path: minified_path,
            content: minified.into_bytes(),
        });
    }
    outputs.push(RawOutput {
        path: format!("{OUTPUT_DIR}/index.js"),
        content: INDEX.as_bytes().to_vec(),
    });
    outputs
}

fn encoder_source(schema: &Schema, is_ts: bool, name: &str) -> String {
    let mut output = PREAMBLE.replace(
        "function encode(program, meta = {})",
        &format!("function {name}(program, meta = {{}})"),
    );
    if !is_ts {
        output = output.replace("let isTs = false;", "const isTs = false;");
    }
    let dispatch = EncoderDispatch::new(schema, is_ts);
    for (kind, ast) in schema
        .types
        .iter()
        .filter_map(|ty| match ty {
            AstType::Struct(ast) => Some(ast),
            _ => None,
        })
        .enumerate()
    {
        if !is_ts && ast.name.starts_with("Ts") {
            continue;
        }
        write_struct_encoder(&mut output, kind, ast, schema, &dispatch);
    }
    write_enum_encoders(&mut output, &dispatch);
    output.push_str(&POSTAMBLE.replace("export { encode };", &format!("export {{ {name} }};")));
    output
}

const INDEX: &str = r#"// generated by tasks/ast_tools, do not edit
import { encodeJs } from "./encode-js.js";
import { encodeTs } from "./encode-ts.js";

export { encodeJs, encodeTs };
export { encodeOptions } from "./encode-ts.js";

export function encode(program, meta) {
  return meta?.lang === "js" || meta?.lang === "jsx"
    ? encodeJs(program, meta)
    : encodeTs(program, meta);
}
"#;

fn write_struct_encoder(
    output: &mut String,
    kind: usize,
    ast: &AstStruct,
    schema: &Schema,
    dispatch: &EncoderDispatch,
) {
    let name = &ast.name;
    let parameters = if name == "FormalParameters" {
        "n, from = 0"
    } else {
        "n"
    };
    writeln!(output, "  function enc{name}({parameters}) {{").unwrap();
    if name.starts_with("Ts") {
        output.push_str("    isTs = true;\n");
    }
    write_special_prelude(output, name, dispatch.is_ts);

    let layout = &ast.layout;
    let mut fields = Vec::with_capacity(ast.fields.len());

    for (field_index, field) in ast.fields.iter().enumerate() {
        let ty = &schema.types[field.type_id];
        let source = field_source(name, &field.name);
        let source = if !dispatch.is_ts && name == "PropertyDefinition" && field.name == "accessor"
        {
            "n.type === \"AccessorProperty\"".into()
        } else if field.attrs.estree_skip {
            "0".into()
        } else {
            source
        };
        let value = if !dispatch.is_ts && is_ts_field(name, &field.name, ty, schema) {
            match ty {
                AstType::Option(_) => "NULL".into(),
                AstType::SubRange(_) => "0".into(),
                _ => "0".into(),
            }
        } else {
            encode_expression(name, &field.name, ty, schema, dispatch, &source)
        };
        let local = format!("v_{field_index}");
        writeln!(output, "    const {local} = {value};").unwrap();

        let source = if !dispatch.is_ts && is_ts_field(name, &field.name, ty, schema) {
            "null".into()
        } else {
            source
        };
        fields.push((
            ty,
            local,
            source,
            range_length(name, &field.name),
            layout.fields[field_index],
        ));
    }

    if layout.extra_count != 0 {
        writeln!(
            output,
            "    const extraId = allocExtra({});",
            layout.extra_count
        )
        .unwrap();
    }
    let (start, end) = span_expression(name);
    writeln!(
        output,
        "    const index = allocNode({kind}, {start}, {end});"
    )
    .unwrap();
    if let Some(offset) = layout.extra_start_offset() {
        writeln!(output, "    setPayloadU32(index, {offset}, extraId);").unwrap();
    }
    for (ty, local, source, length, location) in fields {
        write_value(output, ty, schema, &local, &source, length, location);
    }
    output.push_str("    return index;\n  }\n\n");
}

fn write_value(
    output: &mut String,
    ty: &AstType,
    schema: &Schema,
    value: &str,
    source: &str,
    length: Option<&str>,
    location: FieldLocation,
) {
    let setter = |kind: &str, byte_offset: usize, component: &str| match location {
        FieldLocation::Payload { offset, .. } => {
            format!(
                "setPayload{kind}(index, {}, {component});",
                offset + byte_offset
            )
        }
        FieldLocation::Extra { index } => {
            format!("setExtra{kind}(extraId + {index}, {byte_offset}, {component});")
        }
    };

    if let Some((string, optional)) = string_type(ty, schema) {
        let encoder = "encStr";
        let wtf8 = if string == "AstWtf8" { ", true" } else { "" };
        let destination = buffer_destination(location);
        if optional {
            writeln!(output, "    if ({value} == null) {{").unwrap();
            writeln!(output, "      {}", setter("U32", 0, "NULL")).unwrap();
            writeln!(output, "      {}", setter("U32", 4, "0")).unwrap();
            writeln!(output, "    }} else {{").unwrap();
            writeln!(output, "      {encoder}({value}, {destination}{wtf8});").unwrap();
            output.push_str("    }\n");
        } else {
            writeln!(output, "    {encoder}({value}, {destination}{wtf8});").unwrap();
        }
        return;
    }

    match ty {
        AstType::Struct(_) | AstType::Enum(_) => {
            writeln!(output, "    {}", setter("U32", 0, value)).unwrap();
        }
        AstType::SubRange(_) => {
            let destination = buffer_destination(location);
            if let Some(length) = length {
                writeln!(output, "    writeRange({value}, {length}, {destination});").unwrap();
            } else {
                writeln!(
                    output,
                    "    writeArrayRange({value}, {source}, {destination});"
                )
                .unwrap();
            }
        }

        AstType::Option(option) => match &schema.types[option.inner_type_id] {
            AstType::Struct(_) | AstType::Enum(_) => {
                writeln!(output, "    {}", setter("U32", 0, value)).unwrap();
            }
            AstType::Primitive(primitive) if primitive.name == "Span" => {
                writeln!(output, "    {}", setter("U32", 0, &format!("{value}[0]"))).unwrap();
                writeln!(output, "    {}", setter("U32", 4, &format!("{value}[1]"))).unwrap();
            }
            AstType::Primitive(primitive) => {
                let kind = primitive_setter(primitive.name, schema);
                writeln!(output, "    {}", setter(kind, 0, value)).unwrap();
            }
            inner => panic!("unsupported optional encoder type: {}", inner.name()),
        },
        AstType::Primitive(primitive) => match primitive.name {
            "Span" => {
                writeln!(output, "    {}", setter("U32", 0, &format!("{value}[0]"))).unwrap();
                writeln!(output, "    {}", setter("U32", 4, &format!("{value}[1]"))).unwrap();
            }
            name => {
                let kind = primitive_setter(name, schema);
                writeln!(output, "    {}", setter(kind, 0, value)).unwrap();
            }
        },
    }
}

fn range_length(node: &str, field: &str) -> Option<&'static str> {
    match (node, field) {
        ("Program" | "FunctionBody" | "TsModuleBlock", "directives") => Some("_directiveCount"),
        ("Program" | "FunctionBody" | "TsModuleBlock", "body") => Some("_length - _directiveCount"),
        ("FormalParameters", "items")
        | ("ArrayPattern", "elements")
        | ("ObjectPattern", "properties") => Some("_itemCount"),
        _ => None,
    }
}

fn buffer_destination(location: FieldLocation) -> String {
    match location {
        FieldLocation::Payload { offset, .. } => format!("payload, index * 16 + {offset}"),
        FieldLocation::Extra { index } => format!("extra, (extraId + {index}) * 8"),
    }
}

fn string_type<'a>(ty: &'a AstType, schema: &'a Schema) -> Option<(&'a str, bool)> {
    let (inner, optional) = match ty {
        AstType::Option(option) => (&schema.types[option.inner_type_id], true),
        ty => (ty, false),
    };
    match inner {
        AstType::Primitive(primitive) if matches!(primitive.name, "AstUtf8" | "AstWtf8") => {
            Some((primitive.name, optional))
        }
        _ => None,
    }
}

fn primitive_setter<'a>(name: &str, schema: &'a Schema) -> &'a str {
    match name {
        "bool" | "u8" => "U8",
        "i8" => "I8",
        "u16" => "U16",
        "i16" => "I16",
        "u32" | "ScopeId" | "SymbolId" => "U32",
        "i32" => "I32",
        "f64" => "F64",
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
            "U8"
        }
        _ => panic!("unsupported primitive encoder type: {name}"),
    }
}

fn encode_expression(
    node: &str,
    field: &str,
    ty: &AstType,
    schema: &Schema,
    dispatch: &EncoderDispatch,
    source: &str,
) -> String {
    match ty {
        AstType::Struct(ast) if ast.name == "FormalParameters" && has_this_param(node) => {
            format!(
                "encFormalParameters(n.params, {})",
                if dispatch.is_ts { "_paramStart" } else { "0" }
            )
        }
        AstType::Struct(ast) => child_expression(node, field, &format!("enc{}", ast.name), source),
        AstType::Enum(ast) => {
            child_expression(node, field, &dispatch.encoder_name(&ast.name), source)
        }
        AstType::SubRange(range) => {
            let inner = &schema.types[range.inner_type_id];
            let (inner, holes) = match inner {
                AstType::Option(option) => (&schema.types[option.inner_type_id], true),
                inner => (inner, false),
            };
            let encoder = encoder_name(inner, dispatch);
            match (node, field) {
                ("Program" | "FunctionBody" | "TsModuleBlock", "directives") => {
                    format!("encArraySlice(_body, {encoder}, 0, _directiveCount)")
                }
                ("Program" | "FunctionBody" | "TsModuleBlock", "body") => {
                    format!("encArraySlice(_body, {encoder}, _directiveCount, _length)")
                }
                ("FormalParameters", "items")
                | ("ArrayPattern", "elements")
                | ("ObjectPattern", "properties") => "_itemsStart".into(),
                _ => format!("encArray({source}, {encoder}, {holes})"),
            }
        }
        AstType::Option(option) => match &schema.types[option.inner_type_id] {
            AstType::Struct(ast) => format!(
                "{source} == null ? NULL : {}",
                child_expression(node, field, &format!("enc{}", ast.name), source)
            ),
            AstType::Enum(ast) => format!(
                "{source} == null ? NULL : {}",
                child_expression(node, field, &dispatch.encoder_name(&ast.name), source,)
            ),
            AstType::Primitive(primitive) if primitive.name == "ImportPhase" => {
                format!("{source} == null ? 0xff : enumValue(\"ImportPhase\", {source})")
            }
            AstType::Primitive(primitive) if matches!(primitive.name, "ScopeId" | "SymbolId") => {
                "0".into()
            }
            AstType::Primitive(primitive) if matches!(primitive.name, "AstUtf8" | "AstWtf8") => {
                source.into()
            }
            AstType::Primitive(primitive) if primitive.name == "Span" => {
                format!("{source} == null ? [0, 0] : [asStart({source}), asEnd({source})]")
            }
            inner => panic!("unsupported optional encoder type: {}", inner.name()),
        },
        AstType::Primitive(primitive) => match primitive.name {
            "AstUtf8" | "AstWtf8" => source.into(),
            "bool" => format!("{source} ? 1 : 0"),
            "Span" => format!("[asStart({source}), asEnd({source})]"),
            "ScopeId" | "SymbolId" => "0".into(),
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
                format!("enumValue(\"{name}\", {source})")
            }
            _ => format!("{source} ?? 0"),
        },
    }
}

fn encoder_name(ty: &AstType, dispatch: &EncoderDispatch) -> String {
    match ty {
        AstType::Struct(ast) => format!("enc{}", ast.name),
        AstType::Enum(ast) => dispatch.encoder_name(&ast.name),
        _ => panic!("array items must be AST nodes"),
    }
}

fn child_expression(node: &str, field: &str, encoder: &str, source: &str) -> String {
    let default = || format!("{encoder}({source})");
    match (node, field) {
        ("MemberExpression", "property") => {
            format!("n.computed ? encExpr({source}) : {encoder}({source})")
        }
        (
            "MethodDefinition"
            | "TsMethodDefinition"
            | "PropertyDefinition"
            | "ObjectProperty"
            | "BindingProperty"
            | "TsPropertySignature"
            | "TsMethodSignature",
            "key",
        ) => format!("n.computed ? encExpr({source}) : {encoder}(propertyKeyNode({source}))"),
        _ => default(),
    }
}

fn field_source(node: &str, field: &str) -> String {
    let raw = field.strip_prefix("r#").unwrap_or(field);
    let js = if node == "VariableDeclaration" && raw == "declarators" {
        "declarations".into()
    } else if node == "TsEnumDeclaration" && raw == "is_const" {
        "const".into()
    } else {
        raw.to_case(Case::Camel)
    };

    match (node, raw) {
        ("Program" | "FunctionBody" | "TsModuleBlock", "directives") => "_body".into(),
        ("Program" | "FunctionBody" | "TsModuleBlock", "body") => "_body".into(),
        ("FormalParameter", "pattern") => "_pattern".into(),
        ("FormalParameter", "initializer") => "_initializer".into(),
        ("FormalParameter", "type_annotation" | "optional") => format!("_pattern.{js}"),
        ("VariableDeclarator", "type_annotation") => "n.id?.typeAnnotation".into(),
        ("CatchClause", "type_annotation") => "n.param?.typeAnnotation".into(),
        ("TsIndexSignatureParameter", "name") => "n".into(),
        (
            "Function"
            | "TsFunction"
            | "TsFunctionType"
            | "TsMethodSignature"
            | "TsCallSignatureDeclaration",
            "this_param",
        ) => "_thisParam".into(),
        (
            "Function"
            | "TsFunction"
            | "TsFunctionType"
            | "TsMethodSignature"
            | "TsCallSignatureDeclaration",
            "params",
        ) => "n.params".into(),
        ("FormalParameters", "items") => "_values".into(),
        ("FormalParameters", "rest") => "_rest".into(),
        ("FormalParameters", "kind") => "0".into(),
        ("ArrayPattern", "elements") => "_values".into(),
        ("ArrayPattern", "rest") => "_rest".into(),
        ("ObjectPattern", "properties") => "_values".into(),
        ("ObjectPattern", "rest") => "_rest".into(),
        ("Function", "function_type") => {
            "n.type === \"FunctionExpression\" ? \"FunctionExpression\" : \"FunctionDeclaration\""
                .into()
        }
        ("Class", "class_type") => {
            "n.type === \"ClassExpression\" ? \"ClassExpression\" : \"ClassDeclaration\"".into()
        }
        ("Directive", "expression") => "_directiveExpression".into(),
        ("Directive", "value") => "n.directive".into(),
        ("NumericLiteral", "kind") => "numericKind(_numericRaw)".into(),
        ("NumericLiteral", "raw") => "_numericRaw".into(),
        ("BigIntLiteral", "raw") => "bigIntRaw(n)".into(),
        ("RegExpLiteral", "pattern") => "n.regex?.pattern ?? n.value?.source ?? \"\"".into(),
        ("RegExpLiteral", "flags") => "n.regex?.flags ?? n.value?.flags ?? \"\"".into(),
        ("TemplateElement", "cooked") => "n.value?.cooked ?? \"\"".into(),
        ("TemplateElement", "raw") => "n.value?.raw ?? \"\"".into(),
        ("TemplateElement", "is_cooked_undefined") => "n.value?.cooked == null".into(),
        ("PropertyDefinition", "accessor") => {
            "n.type === \"AccessorProperty\" || n.type === \"TSAbstractAccessorProperty\"".into()
        }
        ("PropertyDefinition" | "MethodDefinition" | "TsMethodDefinition", "abstract") => {
            "n.type.startsWith(\"TSAbstract\")".into()
        }
        ("BindingProperty", "shorthand") => "n.shorthand".into(),
        ("JsxText", "value") => "n.value ?? n.raw ?? \"\"".into(),
        ("TsMappedType", "optional") => "n.optional === false ? null : n.optional".into(),
        _ => format!("n.{js}"),
    }
}

fn has_this_param(node: &str) -> bool {
    matches!(
        node,
        "Function"
            | "TsFunction"
            | "TsFunctionType"
            | "TsMethodSignature"
            | "TsCallSignatureDeclaration"
    )
}

fn is_ts_field(node: &str, field: &str, ty: &AstType, schema: &Schema) -> bool {
    let field = field.strip_prefix("r#").unwrap_or(field);
    if let AstType::Option(option) = ty
        && schema.types[option.inner_type_id].name().starts_with("Ts")
    {
        return true;
    }
    matches!(
        field,
        "declare"
            | "definite"
            | "abstract"
            | "override"
            | "readonly"
            | "accessibility"
            | "import_kind"
            | "export_kind"
            | "this_param"
    ) || matches!(
        (node, field),
        ("Class", "implements")
            | ("FormalParameter", "optional")
            | ("MethodDefinition" | "PropertyDefinition", "optional")
    )
}

fn span_expression(node: &str) -> (&'static str, &'static str) {
    if node == "FormalParameters" {
        ("_start", "_end")
    } else {
        ("asStart(n)", "asEnd(n)")
    }
}

fn write_special_prelude(output: &mut String, name: &str, is_ts: bool) {
    match name {
        "Program" | "FunctionBody" | "TsModuleBlock" => output.push_str(
            r#"    const _body = n.body;
    const _length = _body?.length ?? 0;
    let _directiveCount = 0;
    while (_directiveCount < _length) {
      const statement = _body[_directiveCount];
      if (statement?.type !== "ExpressionStatement" ||
          typeof statement.directive !== "string" ||
          statement.expression?.type !== "Literal" ||
          typeof statement.expression.value !== "string") break;
      _directiveCount++;
    }
"#,
        ),
        "Directive" => output.push_str(
            r#"    const _directiveRaw = n.expression?.raw;
    const _directiveQuote = _directiveRaw?.[0];
    const _directiveExpression =
      typeof _directiveRaw === "string" && _directiveRaw.length >= 2 &&
      (_directiveQuote === "\"" || _directiveQuote === "'") &&
      _directiveRaw[_directiveRaw.length - 1] === _directiveQuote &&
      typeof n.directive === "string" &&
      _directiveRaw.length === n.directive.length + 2 &&
      _directiveRaw.startsWith(n.directive, 1) &&
      directiveRawValue(_directiveRaw, n.directive) === n.expression?.value
        ? n.expression
        : { ...n.expression, raw: "" };
"#,
        ),
        "BigIntLiteral" => output.push_str(
            r#"    const _bigIntValue = bigIntValue(n);
    if (_bigIntValue < 0n) {
      const absolute = -_bigIntValue;
      return encUnaryExpression({
        type: "UnaryExpression",
        start: n.start,
        end: n.end,
        operator: "-",
        argument: { ...n, value: absolute, bigint: String(absolute), raw: null },
      });
    }
"#,
        ),
        "FormalParameter" => output.push_str(
            r#"    const _hasInitializer = n.type === "AssignmentPattern";
    const _pattern = _hasInitializer ? n.left : n;
    const _initializer = _hasInitializer ? n.right : null;
"#,
        ),
        "Function" if !is_ts => {}
        "Function"
        | "TsFunction"
        | "TsFunctionType"
        | "TsMethodSignature"
        | "TsCallSignatureDeclaration" => output.push_str(
            r#"    const _first = n.params?.[0];
    const _thisParam = _first?.type === "Identifier" && _first.name === "this" ? _first : null;
    const _paramStart = _thisParam === null ? 0 : 1;
"#,
        ),
        "FormalParameters" => {
            write_rest_array_prelude(output, "n", "from", "encFormalParameterItem", true)
        }
        "ArrayPattern" => write_array_pattern_prelude(output),
        "ObjectPattern" => {
            write_rest_array_prelude(output, "n.properties", "0", "encBindingProperty", false)
        }
        "NumericLiteral" => {
            output.push_str("    const _numericRaw = n.raw == null ? null : numericRaw(n);\n")
        }
        _ => {}
    }
}

fn write_array_pattern_prelude(output: &mut String) {
    output.push_str(
        r#"    const _values = n.elements;
    const _length = _values?.length ?? 0;
    const _last = _values?.[_length - 1];
    const _rest = _last?.type === "RestElement" ? _last : null;
    const _itemCount = _length - (_rest === null ? 0 : 1);
    const _itemsStart = _itemCount ? allocExtra(_itemCount) : 0;
    for (let index = 0; index < _itemCount; index++) {
      const value = _values[index];
      const id = value == null ? NULL : encBindingPattern(value);
      setExtraU32(_itemsStart + index, 0, id);
    }
"#,
    );
}

fn write_rest_array_prelude(
    output: &mut String,
    values: &str,
    from: &str,
    encoder: &str,
    spans: bool,
) {
    writeln!(output, "    const _values = {values};").unwrap();
    output.push_str("    const _length = _values?.length ?? 0;\n");
    writeln!(output, "    let _itemCount = _length - {from};").unwrap();
    output.push_str("    let _rest = null;\n");
    writeln!(output, "    for (let i = {from}; i < _length; i++) {{").unwrap();
    output.push_str(
        r#"      const value = _values[i];
      if (value?.type === "RestElement") {
        _rest = value;
        _itemCount--;
      }
    }
"#,
    );
    if spans {
        writeln!(
            output,
            "    const _start = _length > {from} ? asStart(_values[{from}]) : 0;"
        )
        .unwrap();
        writeln!(
            output,
            "    const _end = _length > {from} ? asEnd(_values[_length - 1]) : 0;"
        )
        .unwrap();
    }
    output.push_str("    const _itemsStart = _itemCount ? allocExtra(_itemCount) : 0;\n    let slot = _itemsStart;\n");
    writeln!(output, "    for (let i = {from}; i < _length; i++) {{").unwrap();
    output.push_str(
        "      const value = _values[i];\n      if (value?.type !== \"RestElement\") {\n",
    );
    writeln!(output, "        const id = {encoder}(value);").unwrap();
    output.push_str("        setExtraU32(slot++, 0, id);\n      }\n    }\n");
}

struct Dispatch {
    order: Vec<String>,
    groups: HashMap<String, Vec<String>>,
}

impl Dispatch {
    fn new(type_id: TypeId, schema: &Schema, is_ts: bool) -> Self {
        let mut concrete = Vec::new();
        collect_concrete(type_id, schema, &mut HashSet::new(), &mut concrete);
        Self::from_concrete(concrete, is_ts)
    }

    fn from_concrete(concrete: Vec<String>, is_ts: bool) -> Self {
        let mut groups: HashMap<String, Vec<String>> = HashMap::new();
        let mut order = Vec::new();
        for concrete_name in concrete {
            if !is_ts && concrete_name.starts_with("Ts") {
                continue;
            }
            for estree in accepted_estree_types(&concrete_name) {
                if !is_ts && estree.starts_with("TS") {
                    continue;
                }
                if !groups.contains_key(&estree) {
                    order.push(estree.clone());
                }
                let values = groups.entry(estree).or_default();
                if !values.contains(&concrete_name) {
                    values.push(concrete_name.clone());
                }
            }
        }
        Self { order, groups }
    }
}

struct EnumDispatch {
    name: String,
    direct: Dispatch,
    nested: Vec<(String, Dispatch)>,
}

struct EncoderDispatch {
    is_ts: bool,
    enums: Vec<EnumDispatch>,
}

impl EncoderDispatch {
    fn new(schema: &Schema, is_ts: bool) -> Self {
        let enums = schema
            .types
            .iter()
            .filter_map(|ty| {
                let AstType::Enum(ast) = ty else {
                    return None;
                };
                if !is_ts && ast.name.starts_with("Ts") {
                    return None;
                }
                let mut concrete = Vec::new();
                let mut nested = Vec::new();
                for variant in &ast.variants {
                    let Some(type_id) = variant.type_id else {
                        continue;
                    };
                    match &schema.types[type_id] {
                        AstType::Struct(child) => concrete.push(child.name.clone()),
                        AstType::Enum(child) => {
                            let dispatch = Dispatch::new(type_id, schema, is_ts);
                            if !dispatch.order.is_empty() {
                                nested.push((child.name.clone(), dispatch));
                            }
                        }
                        _ => {}
                    }
                }
                let mut direct = Dispatch::from_concrete(concrete, is_ts);
                // Function and class structs store both forms; the enum supplies
                // their grammar role at each field boundary.
                match ast.name.as_str() {
                    "Expr" => direct.order.retain(|name| {
                        !matches!(name.as_str(), "FunctionDeclaration" | "ClassDeclaration")
                    }),
                    "Decl" | "ExportDefaultDeclarationKind" => direct.order.retain(|name| {
                        !matches!(
                            name.as_str(),
                            "FunctionExpression"
                                | "ClassExpression"
                                | "TSEmptyBodyFunctionExpression"
                        )
                    }),
                    _ => {}
                }
                Some(EnumDispatch {
                    name: ast.name.clone(),
                    direct,
                    nested,
                })
            })
            .collect();
        Self { is_ts, enums }
    }

    fn encoder_name(&self, name: &str) -> String {
        format!("enc{name}")
    }
}

fn write_enum_encoders(output: &mut String, dispatch: &EncoderDispatch) {
    for current in &dispatch.enums {
        let name = &current.name;
        writeln!(output, "  function enc{name}(n) {{").unwrap();
        if name == "JsxAttributeValue" {
            output.push_str("    if (n.type === \"Literal\" && typeof n.value === \"string\") return encStringLiteral({ ...n, raw: jsxStringRaw(n) });\n");
        }
        let mut cases = Vec::new();
        for estree in &current.direct.order {
            cases.push((estree, None));
        }
        if current.nested.len() != 1 {
            let mut emitted = current.direct.order.iter().cloned().collect::<HashSet<_>>();
            for (nested, inner) in &current.nested {
                for estree in &inner.order {
                    if emitted.insert(estree.clone()) {
                        cases.push((estree, Some(nested)));
                    }
                }
            }
        }
        cases.sort_by_key(|(estree, _)| {
            ESTREE_TYPE_ORDER
                .iter()
                .position(|candidate| candidate == estree)
                .unwrap()
        });
        output.push_str("    switch (n.type) {\n");
        for (estree, nested) in cases {
            if let Some(nested) = nested {
                writeln!(output, "      case \"{estree}\": return enc{nested}(n);").unwrap();
            } else {
                writeln!(output, "      case \"{estree}\":").unwrap();
                write_dispatch(output, estree, &current.direct.groups[estree]);
            }
        }
        if let [(nested, _)] = current.nested.as_slice() {
            writeln!(output, "      default: return enc{nested}(n);").unwrap();
        } else {
            output.push_str(
                "      default: throw new TypeError(`Unsupported ESTree node type ${n.type}`);\n",
            );
        }
        output.push_str("    }\n  }\n\n");
    }
}

fn write_dispatch(output: &mut String, estree: &str, candidates: &[String]) {
    if let [candidate] = candidates {
        writeln!(output, "        return enc{candidate}(n);").unwrap();
        return;
    }

    if estree == "Literal" {
        let has = |name: &str| candidates.iter().any(|candidate| candidate == name);
        if has("RegExpLiteral") {
            output.push_str(
                "        if (n.regex != null || n.value instanceof RegExp) return encRegExpLiteral(n);\n",
            );
        }
        if has("BigIntLiteral") {
            output.push_str(
                "        if (n.bigint != null || typeof n.value === \"bigint\") return encBigIntLiteral(n);\n",
            );
        }
        if has("StringLiteral") {
            output.push_str(
                "        if (typeof n.value === \"string\") return encStringLiteral(n);\n",
            );
        }
        if has("NumericLiteral") {
            output.push_str(
                "        if (typeof n.value === \"number\") return encNumericLiteral(n);\n",
            );
        }
        if has("BooleanLiteral") {
            output.push_str(
                "        if (typeof n.value === \"boolean\") return encBooleanLiteral(n);\n",
            );
        }
        if has("NullLiteral") {
            output.push_str("        return encNullLiteral(n);\n");
            return;
        }
    }

    if matches!(estree, "MethodDefinition" | "TSAbstractMethodDefinition")
        && candidates
            .iter()
            .any(|candidate| candidate == "TsMethodDefinition")
    {
        if estree == "TSAbstractMethodDefinition" {
            output.push_str("        isTs = true;\n");
        }
        output.push_str(
            "        if (n.value?.type === \"TSEmptyBodyFunctionExpression\") return encTsMethodDefinition(n);\n",
        );
    }
    if estree == "TSModuleDeclaration"
        && candidates
            .iter()
            .any(|candidate| candidate == "TsGlobalDeclaration")
    {
        output.push_str(
            "        if (n.global || n.kind === \"global\") return encTsGlobalDeclaration(n);\n",
        );
    }
    if let Some(candidate) = candidates.first() {
        writeln!(output, "        return enc{candidate}(n);").unwrap();
    } else {
        output.push_str("        break;\n");
    }
}

fn collect_concrete(
    type_id: TypeId,
    schema: &Schema,
    seen: &mut HashSet<TypeId>,
    output: &mut Vec<String>,
) {
    match &schema.types[type_id] {
        AstType::Struct(ast) => output.push(ast.name.clone()),
        AstType::Enum(ast) => {
            if !seen.insert(type_id) {
                return;
            }
            for variant in &ast.variants {
                if let Some(type_id) = variant.type_id {
                    collect_concrete(type_id, schema, seen, output);
                }
            }
        }
        _ => {}
    }
}

fn accepted_estree_types(name: &str) -> Vec<String> {
    match name {
        "FunctionBody" => vec!["BlockStatement".into()],
        "TsModuleBlock" => vec!["TSModuleBlock".into()],
        "FormalParameter" => vec![
            "Identifier".into(),
            "ArrayPattern".into(),
            "ObjectPattern".into(),
            "AssignmentPattern".into(),
        ],
        "TsIndexSignatureParameter" => vec!["Identifier".into()],
        "Directive" => vec!["ExpressionStatement".into()],
        "StringLiteral" | "NumericLiteral" | "BigIntLiteral" | "BooleanLiteral" | "NullLiteral"
        | "RegExpLiteral" => vec!["Literal".into()],
        "BindingProperty" | "ObjectProperty" => vec!["Property".into()],
        "Function" => vec!["FunctionDeclaration".into(), "FunctionExpression".into()],
        "TsFunction" => vec![
            "TSDeclareFunction".into(),
            "TSEmptyBodyFunctionExpression".into(),
        ],
        "Class" => vec!["ClassDeclaration".into(), "ClassExpression".into()],
        "MethodDefinition" | "TsMethodDefinition" => vec![
            "MethodDefinition".into(),
            "TSAbstractMethodDefinition".into(),
        ],
        "PropertyDefinition" => vec![
            "PropertyDefinition".into(),
            "AccessorProperty".into(),
            "TSAbstractPropertyDefinition".into(),
            "TSAbstractAccessorProperty".into(),
        ],
        "TsGlobalDeclaration" => vec!["TSModuleDeclaration".into()],
        "TsThisParameter" => vec!["Identifier".into()],
        _ => vec![estree_type(name)],
    }
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

const PREAMBLE: &str = r#"// generated by tasks/ast_tools, do not edit

const NULL = 0xffffffff;
const writeUtf8 = (() => {
  if (typeof Buffer !== "undefined" && typeof Buffer.prototype.write === "function") {
    const write = Buffer.prototype.write;
    // Older Node versions and Buffer polyfills may reject Uint8Array receivers.
    try {
      const probe = new Uint8Array(1);
      if (write.call(probe, "x", 0, 1, "utf8") === 1 && probe[0] === 0x78) return write;
    } catch {}
  }
  const encoder = new TextEncoder();
  return function(value, start, length) {
    return encoder.encodeInto(value, this.subarray(start, start + length)).written;
  };
})();
const HEADER_SIZE = 40;
const NATIVE_LITTLE_ENDIAN =
  new Uint8Array(new Uint32Array([1]).buffer)[0] === 1;
const isWellFormed = String.prototype.isWellFormed
  ? value => value.isWellFormed()
  : value => {
      for (let index = 0; index < value.length; index++) {
        const code = value.charCodeAt(index);
        if (code >= 0xd800 && code <= 0xdbff) {
          const next = value.charCodeAt(++index);
          if (!(next >= 0xdc00 && next <= 0xdfff)) return false;
        } else if (code >= 0xdc00 && code <= 0xdfff) return false;
      }
      return true;
    };

const ENUMS = Object.freeze({
  Lang: ["js", "ts", "jsx", "tsx", "dts"],
  SourceType: ["module", "script", "commonjs", "unambiguous"],
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

export function encodeOptions(options) {
  if (options === undefined || options === null) return 0;
  if (typeof options !== "object" || Array.isArray(options)) {
    throw new TypeError("options must be an object");
  }

  let value = 1;
  value |= encodeOptionBoolean(options.asciiOnly, "asciiOnly") << 1;
  value |= encodeOptionBoolean(options.minify, "minify") << 3;
  value |= encodeOptionBoolean(options.inlineScript, "inlineScript") << 5;
  return value >>> 0;
}

function encodeOptionBoolean(value, name) {
  if (value === undefined || value === null) return 0;
  if (typeof value !== "boolean") {
    throw new TypeError(`${name} must be a boolean`);
  }
  return value ? 2 : 1;
}

function encode(program, meta = {}) {
  const sizeHint = program.end || 0;
  let nodeCapacity = Math.max(512, (sizeHint / 7) | 0);
  let spans = new Uint32Array(nodeCapacity * 2);
  let kinds = new Uint8Array(nodeCapacity);
  let payloadBuffer = new ArrayBuffer(nodeCapacity * 16);
  let payload = new DataView(payloadBuffer);
  let nodeCount = 0;

  let extraCapacity = Math.max(512, (sizeHint / 20) | 0);
  let extraBuffer = new ArrayBuffer(extraCapacity * 8);
  let extra = new DataView(extraBuffer);
  let extraCount = 0;

  let poolCapacity = Math.max(4096, (sizeHint / 8) | 0);
  let pool = new Uint8Array(poolCapacity);
  let poolLength = 0;
  let hasWtf8 = false;
  let firstNonAsciiPos = NULL;
  let isTs = false;

  const enumValue = (name, value) => {
    const index = ENUMS[name]?.indexOf(value);
    return index == null || index < 0 ? 0 : index;
  };
  const asStart = n => n && typeof n.start === "number" ? n.start >>> 0 : 0;
  const asEnd = n => n && typeof n.end === "number" ? n.end >>> 0 : 0;

  function ensureNodes() {
    if (nodeCount < nodeCapacity) return;
    nodeCapacity *= 2;
    const nextSpans = new Uint32Array(nodeCapacity * 2);
    nextSpans.set(spans); spans = nextSpans;
    const nextKinds = new Uint8Array(nodeCapacity);
    nextKinds.set(kinds); kinds = nextKinds;
    const nextPayload = new ArrayBuffer(nodeCapacity * 16);
    new Uint8Array(nextPayload).set(new Uint8Array(payloadBuffer));
    payloadBuffer = nextPayload; payload = new DataView(payloadBuffer);
  }
  function allocNode(kind, start, end) {
    ensureNodes();
    const index = nodeCount++;
    spans[index * 2] = start >>> 0;
    spans[index * 2 + 1] = end >>> 0;
    kinds[index] = kind;
    return index;
  }
  function allocExtra(count) {
    const start = extraCount;
    extraCount += count;
    if (extraCount > extraCapacity) {
      do extraCapacity *= 2; while (extraCount > extraCapacity);
      const next = new ArrayBuffer(extraCapacity * 8);
      new Uint8Array(next).set(new Uint8Array(extraBuffer));
      extraBuffer = next; extra = new DataView(extraBuffer);
    }
    return start;
  }
  function setPayloadU8(i, o, v) { payload.setUint8(i * 16 + o, v); }
  function setPayloadI8(i, o, v) { payload.setInt8(i * 16 + o, v); }
  function setPayloadU16(i, o, v) { payload.setUint16(i * 16 + o, v, true); }
  function setPayloadI16(i, o, v) { payload.setInt16(i * 16 + o, v, true); }
  function setPayloadU32(i, o, v) { payload.setUint32(i * 16 + o, v, true); }
  function setPayloadI32(i, o, v) { payload.setInt32(i * 16 + o, v, true); }
  function setPayloadF64(i, o, v) { payload.setFloat64(i * 16 + o, v, true); }
  function setExtraU8(i, o, v) { extra.setUint8(i * 8 + o, v); }
  function setExtraI8(i, o, v) { extra.setInt8(i * 8 + o, v); }
  function setExtraU16(i, o, v) { extra.setUint16(i * 8 + o, v, true); }
  function setExtraI16(i, o, v) { extra.setInt16(i * 8 + o, v, true); }
  function setExtraU32(i, o, v) { extra.setUint32(i * 8 + o, v, true); }
  function setExtraI32(i, o, v) { extra.setInt32(i * 8 + o, v, true); }
  function setExtraF64(i, o, v) { extra.setFloat64(i * 8 + o, v, true); }

  function encArray(values, encoder, holes) {
    const length = values?.length ?? 0;
    if (!length) return 0;
    // Reserve the contiguous range before children allocate their own extra data.
    const start = allocExtra(length);
    for (let index = 0; index < length; index++) {
      const value = values[index];
      const id = holes && value == null ? NULL : encoder(value);
      // A child may grow extraBuffer, so use the current DataView.
      setExtraU32(start + index, 0, id);
    }
    return start;
  }

  function encArraySlice(values, encoder, from, end) {
    if (from === end) return 0;
    const start = allocExtra(end - from);
    for (let index = from, slot = start; index < end; index++, slot++) {
      const id = encoder(values[index]);
      setExtraU32(slot, 0, id);
    }
    return start;
  }

  function writeArrayRange(start, values, view, offset) {
    // Called after the parent is allocated, so view refers to the current buffer.
    view.setUint32(offset, start, true);
    view.setUint32(offset + 4, start + (values?.length ?? 0), true);
  }

  function writeRange(start, length, view, offset) {
    view.setUint32(offset, start, true);
    view.setUint32(offset + 4, start + length, true);
  }

  function encStr(value, view, offset, wtf8) {
    if (typeof value === "string") {
      const length = value.length;
      const start = poolLength;
      const end = start + length;
      // Keep the short ASCII path small enough to inline into field encoders.
      if (length > 0 && length <= 12 && end <= pool.length && end <= 0x7fffffff) {
        let index = 0;
        for (; index < length; index++) {
          const code = value.charCodeAt(index);
          if (code >= 0x80) break;
          pool[start + index] = code;
        }
        if (index === length) {
          poolLength = end;
          view.setUint32(offset, start, true);
          view.setUint32(offset + 4, end, true);
          return;
        }
      }
    }
    encStrSlow(value, view, offset, wtf8);
  }

  function encStrSlow(value, view, offset, wtf8 = false) {
    if (value == null || value === "") {
      view.setUint32(offset, 0, true);
      view.setUint32(offset + 4, 0, true);
      return;
    }
    if (typeof value !== "string") value = String(value);
    const start = poolLength;
    const length = value.length;
    ensurePool(length * 3);
    const useWtf8 = wtf8 && !isWellFormed(value);
    const written = useWtf8
      ? writeWtf8(value, start)
      : writeUtf8.call(pool, value, start, poolCapacity - start, "utf8");
    hasWtf8 ||= useWtf8;
    poolLength += written;
    if (firstNonAsciiPos === NULL && written !== length) {
      let index = 0;
      while (value.charCodeAt(index) < 0x80) index++;
      firstNonAsciiPos = start + index;
    }
    view.setUint32(offset, start, true);
    view.setUint32(offset + 4, poolLength, true);
  }

  function writeWtf8(value, start) {
    let offset = start;
    for (let index = 0; index < value.length; index++) {
      let code = value.charCodeAt(index);
      if (code < 0x80) {
        pool[offset++] = code;
      } else if (code < 0x800) {
        pool[offset++] = 0xc0 | (code >> 6);
        pool[offset++] = 0x80 | (code & 0x3f);
      } else {
        if (code >= 0xd800 && code <= 0xdbff) {
          const next = value.charCodeAt(index + 1);
          if (next >= 0xdc00 && next <= 0xdfff) {
            code = 0x10000 + ((code - 0xd800) << 10) + next - 0xdc00;
            index++;
            pool[offset++] = 0xf0 | (code >> 18);
            pool[offset++] = 0x80 | ((code >> 12) & 0x3f);
            pool[offset++] = 0x80 | ((code >> 6) & 0x3f);
            pool[offset++] = 0x80 | (code & 0x3f);
            continue;
          }
        }
        pool[offset++] = 0xe0 | (code >> 12);
        pool[offset++] = 0x80 | ((code >> 6) & 0x3f);
        pool[offset++] = 0x80 | (code & 0x3f);
      }
    }
    return offset - start;
  }

  function ensurePool(additionalBytes) {
    const required = poolLength + additionalBytes;
    if (required > 0x7fffffff) throw new RangeError("AST string source exceeds signed 32-bit range");
    if (required <= poolCapacity) return;
    do poolCapacity *= 2; while (required > poolCapacity);
    const next = new Uint8Array(poolCapacity);
    next.set(pool.subarray(0, poolLength)); pool = next;
  }

  function numericKind(raw) {
    if (typeof raw !== "string" || raw.length < 2 || raw.charCodeAt(0) !== 48) return "decimal";
    const prefix = raw[1].toLowerCase();
    return prefix === "x" ? "hex" : prefix === "o" || /^0[0-7]+$/.test(raw) ? "octal" : prefix === "b" ? "binary" : "decimal";
  }
  function numericRaw(node) {
    if (typeof node.raw !== "string") return null;
    const normalized = node.raw.includes("_") ? node.raw.replaceAll("_", "") : node.raw;
    const kind = numericKind(normalized);
    const value = kind === "octal" && normalized[1]?.toLowerCase() !== "o"
      ? parseInt(normalized.slice(1), 8)
      : +normalized;
    return Object.is(value, node.value) ? node.raw : null;
  }

  function bigIntValue(node) {
    return typeof node.value === "bigint" ? node.value : BigInt(node.bigint ?? 0);
  }

  function bigIntRaw(node) {
    const value = bigIntValue(node);
    if (typeof node.raw === "string" && node.raw.endsWith("n")) {
      const raw = node.raw.slice(0, -1);
      try {
        if (BigInt(raw.replaceAll("_", "")) === value) return raw;
      } catch {}
    }
    return String(value);
  }

  function propertyKeyNode(node) {
    if (node?.type !== "Literal" ||
        (node.bigint == null && typeof node.value !== "bigint") ||
        bigIntValue(node) >= 0n) return node;
    return { ...node, value: String(bigIntValue(node)), bigint: null, raw: "" };
  }

  function hexDigit(code) {
    if (code >= 0x30 && code <= 0x39) return code - 0x30;
    if (code >= 0x41 && code <= 0x46) return code - 0x41 + 10;
    if (code >= 0x61 && code <= 0x66) return code - 0x61 + 10;
    return -1;
  }

  function directiveRawValue(raw, unescaped) {
    // The caller already matched the raw interior to this existing string.
    if (raw.indexOf("\\", 1) === -1) return unescaped;
    let result = "";
    for (let index = 1; index < raw.length - 1; index++) {
      let code = raw.charCodeAt(index);
      if (code !== 0x5c) {
        result += raw[index];
        continue;
      }
      if (++index >= raw.length - 1) return null;
      code = raw.charCodeAt(index);
      if (code === 0x0a || code === 0x2028 || code === 0x2029) continue;
      if (code === 0x0d) {
        if (raw.charCodeAt(index + 1) === 0x0a) index++;
        continue;
      }
      const escaped = { 0x62: 0x08, 0x66: 0x0c, 0x6e: 0x0a,
        0x72: 0x0d, 0x74: 0x09, 0x76: 0x0b }[code];
      if (escaped !== undefined) {
        result += String.fromCharCode(escaped);
        continue;
      }
      if (code === 0x78 || code === 0x75) {
        let digits = code === 0x78 ? 2 : 4;
        let start = index + 1;
        let end = start + digits;
        if (code === 0x75 && raw.charCodeAt(start) === 0x7b) {
          start++;
          end = raw.indexOf("}", start);
          if (end < 0 || end === start) return null;
        }
        let value = 0;
        for (let cursor = start; cursor < end; cursor++) {
          const digit = hexDigit(raw.charCodeAt(cursor));
          if (digit < 0) return null;
          value = value * 16 + digit;
        }
        if (end > raw.length - 1 || value > 0x10ffff) return null;
        result += code === 0x75 && raw.charCodeAt(index + 1) === 0x7b
          ? String.fromCodePoint(value)
          : String.fromCharCode(value);
        index = code === 0x75 && raw.charCodeAt(index + 1) === 0x7b ? end : end - 1;
        continue;
      }
      if (code >= 0x30 && code <= 0x37) {
        let value = code - 0x30;
        const maxDigits = code <= 0x33 ? 3 : 2;
        let digits = 1;
        while (digits < maxDigits) {
          const next = raw.charCodeAt(index + 1);
          if (next < 0x30 || next > 0x37) break;
          value = value * 8 + next - 0x30;
          index++;
          digits++;
        }
        result += String.fromCharCode(value);
        continue;
      }
      result += raw[index];
    }
    return result;
  }

  function jsxStringRaw(node) {
    const raw = node.raw;
    if (typeof raw !== "string" || raw.length < 2) return "";
    const quote = raw[0];
    return (quote === "\"" || quote === "'") &&
      raw[raw.length - 1] === quote && raw.slice(1, -1) === node.value
      ? raw
      : "";
  }

"#;

const POSTAMBLE: &str = r#"  const root = encProgram(program);
  const kindsLength = (nodeCount + 7) & ~7;
  const poolPadded = (poolLength + 3) & ~3;
  const spansOffset = HEADER_SIZE;
  const kindsOffset = spansOffset + nodeCount * 8;
  const payloadOffset = kindsOffset + kindsLength;
  const extraOffset = payloadOffset + nodeCount * 16;
  const stringsOffset = extraOffset + extraCount * 8;
  const buffer = new ArrayBuffer(stringsOffset + poolPadded);
  const bytes = new Uint8Array(buffer);
  const view = new DataView(buffer);

  view.setUint32(0, nodeCount, true);
  view.setUint32(4, extraCount, true);
  // A source section must be valid UTF-8. Move strings containing lone
  // surrogates into the extra WTF-8 pool without changing any string offset.
  view.setUint32(8, hasWtf8 ? poolLength : 0, true);
  view.setUint32(12, hasWtf8 ? 0 : poolLength, true);
  view.setUint32(16, 0, true);
  view.setUint32(20, 0, true);
  view.setUint32(24, 0, true);
  view.setUint32(28, root, true);
  const sourceType = enumValue("SourceType", meta.sourceType ?? program.sourceType);
  const lang = meta.lang ?? (isTs ? "ts" : "js");
  view.setUint32(32, sourceType | (enumValue("Lang", lang) << 2), true);
  view.setUint32(36, hasWtf8 ? 0 : firstNonAsciiPos === NULL ? poolLength : firstNonAsciiPos, true);

  if (NATIVE_LITTLE_ENDIAN) {
    bytes.set(new Uint8Array(spans.buffer, 0, nodeCount * 8), spansOffset);
  } else {
    for (let index = 0; index < nodeCount * 2; index++) {
      view.setUint32(spansOffset + index * 4, spans[index], true);
    }
  }
  bytes.set(kinds.subarray(0, nodeCount), kindsOffset);
  bytes.set(new Uint8Array(payloadBuffer, 0, nodeCount * 16), payloadOffset);
  bytes.set(new Uint8Array(extraBuffer, 0, extraCount * 8), extraOffset);
  bytes.set(pool.subarray(0, poolLength), stringsOffset);

  return new Uint8Array(buffer);
}

export { encode };
"#;
