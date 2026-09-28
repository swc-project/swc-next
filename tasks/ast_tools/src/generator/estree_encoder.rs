use std::{
    collections::{HashMap, HashSet},
    fmt::Write,
};

use convert_case::{Case, Casing};

use crate::{
    output::RawOutput,
    schema::{AstStruct, AstType, FieldLocation, Schema, TypeId},
};

use super::js_minifier::minify;

const PRETTY_OUTPUT_PATH: &str = "packages/encoder/index.pretty.js";
const MINIFIED_OUTPUT_PATH: &str = "packages/encoder/index.js";

pub fn estree_encoder(schema: &Schema) -> (RawOutput, RawOutput) {
    let mut output = String::new();
    output.push_str(PREAMBLE);
    let dispatch = EncoderDispatch::new(schema);

    for (kind, ty) in schema
        .types
        .iter()
        .filter_map(|ty| match ty {
            AstType::Struct(ast) => Some(ast),
            _ => None,
        })
        .enumerate()
    {
        write_struct_encoder(&mut output, kind, ty, schema, &dispatch);
    }

    write_node_encoder(&mut output, &dispatch.global);
    write_contextual_encoders(&mut output, &dispatch);

    output.push_str(POSTAMBLE);
    let minified = minify(&output, PRETTY_OUTPUT_PATH);
    (
        RawOutput {
            path: PRETTY_OUTPUT_PATH.into(),
            content: output.into_bytes(),
        },
        RawOutput {
            path: MINIFIED_OUTPUT_PATH.into(),
            content: minified.into_bytes(),
        },
    )
}

