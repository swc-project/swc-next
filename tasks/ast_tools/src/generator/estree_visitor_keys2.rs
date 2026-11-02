use std::{collections::BTreeMap, fmt::Write};

use super::estree::{estree_field_name, estree_type};
use super::ffi2_protocol::field_rule;
use crate::{
    output::RawOutput,
    schema::{AstType, FieldLocation, Schema, TypeId},
};

/// ESTree type names a native node projects to. Dynamic projections return
/// every possible public name; `None` means the native node is never public.
pub(super) fn public_types(name: &str) -> Option<Vec<String>> {
    Some(match name {
        "FormalParameter" | "FormalParameters" | "TsIndexSignatureParameter" => return None,
        "Program" => vec!["Program".into()],
        "FunctionBody" => vec!["BlockStatement".into()],
        "TsModuleBlock" => vec!["TSModuleBlock".into()],
        "Function" => vec!["FunctionDeclaration".into(), "FunctionExpression".into()],
        "TsFunction" => vec![
            "TSDeclareFunction".into(),
            "TSEmptyBodyFunctionExpression".into(),
        ],
        "ArrowFunctionExpression" => vec![name.into()],
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
        "Directive" => vec!["ExpressionStatement".into()],
        "StringLiteral" | "NumericLiteral" | "BigIntLiteral" | "BooleanLiteral" | "NullLiteral"
        | "RegExpLiteral" => vec!["Literal".into()],
        "TemplateElement" => vec![name.into()],
        "FormalParameterRest" | "BindingRestElement" => vec!["RestElement".into()],
        "BindingProperty" | "ObjectProperty" => vec!["Property".into()],
        "ArrayPattern" => vec![name.into()],
        "ObjectPattern" => vec![name.into()],
        "AssignmentPattern" => vec![name.into()],
        "VariableDeclarator" => vec![name.into()],
        "CatchClause" => vec![name.into()],
        "TsThisParameter" => vec!["Identifier".into()],
        "TsGlobalDeclaration" => vec!["TSModuleDeclaration".into()],
        "TsFunctionType"
        | "TsConstructorType"
        | "TsCallSignatureDeclaration"
        | "TsConstructSignatureDeclaration" => vec![estree_type(name)],
        "TsMethodSignature" => vec![estree_type(name)],
        _ => vec![estree_type(name)],
    })
}

/// Visitor key overrides for projections which flatten or merge fields.
fn special_fields(name: &str) -> Option<&'static [&'static str]> {
    Some(match name {
        "Program" => &["hashbang", "body"],
        "FunctionBody" => &["body"],
        "TsModuleBlock" => &["body"],
        "Function" => &["id", "typeParameters", "params", "returnType", "body"],
        "TsFunction" => &["id", "typeParameters", "params", "returnType", "body"],
        "ArrowFunctionExpression" => &["typeParameters", "params", "returnType", "body"],
        "Class" => &[
            "decorators",
            "id",
            "typeParameters",
            "superClass",
            "superTypeArguments",
            "implements",
            "body",
        ],
        "MethodDefinition" | "TsMethodDefinition" => &["decorators", "key", "value"],
        "PropertyDefinition" => &["decorators", "key", "typeAnnotation", "value"],
        "Directive" => &["expression"],
        "StringLiteral" | "NumericLiteral" | "BigIntLiteral" | "BooleanLiteral" | "NullLiteral"
        | "RegExpLiteral" => &[],
        "TemplateElement" => &[],
        "FormalParameterRest" | "BindingRestElement" => {
            &["decorators", "argument", "typeAnnotation"]
        }
        "BindingProperty" | "ObjectProperty" => &["key", "value"],
        "ArrayPattern" => &["decorators", "elements", "typeAnnotation"],
        "ObjectPattern" => &["decorators", "properties", "typeAnnotation"],
        "AssignmentPattern" => &["decorators", "left", "typeAnnotation", "right"],
        "VariableDeclarator" => &["id", "init"],
        "CatchClause" => &["param", "body"],
        "TsThisParameter" => &["decorators", "typeAnnotation"],
        "TsGlobalDeclaration" => &["id", "body"],
        "TsFunctionType"
        | "TsConstructorType"
        | "TsCallSignatureDeclaration"
        | "TsConstructSignatureDeclaration" => &["typeParameters", "params", "returnType"],
        "TsMethodSignature" => &["key", "typeParameters", "params", "returnType"],
        _ => return None,
    })
}

