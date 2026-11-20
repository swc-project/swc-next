use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write,
};

use super::estree::{estree_field_name, estree_type};
use super::ffi_protocol::field_rule;
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

// ESTree projections which flatten, merge, or synthesize internal AST nodes.
// Keep these aligned with the generated AST projection rules.
pub fn estree_visitor_keys(schema: &Schema) -> Vec<RawOutput> {
    let mut keys: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut lists = BTreeMap::new();
    let mut projected_types = BTreeMap::new();
    let mut concrete_fields = BTreeMap::new();
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
                if special.is_none()
                    && list
                    && let Some(child) = ast
                        .fields
                        .iter()
                        .find(|f| estree_field_name(ast.name.as_str(), &f.name) == *field)
                        .and_then(|f| concrete_child(f.type_id, schema))
                {
                    concrete_fields.insert((name.clone(), field.clone()), child.to_string());
                }
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
    let concrete_fields: BTreeMap<_, _> = concrete_fields
        .into_iter()
        .filter_map(|(field, child)| {
            projected_types
                .get(&child)
                .map(|projected| (field, projected.clone()))
        })
        .collect();
    let dispatch_types: BTreeSet<_> = concrete_fields.values().cloned().collect();
    let mut js = String::from(
        "// generated by tasks/ast_tools, do not edit\nexport const visitorKeys = Object.freeze({\n",
    );
    let mut dts = String::from(
        "// generated by tasks/ast_tools, do not edit\nexport declare const visitorKeys: {\n",
    );
    let mut children = String::from(
        "// generated by tasks/ast_tools, do not edit\nimport { NodeView } from \"@swc-next/ast\";\nexport const childVisitors = {\n",
    );
    for (name, fields) in &keys {
        writeln!(js, "  {name}: Object.freeze({fields:?}),").unwrap();
        writeln!(dts, "  readonly {name}: readonly string[];").unwrap();
        if fields.is_empty() {
            writeln!(children, "  {name}: null,").unwrap();
        } else {
            writeln!(children, "  {name}(node, walker) {{").unwrap();
            for (index, field) in fields.iter().enumerate() {
                let dispatch = concrete_fields
                    .get(&(name.clone(), field.clone()))
                    .map_or_else(String::new, |name| format!(", dispatch{name}"));
                if lists[&(name.clone(), field.clone())] {
                    let method = if dispatch.is_empty() {
                        "visitList"
                    } else {
                        "visitListKnown"
                    };
                    writeln!(children, "    if (node instanceof NodeView) {{\n      if (walker.visitListField(node, {field:?}{dispatch})) {{ return true; }}\n    }} else {{\n      const child{index} = node.{field};\n      if (child{index} != null && walker.{method}(child{index}, node, {field:?}{dispatch})) {{ return true; }}\n    }}").unwrap();
                } else {
                    writeln!(children, "    const child{index} = node.{field};\n    if (child{index} != null && walker.visitScalar(child{index}, node, {field:?})) {{ return true; }}").unwrap();
                }
            }
            writeln!(children, "    return false;\n  }},").unwrap();
        }
    }
    js.push_str("});\n");
    children.push_str("};\n");
    children.push_str("\nexport const nodeWalkers = Object.assign(Object.create(null), {\n");
    for (name, fields) in &keys {
        if fields.is_empty() {
            writeln!(children, "  {name}: null,").unwrap();
        } else {
            writeln!(children, "  {name}: walk{name},").unwrap();
        }
    }
    children.push_str("});\n");
    for (name, fields) in &keys {
        if fields.is_empty() {
            continue;
        }
        writeln!(
            children,
            "\nfunction walk{name}(node, walker, parent, key, list, index) {{"
        )
        .unwrap();
        for (index, field) in fields.iter().enumerate() {
            let dispatch = concrete_fields
                .get(&(name.clone(), field.clone()))
                .map_or_else(String::new, |name| format!(", dispatch{name}"));
            if lists[&(name.clone(), field.clone())] {
                let method = if dispatch.is_empty() {
                    "visitList"
                } else {
                    "visitListKnown"
                };
                writeln!(children, "  if (node instanceof NodeView) {{\n    if (walker.visitListField(node, {field:?}{dispatch})) {{ return index; }}\n  }} else {{\n    const child{index} = node.{field};\n    if (child{index} != null && walker.{method}(child{index}, node, {field:?}{dispatch})) {{ return index; }}\n  }}").unwrap();
            } else {
                writeln!(children, "  const child{index} = node.{field};\n  if (child{index} != null && walker.visitScalar(child{index}, node, {field:?})) {{ return index; }}").unwrap();
            }
        }
        writeln!(children, "  if (node.type !== {name:?}) {{ return walker.leaveChanged(node, parent, key, list, index); }}\n  return index;\n}}").unwrap();
    }
    for name in dispatch_types {
        writeln!(children, "\nconst dispatch{name} = function(node, walker, parent, key, list, index, walkers) {{\n  const type = node.type;\n  if (type !== {name:?}) {{\n    if (typeof type !== \"string\") {{ throw new TypeError(`Unsupported AST node type: ${{type}}`); }}\n    const fallback = walkers[type];\n    if (fallback === undefined) {{ throw new TypeError(`Unsupported AST node type: ${{type}}`); }}\n    return fallback === null ? index : fallback(node, walker, parent, key, list, index);\n  }}\n  const visit = walkers.{name};\n  return visit === null ? index : visit(node, walker, parent, key, list, index);\n}};\n").unwrap();
    }
    // Partition large same-length groups by the most distinguishing character.
    // Full-name guards preserve rejection of unsupported names; each known type
    // retains its own property/call feedback site. No corpus frequencies enter
    // the generator's choice of discriminator.
    let mut by_length: BTreeMap<usize, Vec<&String>> = BTreeMap::new();
    for name in keys.keys() {
        if name != "Identifier" {
            by_length.entry(name.len()).or_default().push(name);
        }
    }
    children.push_str("\nexport function dispatchNode(node, walker, parent, key, list, index, walkers) {\n  const type = node.type;\n");
    // Identifiers are common across JS and TS trees. Keep their existing full
    // walker, including annotation children, but avoid length partitioning.
    if keys.contains_key("Identifier") {
        children.push_str("  if (type === \"Identifier\") {\n    const visit = walkers.Identifier;\n    return visit === null ? index : visit(node, walker, parent, key, list, index);\n  }\n");
    }
    children.push_str("  switch (typeof type === \"string\" ? type.length : -1) {\n");
    for (length, names) in by_length {
        writeln!(children, "    case {length}:").unwrap();
        write_dispatch_group(&mut children, &names, "      ");
        children.push_str("      break;\n");
    }
    children.push_str("  }\n  throw new TypeError(`Unsupported AST node type: ${type}`);\n}\n");
    // Native IDs need wrappers only for hooks and unsupported projections.
    // Readers retain canonical ESTree field order and reuse the live list cursor.
    let mut reader_groups = BTreeMap::<String, usize>::new();
    let mut readers = [Vec::new(), Vec::new()];
    for ast in schema.types.iter().filter_map(|ty| match ty {
        AstType::Struct(ast) => Some(ast),
        _ => None,
    }) {
        for (language, ts) in [false, true].into_iter().enumerate() {
            let body = projected_types.get(&ast.name).and_then(|name| {
                let mut body = String::from("  const version = ast._editVersion;\n");
                for field in &keys[name] {
                    let rule = field_rule(ast, field, ts);
                    if matches!(rule, "Missing" | "Null" | "False" | "True") {
                        continue;
                    }
                    if lists[&(name.clone(), field.clone())] {
                        writeln!(body, "  if (walker.visitListField(id, {field:?})) {{ return true; }}").unwrap();
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
                    body.push_str("  {\n    if (ast._editVersion !== version) {\n");
                    writeln!(body, "      const node = ast.node(id), child = node.{field};\n      if (child != null && walker.visitScalar(child, node, {field:?})) {{ return true; }}\n    }} else {{").unwrap();
                    body.push_str("      const row = ast.row(id), payload = ast._payloadAt(id, row);\n      let view = row === -1 ? ast._view : ast._arena.view;\n");
                    match ast.layout.fields[index] {
                        FieldLocation::Payload { offset, .. } => {
                            writeln!(body, "      const offset = payload + {offset};").unwrap();
                        }
                        FieldLocation::Extra { index } => {
                            writeln!(body, "      const extra = view.getUint32(payload + {}, true) + {index};", ast.layout.extra_start_offset().unwrap()).unwrap();
                            body.push_str("      if (extra >= ast.extraDataCount) { throw new RangeError(\"AST extra-data reference out of bounds\"); }\n      const offset = extra < ast._extraCount ? ast._extra + extra * 4 : ast._arena.extraAddress(extra);\n      view = extra < ast._extraCount ? ast._view : ast._arena.view;\n");
                        }
                    }
                    writeln!(body, "      const child = ast.childAt(id, offset, {optional}, view);\n      if (child !== 0xffffffff && walker.visitScalarId(child, id, {field:?})) {{ return true; }}\n    }}\n  }}").unwrap();
                }
                body.push_str("  return false;\n");
                Some(body)
            });
            let reader = body.map(|body| {
                if let Some(index) = reader_groups.get(&body) {
                    return *index;
                }
                let index = reader_groups.len();
                writeln!(
                    children,
                    "\nfunction readNativeChildren{index}(ast, id, walker) {{\n{body}}}"
                )
                .unwrap();
                reader_groups.insert(body, index);
                index
            });
            readers[language].push(reader);
        }
    }
    children.push_str("\nexport const nativeChildVisitors = [\n");
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
            path: "packages/walker/child-visitors.js".into(),
            content: children.into_bytes(),
        },
        RawOutput {
            path: "packages/walker/visitor-keys.js".into(),
            content: js.into_bytes(),
        },
        RawOutput {
            path: "packages/walker/visitor-keys.d.ts".into(),
            content: dts.into_bytes(),
        },
    ]
}