fn write_struct_encoder(
    output: &mut String,
    kind: usize,
    ast: &AstStruct,
    schema: &Schema,
    dispatch: &EncoderDispatch,
) {
    let name = &ast.name;
    writeln!(output, "  function enc{name}(n) {{").unwrap();
    if name.starts_with("Ts") {
        output.push_str("    isTs = true;\n");
    }
    write_special_prelude(output, name);

    let layout = &ast.layout;
    let mut fields = Vec::with_capacity(ast.fields.len());

    for (field_index, field) in ast.fields.iter().enumerate() {
        let ty = &schema.types[field.type_id];
        let source = field_source(name, &field.name);
        let source = if field.attrs.estree_skip {
            "0".into()
        } else {
            source
        };
        let value = encode_expression(name, &field.name, ty, schema, dispatch, &source);
        let local = format!("v_{field_index}");
        writeln!(output, "    const {local} = {value};").unwrap();

        fields.push((ty, local, layout.fields[field_index]));
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
    for (ty, local, location) in fields {
        write_value(output, ty, schema, &local, location);
    }
    output.push_str("    return index;\n  }\n\n");
}

fn write_value(
    output: &mut String,
    ty: &AstType,
    schema: &Schema,
    value: &str,
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

    match ty {
        AstType::Struct(_) | AstType::Enum(_) => {
            writeln!(output, "    {}", setter("U32", 0, value)).unwrap();
        }
        AstType::SubRange(_) => {
            writeln!(
                output,
                "    {}",
                setter("U32", 0, &format!("{value}.start"))
            )
            .unwrap();
            writeln!(output, "    {}", setter("U32", 4, &format!("{value}.end"))).unwrap();
        }
        AstType::Option(option) => match &schema.types[option.inner_type_id] {
            AstType::Struct(_) | AstType::Enum(_) => {
                writeln!(output, "    {}", setter("U32", 0, value)).unwrap();
            }
            AstType::Primitive(primitive) if matches!(primitive.name, "AstUtf8" | "AstWtf8") => {
                writeln!(
                    output,
                    "    {}",
                    setter("U32", 0, &format!("{value}.start"))
                )
                .unwrap();
                writeln!(output, "    {}", setter("U32", 4, &format!("{value}.end"))).unwrap();
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
            "AstUtf8" | "AstWtf8" | "Span" => {
                let (start, end) = if primitive.name == "Span" {
                    (format!("{value}[0]"), format!("{value}[1]"))
                } else {
                    (format!("{value}.start"), format!("{value}.end"))
                };
                writeln!(output, "    {}", setter("U32", 0, &start)).unwrap();
                writeln!(output, "    {}", setter("U32", 4, &end)).unwrap();
            }
            name => {
                let kind = primitive_setter(name, schema);
                writeln!(output, "    {}", setter(kind, 0, value)).unwrap();
            }
        },
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
            format!("encArray({source}, {encoder}, {holes})")
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
                let encoder = if primitive.name == "AstUtf8" {
                    "encUtf8"
                } else {
                    "encStr"
                };
                format!("{source} == null ? {{ start: NULL, end: 0 }} : {encoder}({source})")
            }
            AstType::Primitive(primitive) if primitive.name == "Span" => {
                format!("{source} == null ? [0, 0] : [asStart({source}), asEnd({source})]")
            }
            inner => panic!("unsupported optional encoder type: {}", inner.name()),
        },
        AstType::Primitive(primitive) => match primitive.name {
            "AstUtf8" => format!("encUtf8({source})"),
            "AstWtf8" => format!("encStr({source})"),
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
            format!("n.computed ? encNode({source}) : {encoder}({source})")
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
        ) => format!("n.computed ? encNode({source}) : {encoder}(propertyKeyNode({source}))"),
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
        ("Program" | "FunctionBody" | "TsModuleBlock", "directives") => "_directives".into(),
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
        ) => "_params".into(),
        ("FormalParameters", "items") => "_items".into(),
        ("FormalParameters", "rest") => "_rest".into(),
        ("FormalParameters", "kind") => "0".into(),
        ("ArrayPattern", "elements") => "_elements".into(),
        ("ArrayPattern", "rest") => "_rest".into(),
        ("ObjectPattern", "properties") => "_properties".into(),
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

fn span_expression(node: &str) -> (&'static str, &'static str) {
    if node == "FormalParameters" {
        ("_start", "_end")
    } else {
        ("asStart(n)", "asEnd(n)")
    }
}

fn write_special_prelude(output: &mut String, name: &str) {
    match name {
        "Program" | "FunctionBody" | "TsModuleBlock" => output.push_str(
            r#"    const _directives = [];
    const _body = [];
    let inDirectivePrologue = true;
    for (const statement of n.body ?? []) {
      if (inDirectivePrologue &&
          statement?.type === "ExpressionStatement" && statement.directive != null) {
        _directives.push(statement);
      } else {
        inDirectivePrologue = false;
        _body.push(statement);
      }
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
      _directiveRaw.slice(1, -1) === n.directive &&
      directiveRawValue(_directiveRaw) === n.expression?.value
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
        "Function"
        | "TsFunction"
        | "TsFunctionType"
        | "TsMethodSignature"
        | "TsCallSignatureDeclaration" => output.push_str(
            r#"    const _first = n.params?.[0];
    const _thisParam = _first?.type === "Identifier" && _first.name === "this" ? _first : null;
    const _params = _thisParam === null ? n.params : n.params.slice(1);
"#,
        ),
        "FormalParameters" => output.push_str(
            r#"    const _items = [];
    let _rest = null;
    for (const parameter of n ?? []) {
      if (parameter?.type === "RestElement") _rest = parameter;
      else _items.push(parameter);
    }
    const _start = n?.length ? asStart(n[0]) : 0;
    const _end = n?.length ? asEnd(n[n.length - 1]) : 0;
"#,
        ),
        "ArrayPattern" => output.push_str(
            r#"    const _elements = [];
    let _rest = null;
    for (const element of n.elements ?? []) {
      if (element?.type === "RestElement") _rest = element;
      else _elements.push(element);
    }
"#,
        ),
        "ObjectPattern" => output.push_str(
            r#"    const _properties = [];
    let _rest = null;
    for (const property of n.properties ?? []) {
      if (property?.type === "RestElement") _rest = property;
      else _properties.push(property);
    }
"#,
        ),
        "NumericLiteral" => {
            output.push_str("    const _numericRaw = n.raw == null ? null : numericRaw(n);\n")
        }
        _ => {}
    }
}

struct Dispatch {
    order: Vec<String>,
    groups: HashMap<String, Vec<String>>,
}

impl Dispatch {
    fn new(type_id: TypeId, schema: &Schema) -> Self {
        let mut concrete = Vec::new();
        collect_concrete(type_id, schema, &mut HashSet::new(), &mut concrete);
        Self::from_concrete(concrete)
    }

    fn from_concrete(concrete: Vec<String>) -> Self {
        let mut groups: HashMap<String, Vec<String>> = HashMap::new();
        let mut order = Vec::new();
        for concrete_name in concrete {
            for estree in accepted_estree_types(&concrete_name) {
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

struct EncoderDispatch {
    global: Dispatch,
    enums: Vec<(String, Dispatch)>,
    contextual: HashSet<String>,
}

impl EncoderDispatch {
    fn new(schema: &Schema) -> Self {
        let enums = schema
            .types
            .iter()
            .filter_map(|ty| match ty {
                AstType::Enum(ast) => Some((ast.name.clone(), Dispatch::new(ast.type_id, schema))),
                _ => None,
            })
            .collect::<Vec<_>>();

        let mut order = Vec::new();
        let mut choices: HashMap<String, Vec<(Vec<String>, usize)>> = HashMap::new();
        for (_, dispatch) in &enums {
            for estree in &dispatch.order {
                let candidates = &dispatch.groups[estree];
                let entries = choices.entry(estree.clone()).or_insert_with(|| {
                    order.push(estree.clone());
                    Vec::new()
                });
                if let Some((_, count)) = entries
                    .iter_mut()
                    .find(|(existing, _)| same_dispatch(estree, existing, candidates))
                {
                    *count += 1;
                } else {
                    entries.push((candidates.clone(), 1));
                }
            }
        }

        let groups = choices
            .into_iter()
            .map(|(estree, entries)| {
                let mut selected: Option<(Vec<String>, usize)> = None;
                for (candidates, count) in entries {
                    if selected
                        .as_ref()
                        .is_none_or(|(_, selected_count)| count > *selected_count)
                    {
                        selected = Some((candidates, count));
                    }
                }
                (estree, selected.unwrap().0)
            })
            .collect();
        let global = Dispatch { order, groups };

        let contextual = enums
            .iter()
            .filter(|(name, dispatch)| {
                name == "FormalParameterPattern"
                    || dispatch.groups.iter().any(|(estree, candidates)| {
                        !same_dispatch(estree, &global.groups[estree], candidates)
                    })
            })
            .map(|(name, _)| name.clone())
            .collect();

        Self {
            global,
            enums,
            contextual,
        }
    }

    fn encoder_name(&self, name: &str) -> String {
        if self.contextual.contains(name) {
            format!("enc{name}")
        } else {
            "encNode".into()
        }
    }
}

fn write_node_encoder(output: &mut String, dispatch: &Dispatch) {
    output.push_str("  function encNode(n) {\n    switch (n.type) {\n");
    for estree in &dispatch.order {
        writeln!(output, "      case \"{estree}\":").unwrap();
        write_dispatch(output, estree, &dispatch.groups[estree]);
    }
    output.push_str(
        "      default: throw new TypeError(`Unsupported ESTree node type ${n.type}`);\n    }\n  }\n\n",
    );
}

fn write_contextual_encoders(output: &mut String, dispatch: &EncoderDispatch) {
    for (name, current) in &dispatch.enums {
        if !dispatch.contextual.contains(name) {
            continue;
        }

        writeln!(output, "  function enc{name}(n) {{").unwrap();
        if name == "JsxAttributeValue" {
            output.push_str(
                "    if (n.type === \"Literal\" && typeof n.value === \"string\") return encStringLiteral({ ...n, raw: jsxStringRaw(n) });\n",
            );
        }
        output.push_str("    switch (n.type) {\n");
        for estree in &current.order {
            let candidates = &current.groups[estree];
            if same_dispatch(estree, &dispatch.global.groups[estree], candidates) {
                continue;
            }
            writeln!(output, "      case \"{estree}\":").unwrap();
            write_dispatch(output, estree, candidates);
        }
        output.push_str("      default: return encNode(n);\n    }\n  }\n\n");
    }
}

fn same_dispatch(estree: &str, left: &[String], right: &[String]) -> bool {
    let mut left_output = String::new();
    let mut right_output = String::new();
    write_dispatch(&mut left_output, estree, left);
    write_dispatch(&mut right_output, estree, right);
    left_output == right_output
}

fn write_dispatch(output: &mut String, estree: &str, candidates: &[String]) {
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
        output.push_str(
            "        if (n.type === \"TSAbstractMethodDefinition\" || n.value?.type === \"TSEmptyBodyFunctionExpression\") return encTsMethodDefinition(n);\n",
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
        "TsMethodDefinition" => vec![
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
const HEADER_SIZE = 40;
const NATIVE_LITTLE_ENDIAN =
  new Uint8Array(new Uint32Array([1]).buffer)[0] === 1;

const ENUMS = Object.freeze({
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

function encode(program) {
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
  const strings = new Map();
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
  function setPayloadU16(i, o, v) { payload.setUint16(i * 16 + o, v, NATIVE_LITTLE_ENDIAN); }
  function setPayloadI16(i, o, v) { payload.setInt16(i * 16 + o, v, NATIVE_LITTLE_ENDIAN); }
  function setPayloadU32(i, o, v) { payload.setUint32(i * 16 + o, v, NATIVE_LITTLE_ENDIAN); }
  function setPayloadI32(i, o, v) { payload.setInt32(i * 16 + o, v, NATIVE_LITTLE_ENDIAN); }
  function setPayloadF64(i, o, v) { payload.setFloat64(i * 16 + o, v, NATIVE_LITTLE_ENDIAN); }
  function setExtraU8(i, o, v) { extra.setUint8(i * 8 + o, v); }
  function setExtraI8(i, o, v) { extra.setInt8(i * 8 + o, v); }
  function setExtraU16(i, o, v) { extra.setUint16(i * 8 + o, v, NATIVE_LITTLE_ENDIAN); }
  function setExtraI16(i, o, v) { extra.setInt16(i * 8 + o, v, NATIVE_LITTLE_ENDIAN); }
  function setExtraU32(i, o, v) { extra.setUint32(i * 8 + o, v, NATIVE_LITTLE_ENDIAN); }
  function setExtraI32(i, o, v) { extra.setInt32(i * 8 + o, v, NATIVE_LITTLE_ENDIAN); }
  function setExtraF64(i, o, v) { extra.setFloat64(i * 8 + o, v, NATIVE_LITTLE_ENDIAN); }

  function encArray(values, encoder, holes) {
    if (!values?.length) return { start: 0, end: 0 };
    const ids = new Uint32Array(values.length);
    for (let index = 0; index < values.length; index++) {
      const value = values[index];
      ids[index] = holes && value == null ? NULL : encoder(value);
    }
    const start = allocExtra(ids.length);
    for (let index = 0; index < ids.length; index++) {
      setExtraU32(start + index, 0, ids[index]);
    }
    return { start, end: start + ids.length };
  }

  function encUtf8(value) {
    const result = encStr(value);
    if (!result.isUtf8) {
      throw new TypeError("AST UTF-8 field contains an unpaired surrogate");
    }
    return result;
  }

  function encStr(value) {
    if (value == null || value === "") return { start: 0, end: 0, isUtf8: true };
    value = String(value);
    const previous = strings.get(value);
    if (previous !== undefined) return previous;
    const required = poolLength + value.length * 3;
    if (required > poolCapacity) {
      do poolCapacity *= 2; while (required > poolCapacity);
      const next = new Uint8Array(poolCapacity);
      next.set(pool); pool = next;
    }
    const start = poolLength;
    let isUtf8 = true;
    for (let index = 0; index < value.length; index++) {
      let code = value.charCodeAt(index);
      if (code < 0x80) pool[poolLength++] = code;
      else if (code < 0x800) {
        pool[poolLength++] = 0xc0 | (code >> 6);
        pool[poolLength++] = 0x80 | (code & 0x3f);
      } else if (code >= 0xd800 && code <= 0xdbff && index + 1 < value.length &&
                 value.charCodeAt(index + 1) >= 0xdc00 && value.charCodeAt(index + 1) <= 0xdfff) {
        code = 0x10000 + ((code - 0xd800) << 10) + value.charCodeAt(++index) - 0xdc00;
        pool[poolLength++] = 0xf0 | (code >> 18);
        pool[poolLength++] = 0x80 | ((code >> 12) & 0x3f);
        pool[poolLength++] = 0x80 | ((code >> 6) & 0x3f);
        pool[poolLength++] = 0x80 | (code & 0x3f);
      } else {
        if (code >= 0xd800 && code <= 0xdfff) isUtf8 = false;
        pool[poolLength++] = 0xe0 | (code >> 12);
        pool[poolLength++] = 0x80 | ((code >> 6) & 0x3f);
        pool[poolLength++] = 0x80 | (code & 0x3f);
      }
    }
    const result = { start, end: poolLength, isUtf8 };
    strings.set(value, result);
    return result;
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

  function directiveRawValue(raw) {
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
  const kindsLength = (nodeCount + 3) & ~3;
  const poolPadded = (poolLength + 3) & ~3;
  const spansOffset = HEADER_SIZE;
  const kindsOffset = spansOffset + nodeCount * 8;
  const payloadOffset = kindsOffset + kindsLength;
  const extraOffset = payloadOffset + nodeCount * 16;
  const stringsOffset = extraOffset + extraCount * 8;
  const buffer = new ArrayBuffer(stringsOffset + poolPadded);
  const bytes = new Uint8Array(buffer);
  const view = new DataView(buffer);

  view.setUint32(0, nodeCount, NATIVE_LITTLE_ENDIAN);
  view.setUint32(4, extraCount, NATIVE_LITTLE_ENDIAN);
  view.setUint32(8, poolLength, NATIVE_LITTLE_ENDIAN);
  view.setUint32(12, 0, NATIVE_LITTLE_ENDIAN);
  view.setUint32(16, 0, NATIVE_LITTLE_ENDIAN);
  view.setUint32(20, 0, NATIVE_LITTLE_ENDIAN);
  view.setUint32(24, 0, NATIVE_LITTLE_ENDIAN);
  view.setUint32(28, root, NATIVE_LITTLE_ENDIAN);
  const sourceType = enumValue("SourceType", program.sourceType);
  view.setUint32(32, (isTs ? 1 : 0) | (sourceType << 1), NATIVE_LITTLE_ENDIAN);
  view.setUint32(36, 0, NATIVE_LITTLE_ENDIAN);

  bytes.set(new Uint8Array(spans.buffer, 0, nodeCount * 8), spansOffset);
  bytes.set(kinds.subarray(0, nodeCount), kindsOffset);
  bytes.set(new Uint8Array(payloadBuffer, 0, nodeCount * 16), payloadOffset);
  bytes.set(new Uint8Array(extraBuffer, 0, extraCount * 8), extraOffset);
  bytes.set(pool.subarray(0, poolLength), stringsOffset);

  return new Uint8Array(buffer);
}

export { encode };
"#;
