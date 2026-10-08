use std::{collections::BTreeMap, fmt::Write};

use super::estree_decoder::{estree_field_name, estree_type};
use crate::{
    output::RawOutput,
    schema::{AstType, Schema, TypeId},
};

// ESTree projections which flatten, merge, or synthesize internal AST nodes.
// Keep these aligned with write_special_node in estree_decoder.
pub fn estree_visitor_keys(schema: &Schema) -> Vec<RawOutput> {
    let mut keys: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for ty in &schema.types {
        let AstType::Struct(ast) = ty else { continue };
        let name = ast.name.as_str();
        let (names, special): (Vec<String>, Option<&[&str]>) = match name {
            "FormalParameter" | "FormalParameters" | "TsIndexSignatureParameter" => continue,
            "Program" => (vec!["Program".into()], Some(&["body"])),
            "FunctionBody" => (vec!["BlockStatement".into()], Some(&["body"])),
            "TsModuleBlock" => (vec!["TSModuleBlock".into()], Some(&["body"])),
            "Function" => (
                vec!["FunctionDeclaration".into(), "FunctionExpression".into()],
                Some(&["id", "typeParameters", "params", "returnType", "body"]),
            ),
            "TsFunction" => (
                vec![
                    "TSDeclareFunction".into(),
                    "TSEmptyBodyFunctionExpression".into(),
                ],
                Some(&["id", "typeParameters", "params", "returnType", "body"]),
            ),
            "ArrowFunctionExpression" => (
                vec![name.into()],
                Some(&["typeParameters", "params", "returnType", "body"]),
            ),
            "Class" => (
                vec!["ClassDeclaration".into(), "ClassExpression".into()],
                Some(&[
                    "decorators",
                    "id",
                    "typeParameters",
                    "superClass",
                    "superTypeArguments",
                    "implements",
                    "body",
                ]),
            ),
            "MethodDefinition" | "TsMethodDefinition" => (
                vec![
                    "MethodDefinition".into(),
                    "TSAbstractMethodDefinition".into(),
                ],
                Some(&["decorators", "key", "value"]),
            ),
            "PropertyDefinition" => (
                vec![
                    "PropertyDefinition".into(),
                    "AccessorProperty".into(),
                    "TSAbstractPropertyDefinition".into(),
                    "TSAbstractAccessorProperty".into(),
                ],
                Some(&["decorators", "key", "typeAnnotation", "value"]),
            ),
            "Directive" => (vec!["ExpressionStatement".into()], Some(&["expression"])),
            "StringLiteral" | "NumericLiteral" | "BigIntLiteral" | "BooleanLiteral"
            | "NullLiteral" | "RegExpLiteral" => (vec!["Literal".into()], Some(&[])),
            "TemplateElement" => (vec![name.into()], Some(&[])),
            "FormalParameterRest" | "BindingRestElement" => (
                vec!["RestElement".into()],
                Some(&["decorators", "argument", "typeAnnotation"]),
            ),
            "BindingProperty" | "ObjectProperty" => {
                (vec!["Property".into()], Some(&["key", "value"]))
            }
            "ArrayPattern" => (
                vec![name.into()],
                Some(&["decorators", "elements", "typeAnnotation"]),
            ),
            "ObjectPattern" => (
                vec![name.into()],
                Some(&["decorators", "properties", "typeAnnotation"]),
            ),
            "AssignmentPattern" => (
                vec![name.into()],
                Some(&["decorators", "left", "typeAnnotation", "right"]),
            ),
            "VariableDeclarator" => (vec![name.into()], Some(&["id", "init"])),
            "CatchClause" => (vec![name.into()], Some(&["param", "body"])),
            "TsThisParameter" => (
                vec!["Identifier".into()],
                Some(&["decorators", "typeAnnotation"]),
            ),
            "TsGlobalDeclaration" => (vec!["TSModuleDeclaration".into()], Some(&["id", "body"])),
            "TsFunctionType"
            | "TsConstructorType"
            | "TsCallSignatureDeclaration"
            | "TsConstructSignatureDeclaration" => (
                vec![estree_type(name)],
                Some(&["typeParameters", "params", "returnType"]),
            ),
            "TsMethodSignature" => (
                vec![estree_type(name)],
                Some(&["key", "typeParameters", "params", "returnType"]),
            ),
            _ => (vec![estree_type(name)], None),
        };
        let fields: Vec<String> = special
            .map(|fields| fields.iter().map(|s| (*s).into()).collect())
            .unwrap_or_else(|| {
                ast.fields
                    .iter()
                    .filter(|f| !f.attrs.estree_skip && is_child(f.type_id, schema))
                    .map(|f| estree_field_name(name, &f.name))
                    .collect()
            });
        for name in names {
            let entry = keys.entry(name).or_default();
            for field in &fields {
                if !entry.contains(field) {
                    entry.push(field.clone());
                }
            }
        }
    }
    let mut js = String::from(
        "// generated by tasks/ast_tools, do not edit\nexport const visitorKeys = Object.freeze({\n",
    );
    let mut dts = String::from(
        "// generated by tasks/ast_tools, do not edit\nexport declare const visitorKeys: {\n",
    );
    for (name, fields) in keys {
        writeln!(js, "  {name}: Object.freeze({fields:?}),").unwrap();
        writeln!(dts, "  readonly {name}: readonly string[];").unwrap();
    }
    js.push_str("});\n");
    dts.push_str("};\nexport type NodeType = keyof typeof visitorKeys;\n");
    vec![
        RawOutput {
            path: "npm/core/src-js/visitor-keys.js".into(),
            content: js.into_bytes(),
        },
        RawOutput {
            path: "npm/core/src-js/visitor-keys.d.ts".into(),
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
