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

const PRETTY_OUTPUT_PATH: &str = "napi/core/src-js/decode.pretty.js";
const MINIFIED_OUTPUT_PATH: &str = "napi/core/src-js/decode.js";

pub fn estree_decoder(schema: &Schema) -> (RawOutput, RawOutput) {
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
            path: PRETTY_OUTPUT_PATH.into(),
            content: output.into_bytes(),
        },
        RawOutput {
            path: MINIFIED_OUTPUT_PATH.into(),
            content: minified.into_bytes(),
        },
    )
}

fn minify(source: &str) -> String {
    GLOBALS.set(&Globals::new(), || {
        let source_map = Lrc::new(SourceMap::default());
        let source_file = source_map.new_source_file(
            FileName::Custom(PRETTY_OUTPUT_PATH.into()).into(),
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
    writeln!(output, "        const f = {{").unwrap();

    let mut extra_offset = 0usize;
    for (field_index, field) in ast.fields.iter().enumerate() {
        if is_semantic_field(&field.name) {
            if !is_inline_field(layout.as_ref(), field_index) {
                extra_offset += 1;
            }
            continue;
        }

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
        writeln!(output, "          {field_name}: {expression},").unwrap();
    }

    writeln!(output, "        }};").unwrap();
    writeln!(
        output,
        "        result = makeNode(\"{}\", f, start, end);",
        ast.name
    )
    .unwrap();
    writeln!(output, "        break;").unwrap();
    writeln!(output, "      }}").unwrap();
}

fn is_inline_field(layout: Option<&crate::util::InlineLayout>, field_index: usize) -> bool {
    layout.is_some_and(|layout| {
        layout
            .fields
            .iter()
            .any(|(index, _, _)| *index == field_index)
    })
}

fn is_semantic_field(name: &str) -> bool {
    matches!(name, "symbol_id" | "scope_id")
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
            "String" => {
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
const textDecoder = new TextDecoder();

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

  const cache = new Array(nodeCount);
  function node(index) {
    const cached = cache[index];
    if (cached !== undefined) return cached;
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
    cache[index] = result;
    return result;
  }

  function makeNode(name, f, start, end) {
    let result;
    switch (name) {
      case "Program":
        result = {
          type: "Program", start, end,
          sourceType: f.sourceType,
          hashbang: f.hashbang,
          body: [...f.directives, ...f.body],
        };
        break;
      case "FunctionBody":
      case "TsModuleBlock":
        result = { type: "BlockStatement", start, end, body: [...f.directives, ...f.body] };
        if (name === "TsModuleBlock") result.type = "TSModuleBlock";
        break;
      case "FormalParameter":
        return f.pattern;
      case "FormalParameters":
        return { params: f.rest === null ? f.items : [...f.items, f.rest] };
      case "Function":
        result = {
          type: f.functionType, start, end,
          id: f.id,
          generator: f.generator,
          async: f.async,
          params: f.params.params,
          body: f.body,
          expression: false,
        };
        if (isTs) {
          result.typeParameters = f.typeParameters;
          result.returnType = f.returnType;
          result.declare = false;
        }
        break;
      case "TsFunction":
        result = {
          type: "TSDeclareFunction",
          start, end,
          id: f.id,
          generator: f.generator,
          async: f.async,
          params: f.params.params,
          body: null,
          expression: false,
          typeParameters: f.typeParameters,
          returnType: f.returnType,
          declare: f.declare,
        };
        break;
      case "ArrowFunctionExpression":
        result = {
          type: "ArrowFunctionExpression", start, end,
          id: null, generator: false, async: f.async,
          params: f.params.params,
          body: f.body, expression: f.expression,
        };
        if (isTs) {
          result.typeParameters = f.typeParameters;
          result.returnType = f.returnType;
        }
        break;
      case "Directive":
        result = {
          type: "ExpressionStatement", start, end,
          expression: f.expression, directive: f.value,
        };
        break;
      case "StringLiteral":
        result = { type: "Literal", start, end, value: f.value, raw: f.raw };
        break;
      case "NumericLiteral": {
        const raw = f.raw;
        const normalized = raw.includes("_") ? raw.replaceAll("_", "") : raw;
        const value =
          f.kind === "octal" && normalized[1]?.toLowerCase() !== "o"
            ? parseInt(normalized.slice(1), 8)
            : +normalized;
        result = { type: "Literal", start, end, value, raw };
        break;
      }
      case "BigIntLiteral": {
        const raw = source.slice(start, end);
        const value = BigInt(f.raw.replaceAll("_", ""));
        result = { type: "Literal", start, end, value, raw, bigint: value.toString() };
        break;
      }
      case "BooleanLiteral":
        result = {
          type: "Literal", start, end,
          value: f.value, raw: f.value ? "true" : "false",
        };
        break;
      case "NullLiteral":
        result = { type: "Literal", start, end, value: null, raw: "null" };
        break;
      case "RegExpLiteral": {
        let value = null;
        try {
          value = new RegExp(f.pattern, f.flags);
        } catch {}
        result = {
          type: "Literal", start, end, value,
          raw: `/${f.pattern}/${f.flags}`,
          regex: { pattern: f.pattern, flags: [...f.flags].sort().join("") },
        };
        break;
      }
      case "TemplateElement":
        result = {
          type: "TemplateElement",
          start: isTs ? start - 1 : start,
          end: isTs ? (f.tail ? end + 1 : end + 2) : end,
          value: { raw: f.raw, cooked: f.isCookedUndefined ? null : f.cooked },
          tail: f.tail,
        };
        break;
      case "Class":
        result = {
          type: f.classType, start, end,
          decorators: f.decorators,
          id: f.id,
          superClass: f.superClass,
          body: f.body,
        };
        if (isTs) {
          result.typeParameters = f.typeParameters;
          result.superTypeArguments = f.superTypeArguments;
          result.implements = f.implements;
          result.abstract = f.abstract;
          result.declare = f.declare;
        }
        break;
      case "MethodDefinition":
      case "TsMethodDefinition": {
        if (name === "TsMethodDefinition") {
          f.value.type = "TSEmptyBodyFunctionExpression";
        }
        result = {
          type: name === "TsMethodDefinition" && f.abstract
            ? "TSAbstractMethodDefinition"
            : "MethodDefinition",
          start, end,
          decorators: f.decorators,
          key: f.key,
          value: f.value,
          kind: f.kind,
          computed: f.computed,
          static: f.static,
        };
        if (isTs) {
          result.override = f.override;
          result.optional = f.optional;
          result.accessibility = f.accessibility;
          if (f.abstract) result.type = "TSAbstractMethodDefinition";
        }
        break;
      }
      case "PropertyDefinition": {
        const accessor = f.accessor;
        result = {
          type: accessor ? "AccessorProperty" : "PropertyDefinition",
          start, end,
          decorators: f.decorators,
          key: f.key,
          value: f.value,
          computed: f.computed,
          static: f.static,
        };
        if (isTs) {
          result.typeAnnotation = f.typeAnnotation;
          result.declare = f.declare;
          result.override = f.override;
          result.optional = f.optional;
          result.definite = f.definite;
          result.readonly = f.readonly;
          result.accessibility = f.accessibility;
          if (f.abstract) {
            result.type = accessor
              ? "TSAbstractAccessorProperty"
              : "TSAbstractPropertyDefinition";
          }
        }
        break;
      }
      case "UnaryExpression":
        result = {
          type: "UnaryExpression", start, end,
          operator: f.operator, prefix: true, argument: f.argument,
        };
        break;
      case "BindingProperty":
        result = {
          type: "Property", start, end, kind: "init",
          key: f.key, value: f.value,
          method: false, shorthand: f.shorthand, computed: f.computed,
        };
        if (isTs) result.optional = false;
        break;
      case "ArrayPattern": {
        const elements = f.rest === null ? f.elements : [...f.elements, f.rest];
        result = { type: "ArrayPattern", start, end, elements };
        if (isTs) {
          result.decorators = f.decorators;
          result.optional = f.optional;
          result.typeAnnotation = f.typeAnnotation;
        }
        break;
      }
      case "ObjectPattern": {
        const properties = f.rest === null ? f.properties : [...f.properties, f.rest];
        result = { type: "ObjectPattern", start, end, properties };
        if (isTs) {
          result.decorators = f.decorators;
          result.optional = f.optional;
          result.typeAnnotation = f.typeAnnotation;
        }
        break;
      }
      case "JsxText":
        result = { type: "JSXText", start, end, value: f.value, raw: f.value };
        break;
      case "TsFunctionType":
      case "TsConstructorType":
      case "TsMethodSignature":
      case "TsCallSignatureDeclaration":
      case "TsConstructSignatureDeclaration":
        result = {
          type: estreeType(name), start, end,
          ...(name === "TsConstructorType" ? { abstract: f.abstract } : {}),
          ...(name === "TsMethodSignature"
            ? { key: f.key, computed: f.computed, optional: f.optional, kind: f.kind }
            : {}),
          typeParameters: f.typeParameters,
          params: f.params.params,
          returnType: f.returnType,
        };
        if (name === "TsMethodSignature") {
          result.accessibility = null;
          result.readonly = false;
          result.static = false;
        }
        break;
      case "TsMappedType":
        result = {
          type: "TSMappedType", start, end,
          key: f.key, constraint: f.constraint,
          nameType: f.nameType, typeAnnotation: f.typeAnnotation,
          optional: f.optional === null ? false : f.optional,
          readonly: f.readonly,
        };
        break;
      case "TsModuleDeclaration":
        result = {
          type: "TSModuleDeclaration", start, end,
          id: f.id, kind: f.kind, declare: f.declare, global: false,
        };
        if (f.body !== null) result.body = f.body;
        break;
      case "TsGlobalDeclaration":
        result = {
          type: "TSModuleDeclaration", start, end,
          id: f.id, body: f.body, kind: "global",
          declare: f.declare, global: true,
        };
        break;
      case "TsThisParameter":
        result = {
          type: "Identifier", start, end,
          decorators: [], name: "this", optional: false,
          typeAnnotation: f.typeAnnotation,
        };
        break;
      default: {
        result = { type: estreeType(name), start, end };
        for (const [key, value] of Object.entries(f)) {
          if (key !== "__extra" && (isTs || !isTsField(name, key))) result[key] = value;
        }
        addTsExtras(name, result, isTs);
      }
    }
    return result;
  }

  const root = node(programIndex);
  return root;
}

function estreeType(name) {
  const overrides = {
    BindingRestElement: "RestElement",
    ObjectProperty: "Property",
    IdentifierReference: "Identifier",
    BindingIdentifier: "Identifier",
    IdentifierName: "Identifier",
    LabelIdentifier: "Identifier",
    TsBigIntKeyword: "TSBigIntKeyword",
    TsJsDocNullableType: "TSJSDocNullableType",
    TsJsDocNonNullableType: "TSJSDocNonNullableType",
    TsJsDocUnknownType: "TSJSDocUnknownType",
  };
  if (overrides[name]) return overrides[name];
  if (name.startsWith("Jsx")) return `JSX${name.slice(3)}`;
  if (name.startsWith("Ts")) return `TS${name.slice(2)}`;
  return name;
}

const TS_FIELDS = Object.freeze({
  VariableDeclaration: ["declare"],
  VariableDeclarator: ["definite"],
  CallExpression: ["typeArguments"],
  NewExpression: ["typeArguments"],
  TaggedTemplateExpression: ["typeArguments"],
  JsxOpeningElement: ["typeArguments"],
  ImportDeclaration: ["importKind"],
  ImportSpecifier: ["importKind"],
  ExportNamedDeclaration: ["exportKind"],
  ExportAllDeclaration: ["exportKind"],
  ExportSpecifier: ["exportKind"],
  BindingRestElement: ["decorators", "optional", "typeAnnotation"],
  BindingIdentifier: ["decorators", "optional", "typeAnnotation"],
  AssignmentPattern: ["decorators", "optional", "typeAnnotation"],
  ObjectPattern: ["decorators", "optional", "typeAnnotation"],
  ArrayPattern: ["decorators", "optional", "typeAnnotation"],
  TsIndexSignature: ["static"],
});

function isTsField(name, field) {
  return TS_FIELDS[name]?.includes(field) ?? false;
}

function addTsExtras(name, result, isTs) {
  if (!isTs) return;
  if (["IdentifierReference", "IdentifierName", "LabelIdentifier"].includes(name)) {
    result.decorators = [];
    result.optional = false;
    result.typeAnnotation = null;
  } else if (name === "ObjectProperty") {
    result.optional = false;
  } else if (name === "ExpressionStatement") {
    result.directive = null;
  } else if (name === "BindingRestElement") {
    result.value = null;
  } else if (name === "ExportDefaultDeclaration") {
    result.exportKind = "value";
  } else if (name === "TsPropertySignature") {
    result.accessibility = null;
    result.static = false;
  } else if (name === "TsIndexSignature") {
    result.accessibility = null;
  } else if (name === "TsParameterProperty") {
    result.static = false;
  }
}
"#;