fn write_dispatch_group(out: &mut String, names: &[&String], indent: &str) {
    if names.len() <= 16 {
        writeln!(out, "{indent}switch (type) {{").unwrap();
        for name in names {
            writeln!(out, "{indent}  case {name:?}: {{\n{indent}    const visit = walkers.{name};\n{indent}    return visit === null ? index : visit(node, walker, parent, key, list, index);\n{indent}  }}").unwrap();
        }
        writeln!(out, "{indent}}}").unwrap();
        return;
    }
    let position = (0..names[0].len())
        .min_by_key(|&position| {
            let mut counts = BTreeMap::<u8, usize>::new();
            for name in names {
                *counts.entry(name.as_bytes()[position]).or_default() += 1;
            }
            counts.values().map(|count| count * count).sum::<usize>()
        })
        .unwrap();
    let mut groups = BTreeMap::<u8, Vec<&String>>::new();
    for name in names {
        groups
            .entry(name.as_bytes()[position])
            .or_default()
            .push(name);
    }
    writeln!(out, "{indent}switch (type.charCodeAt({position})) {{").unwrap();
    for (code, group) in groups {
        writeln!(out, "{indent}  case {code}:").unwrap();
        write_dispatch_group(out, &group, &format!("{indent}    "));
        writeln!(out, "{indent}    break;").unwrap();
    }
    writeln!(out, "{indent}}}").unwrap();
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

fn concrete_child(id: TypeId, schema: &Schema) -> Option<&str> {
    match &schema.types[id] {
        AstType::Struct(ast) => Some(&ast.name),
        AstType::Option(opt) => concrete_child(opt.inner_type_id, schema),
        AstType::SubRange(range) => concrete_child(range.inner_type_id, schema),
        _ => None,
    }
}