/// Canonical child order for public ESTree types assembled from several native
/// nodes. Native declaration order must not leak into the public traversal order.
fn public_field_order(name: &str) -> Option<&'static [&'static str]> {
    Some(match name {
        "Identifier" => &["decorators", "typeAnnotation"],
        _ => return None,
    })
}

// ESTree projections which flatten, merge, or synthesize internal AST nodes.
// Keep these aligned with the generated AST projection rules.
pub fn estree_visitor_keys(schema: &Schema) -> Vec<RawOutput> {
    let mut keys: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut lists = BTreeMap::new();
    let mut projected_types = BTreeMap::new();
    for ty in &schema.types {
        let AstType::Struct(ast) = ty else { continue };
        let name = ast.name.as_str();
        let Some(names) = public_types(name) else {
            continue;
        };
        let special = special_fields(name);
        let fields: Vec<String> = special
            .map(|fields| fields.iter().map(|s| (*s).into()).collect())
            .unwrap_or_else(|| {
                ast.fields
                    .iter()
                    .filter(|f| !f.attrs.estree_skip && is_child(f.type_id, schema))
                    .map(|f| estree_field_name(name, &f.name))
                    .collect()
            });
        if names.len() == 1 {
            projected_types.insert(ast.name.clone(), names[0].clone());
        }
        for name in names {
            for field in &fields {
                let list = if special.is_some() {
                    matches!(
                        field.as_str(),
                        "decorators" | "params" | "implements" | "elements" | "properties"
                    ) || (field == "body"
                        && matches!(
                            name.as_str(),
                            "Program" | "BlockStatement" | "TSModuleBlock"
                        ))
                } else {
                    ast.fields.iter().any(|f| {
                        estree_field_name(ast.name.as_str(), &f.name) == *field
                            && is_list(f.type_id, schema)
                    })
                };
                let previous = lists.insert((name.clone(), field.clone()), list);
                assert!(previous.is_none_or(|previous| previous == list));
            }
            let entry = keys.entry(name).or_default();
            for field in &fields {
                if !entry.contains(field) {
                    entry.push(field.clone());
                }
            }
        }
    }
    for (name, fields) in &mut keys {
        let Some(order) = public_field_order(name) else {
            continue;
        };
        let mut ordered = Vec::with_capacity(fields.len());
        for field in order {
            if fields.iter().any(|current| current.as_str() == *field) {
                ordered.push((*field).to_string());
            }
        }
        for field in fields.iter() {
            if !ordered.iter().any(|current| current == field) {
                ordered.push(field.clone());
            }
        }
        *fields = ordered;
    }
    let mut js = String::from(
        "// generated by tasks/ast_tools, do not edit\nexport const visitorKeys = Object.freeze({\n",
    );
    let mut dts = String::from(
        "// generated by tasks/ast_tools, do not edit\nexport declare const visitorKeys: {\n",
    );
    let mut children = String::from("// generated by tasks/ast_tools, do not edit\n");
    // Keep straight-line readers only for the most frequent kinds; the generic
    // walker plan covers the rest without the full child-visitors module.
    const HOT_READER_KINDS: &[&str] = &[
        "MemberExpression",
        "CallExpression",
        "ExpressionStatement",
        "VariableDeclarator",
        "BinaryExpression",
        "LogicalExpression",
        "AssignmentExpression",
    ];
    for (name, fields) in &keys {
        writeln!(js, "  {name}: Object.freeze({fields:?}),").unwrap();
        writeln!(dts, "  readonly {name}: readonly string[];").unwrap();
    }
    js.push_str("});\n");
    // Native IDs need wrappers only for hooks and unsupported projections.
    // Readers retain canonical ESTree field order and reuse the live list cursor.
    let mut reader_groups = BTreeMap::<String, usize>::new();
    let mut readers = [Vec::new(), Vec::new()];
    for ast in schema.types.iter().filter_map(|ty| match ty {
        AstType::Struct(ast) => Some(ast),
        _ => None,
    }) {
        let hot = HOT_READER_KINDS.contains(&ast.name.as_str());
        for (language, ts) in [false, true].into_iter().enumerate() {
            let body = if hot {
                projected_types.get(&ast.name).and_then(|name| {
                let mut body = String::from(
                    "  let version = ast._editVersion;\n  let row = ast.row(id), payload = ast._payloadAt(id, row);\n  let view = row === -1 ? ast._view : ast._arena.view;\n",
                );
                for (field_index, field) in keys[name].iter().enumerate() {
                    let rule = field_rule(ast, field, ts);
                    if matches!(rule, "Missing" | "Null" | "False" | "True") {
                        continue;
                    }
                    if lists[&(name.clone(), field.clone())] {
                        writeln!(body, "  if (walker.visitListField(id, {field:?}, plan)) {{ return true; }}").unwrap();
                        writeln!(body, "  if (ast._editVersion !== version) {{\n    version = ast._editVersion;\n    const liveKind = ast._kindAt(id);\n    if (liveKind !== kind) {{ return walker.visitRemaining(id, liveKind, plan, {}); }}\n    row = ast.row(id); payload = ast._payloadAt(id, row); view = row === -1 ? ast._view : ast._arena.view;\n  }}", field_index + 1).unwrap();
                        continue;
                    }
                    if rule != "Native" { return None; }
                    let (index, native) = ast.fields.iter().enumerate().find(|(_, native)| {
                        estree_field_name(&ast.name, &native.name) == *field
                    })?;
                    if !is_child(native.type_id, schema) || is_list(native.type_id, schema) {
                        return None;
                    }
                    let optional = matches!(schema.types[native.type_id], AstType::Option(_));
                    body.push_str("  {\n");
                    match ast.layout.fields[index] {
                        FieldLocation::Payload { offset, .. } => {
                            writeln!(body, "    const child = ast.childAt(id, payload + {offset}, {optional}, view);").unwrap();
                        }
                        FieldLocation::Extra { index } => {
                            writeln!(body, "    const start = view.getUint32(payload + {}, true);", ast.layout.extra_start_offset().unwrap()).unwrap();
                            writeln!(body, "    const extra = start + {index};").unwrap();
                            body.push_str("    const limit = start < ast._extraCount ? ast._extraCount : ast.extraDataCount;\n    if (extra + 1 > limit) { throw new RangeError(\"AST extra-data reference out of bounds\"); }\n    const extraView = extra < ast._extraCount ? ast._view : ast._arena.view;\n    const offset = extra < ast._extraCount ? ast._extra + extra * 4 : ast._arena.extraAddress(extra);\n");
                            writeln!(body, "    const child = ast.childAt(id, offset, {optional}, extraView);").unwrap();
                        }
                    }
                    writeln!(body, "    if (child !== 0xffffffff && walker.visitScalarId(child, id, {field:?})) {{ return true; }}").unwrap();
                    writeln!(body, "    if (ast._editVersion !== version) {{\n      version = ast._editVersion;\n      const liveKind = ast._kindAt(id);\n      if (liveKind !== kind) {{ return walker.visitRemaining(id, liveKind, plan, {}); }}\n      row = ast.row(id); payload = ast._payloadAt(id, row); view = row === -1 ? ast._view : ast._arena.view;\n    }}\n  }}", field_index + 1).unwrap();
                }
                body.push_str("  return false;\n");
                Some(body)
            })
            } else {
                None
            };
            let reader = body.map(|body| {
                if let Some(index) = reader_groups.get(&body) {
                    return *index;
                }
                let index = reader_groups.len();
                writeln!(
                    children,
                    "\nfunction readNativeChildren{index}(ast, id, walker, plan, kind) {{\n{body}}}"
                )
                .unwrap();
                reader_groups.insert(body, index);
                index
            });
            readers[language].push(reader);
        }
    }
    children.push_str("\nexport const childVisitors = [\n");
    for language in readers {
        children.push_str("  [\n");
        for reader in language {
            let name = reader.map_or_else(
                || "null".into(),
                |index| format!("readNativeChildren{index}"),
            );
            writeln!(children, "    {name},").unwrap();
        }
        children.push_str("  ],\n");
    }
    children.push_str("];\n");
    dts.push_str("};\nexport type NodeType = keyof typeof visitorKeys;\n");
    vec![
        RawOutput {
            path: "packages/walker2/child-visitors.js".into(),
            content: children.into_bytes(),
        },
        RawOutput {
            path: "packages/walker2/visitor-keys.js".into(),
            content: js.into_bytes(),
        },
        RawOutput {
            path: "packages/walker2/visitor-keys.d.ts".into(),
            content: dts.into_bytes(),
        },
    ]
}

fn is_child(id: TypeId, schema: &Schema) -> bool {
    match &schema.types[id] {
        AstType::Struct(_) | AstType::Enum(_) => true,
        AstType::Option(opt) => is_child(opt.inner_type_id, schema),
        AstType::SubRange(range) => is_child(range.inner_type_id, schema),
        AstType::Primitive(_) => false,
    }
}

fn is_list(id: TypeId, schema: &Schema) -> bool {
    match &schema.types[id] {
        AstType::SubRange(_) => true,
        AstType::Option(opt) => is_list(opt.inner_type_id, schema),
        _ => false,
    }
}
