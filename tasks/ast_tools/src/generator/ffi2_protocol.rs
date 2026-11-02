use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write,
};

use super::{
    ast_node::concrete_node_kinds,
    estree::{ENUMS, estree_field_name, estree_type, is_ts_field, ts_extra_fields},
    estree_visitor_keys2::public_types,
    ffi2_projection_spec as spec,
};
use crate::{
    output::RawOutput,
    schema::{AstStruct, AstType, FieldLocation, Schema},
};

fn native_range(ast: &AstStruct, key: &str, schema: &Schema) -> Option<(i32, i32)> {
    let (index, _) = ast.fields.iter().enumerate().find(|(_, field)| {
        estree_field_name(&ast.name, &field.name) == key
            && matches!(schema.types[field.type_id], AstType::SubRange(_))
    })?;
    Some(match ast.layout.fields[index] {
        FieldLocation::Payload { offset, .. } => (-1, offset as i32),
        FieldLocation::Extra { index } => (
            ast.layout.extra_start_offset().unwrap() as i32,
            index as i32,
        ),
    })
}

pub(super) fn keys(ast: &AstStruct, ts: bool) -> Vec<String> {
    let mut fields = vec!["type".into(), "start".into(), "end".into()];
    if let Some((_, special)) = spec::PUBLIC_KEYS.iter().find(|(name, _)| *name == ast.name) {
        fields.extend(special[0].iter().map(|field| (*field).to_owned()));
        if ts {
            fields.extend(special[1].iter().map(|field| (*field).to_owned()));
        }
    } else {
        for field in &ast.fields {
            let name = estree_field_name(&ast.name, &field.name);
            if !field.attrs.estree_skip && (ts || !is_ts_field(&ast.name, &name)) {
                fields.push(name);
            }
        }
        if ts {
            fields.extend(
                ts_extra_fields(&ast.name)
                    .iter()
                    .map(|(field, _)| (*field).to_owned()),
            );
        }
    }
    let mut seen = BTreeSet::new();
    fields.retain(|field| seen.insert(field.clone()));
    fields
}

fn intern<T: Eq>(values: &mut Vec<T>, value: T) -> usize {
    if let Some(index) = values.iter().position(|existing| *existing == value) {
        index
    } else {
        values.push(value);
        values.len() - 1
    }
}

fn write_ids(output: &mut String, name: &str, ids: &[u8]) {
    writeln!(output, "export const {name} = new Uint8Array([").unwrap();
    for row in ids.chunks(32) {
        writeln!(
            output,
            "  {},",
            row.iter().map(u8::to_string).collect::<Vec<_>>().join(", ")
        )
        .unwrap();
    }
    output.push_str("]);\n");
}

pub(super) fn field_rule(ast: &AstStruct, field: &str, ts: bool) -> &'static str {
    if !keys(ast, ts).iter().any(|key| key == field) {
        return "Missing";
    }
    let mut rule = "Native";
    if ts
        && !ast.fields.iter().any(|native| {
            !native.attrs.estree_skip && estree_field_name(&ast.name, &native.name) == field
        })
        && let Some((_, value)) = ts_extra_fields(&ast.name)
            .iter()
            .find(|(name, _)| *name == field)
    {
        rule = match *value {
            "false" => "False",
            "true" => "True",
            "null" => "Null",
            "[]" => "List",
            "\"value\"" => "Value",
            _ => panic!("unsupported projection default {value}"),
        };
    }
    if let Some((_, _, override_rule)) = spec::FIELD_RULES
        .iter()
        .find(|(name, key, _)| *name == ast.name && *key == field)
    {
        rule = override_rule;
    }
    if field == "params" && spec::FUNCTIONS.contains(&ast.name.as_str()) {
        rule = "Params";
    }
    rule
}

/// Public ESTree names produced by a dynamic native node. Visitor-key
/// projections are the shared source for names; FormalParameter additionally
/// synthesizes AssignmentPattern from its initializer and maps its pattern
/// child kinds to their public names.
fn dynamic_public_types(ast: &AstStruct, schema: &Schema) -> Vec<String> {
    let name = ast.name.as_str();
    let mut types = if name == "FormalParameter" {
        assert!(
            ast.fields.iter().any(|field| field.name == "initializer"),
            "FormalParameter must have an initializer field"
        );
        let pattern = ast
            .fields
            .iter()
            .find(|field| field.name == "pattern")
            .expect("FormalParameter must have a pattern field");
        let mut types = vec!["AssignmentPattern".to_owned()];
        types.extend(
            concrete_node_kinds(pattern.type_id, schema)
                .iter()
                .map(|kind| estree_type(kind)),
        );
        types
    } else {
        public_types(name)
            .unwrap_or_else(|| panic!("dynamic AST type `{name}` has no public projection"))
    };
    types.sort();
    types.dedup();
    types
}

pub fn protocol(schema: &Schema) -> RawOutput {
    let nodes: Vec<_> = schema
        .types
        .iter()
        .filter_map(|ty| match ty {
            AstType::Struct(ast) => Some(ast),
            _ => None,
        })
        .collect();
    let keys: Vec<_> = [false, true]
        .into_iter()
        .map(|ts| nodes.iter().map(|ast| keys(ast, ts)).collect::<Vec<_>>())
        .collect();
    let mut names = BTreeSet::new();
    for ast in &nodes {
        for field in &ast.fields {
            if !field.attrs.estree_skip {
                names.insert(estree_field_name(&ast.name, &field.name));
            }
        }
    }
    for language in &keys {
        for fields in language {
            names.extend(fields.iter().cloned());
        }
    }
    let names: Vec<_> = names.into_iter().collect();
    let ids: BTreeMap<_, _> = names
        .iter()
        .enumerate()
        .map(|(index, name)| (name.as_str(), index))
        .collect();
    let rule_ids: BTreeMap<_, _> = spec::READ_RULES
        .iter()
        .enumerate()
        .map(|(index, name)| (*name, index as u8))
        .collect();
    let width = names.len();
    let mut native_names = vec![BTreeSet::<String>::new(); nodes.len()];
    let mut initial_fields = vec![Vec::<String>::new(); nodes.len()];
    for (kind, ast) in nodes.iter().enumerate() {
        for field in &ast.fields {
            if field.attrs.estree_skip {
                continue;
            }
            let name = estree_field_name(&ast.name, &field.name);
            let (ty, optional) = match &schema.types[field.type_id] {
                AstType::Option(option) => (&schema.types[option.inner_type_id], true),
                ty => (ty, false),
            };
            if matches!(ty, AstType::Struct(_) | AstType::Enum(_))
                || optional
                    && matches!(ty, AstType::Primitive(primitive) if matches!(primitive.name, "AstUtf8" | "AstWtf8") || ENUMS.iter().any(|(name, _)| *name == primitive.name))
            {
                initial_fields[kind].push(name.clone());
            }
            native_names[kind].insert(name);
        }
    }
    let mut rules = vec![0u8; nodes.len() * width * 2];
    let mut cursors = vec![0u16; rules.len()];
    let mut recipes: Vec<[i32; 5]> = vec![[0; 5]];
    let formal = nodes
        .iter()
        .find(|ast| ast.name == "FormalParameters")
        .unwrap();
    for (language, ts) in [false, true].into_iter().enumerate() {
        for (kind, ast) in nodes.iter().enumerate() {
            for field in &keys[language][kind] {
                let slot = (language * nodes.len() + kind) * width + ids[field.as_str()];
                let rule = field_rule(ast, field, ts);
                rules[slot] = rule_ids[rule];

                let mut recipe = [-2, 0, -2, 0, 0];
                if field == "params" && spec::FUNCTIONS.contains(&ast.name.as_str()) {
                    let (pointer, offset) = native_range(formal, "items", schema).unwrap();
                    recipe[0] = pointer;
                    recipe[1] = offset;
                    recipe[4] = 4;
                } else if field == "body"
                    && matches!(
                        ast.name.as_str(),
                        "Program" | "FunctionBody" | "TsModuleBlock"
                    )
                {
                    let (p0, o0) = native_range(ast, "directives", schema).unwrap();
                    let (p1, o1) = native_range(ast, "body", schema).unwrap();
                    recipe = [p0, o0, p1, o1, 0];
                } else if matches!(
                    (ast.name.as_str(), field.as_str()),
                    ("ArrayPattern", "elements") | ("ObjectPattern", "properties")
                ) {
                    let (pointer, offset) = native_range(ast, field, schema).unwrap();
                    recipe[0] = pointer;
                    recipe[1] = offset;
                    recipe[4] = 2;
                } else if ts && field == "decorators" && spec::PATTERNS.contains(&ast.name.as_str())
                {
                    continue;
                } else if ast.name == "TsThisParameter" && field == "decorators"
                    || ts
                        && ts_extra_fields(&ast.name)
                            .iter()
                            .any(|(name, value)| *name == field && *value == "[]")
                {
                    recipe[4] = 8;
                } else if let Some((pointer, offset)) = native_range(ast, field, schema) {
                    recipe[0] = pointer;
                    recipe[1] = offset;
                } else {
                    continue;
                }
                let index = recipes
                    .iter()
                    .position(|existing| *existing == recipe)
                    .unwrap_or_else(|| {
                        recipes.push(recipe);
                        recipes.len() - 1
                    });
                cursors[slot] = index.try_into().unwrap();
            }
        }
    }
    let mut field_maps = vec![BTreeMap::<String, u8>::new()];
    let mut field_map_ids = Vec::<u8>::new();
    let cursor_fields = names
        .iter()
        .enumerate()
        .filter_map(|(field, name)| {
            (0..nodes.len() * 2)
                .any(|row| cursors[row * width + field] != 0)
                .then_some((field, name))
        })
        .collect::<Vec<_>>();
    let cursor_width = cursor_fields.len();
    let mut cursor_rows = Vec::<Vec<u8>>::new();
    let mut cursor_map_ids = Vec::<u8>::new();
    let mut key_tables = Vec::<Vec<String>>::new();
    let mut key_ids = Vec::<u8>::new();
    for language in 0..2 {
        for kind in 0..nodes.len() {
            let mut field_map = BTreeMap::new();
            for (field_id, field) in names.iter().enumerate() {
                if matches!(field.as_str(), "type" | "start" | "end") {
                    continue;
                }
                let slot = (language * nodes.len() + kind) * width + field_id;
                let rule = rules[slot];
                // Native fields use schema.ts directly. Only projections and
                // hidden native fields need an override.
                if rule != rule_ids["Native"]
                    && (rule != rule_ids["Missing"] || native_names[kind].contains(field))
                {
                    field_map.insert(field.clone(), rule);
                }
            }
            field_map_ids.push(intern(&mut field_maps, field_map).try_into().unwrap());
            let row = cursor_fields
                .iter()
                .map(|(field, _)| {
                    cursors[(language * nodes.len() + kind) * width + field]
                        .try_into()
                        .unwrap()
                })
                .collect();
            cursor_map_ids.push(intern(&mut cursor_rows, row).try_into().unwrap());
            key_ids.push(
                intern(&mut key_tables, keys[language][kind].clone())
                    .try_into()
                    .unwrap(),
            );
        }
    }
    let rule_fields = field_maps
        .iter()
        .flat_map(|row| row.keys().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let mut output = String::from(
        "// Generated by ast_tools. Do not edit.\nimport type { AstMeta } from './types.ts';\n",
    );
    writeln!(output, "export const NODE_KIND_COUNT = {};", nodes.len()).unwrap();
    writeln!(output, "export const FIELD_COUNT = {};\nexport const fieldIds: Readonly<Record<string, number>> = {{", rule_fields.len()).unwrap();
    for (id, name) in rule_fields.iter().enumerate() {
        writeln!(output, "  {name:?}: {id},").unwrap();
    }
    output.push_str("};\nexport const ReadRule = {\n");
    for (name, id) in &rule_ids {
        writeln!(output, "  {name}: {id},").unwrap();
    }
    output.push_str("} as const;\n");
    let mut initial_field_counts = BTreeMap::<Vec<String>, usize>::new();
    for fields in &initial_fields {
        *initial_field_counts.entry(fields.clone()).or_default() += 1;
    }
    let mut initial_field_ids = BTreeMap::<Vec<String>, usize>::new();
    let mut initial_field_consts = String::new();
    let mut initial_field_refs = Vec::with_capacity(initial_fields.len());
    for fields in &initial_fields {
        if initial_field_counts[fields] > 1 {
            let next = initial_field_ids.len();
            let id = *initial_field_ids.entry(fields.clone()).or_insert(next);
            if id == next {
                if fields.is_empty() {
                    writeln!(initial_field_consts, "const I{id}: readonly string[] = [];").unwrap();
                } else {
                    writeln!(initial_field_consts, "const I{id} = {fields:?};").unwrap();
                }
            }
            initial_field_refs.push(Some(id));
        } else {
            initial_field_refs.push(None);
        }
    }
    output.push_str(&initial_field_consts);
    output.push_str("export const initialFields: readonly (readonly string[])[] = [\n");
    for (kind, fields) in initial_fields.iter().enumerate() {
        match initial_field_refs[kind] {
            Some(id) => writeln!(output, "  I{id}, // {}", nodes[kind].name).unwrap(),
            None => writeln!(output, "  {fields:?}, // {}", nodes[kind].name).unwrap(),
        }
    }
    output.push_str("];\n");
    output.push_str("export const fieldRules = new Uint8Array([\n");
    for row in &field_maps {
        let values = rule_fields
            .iter()
            .map(|field| {
                row.get(field)
                    .copied()
                    .unwrap_or(rule_ids["Native"])
                    .to_string()
            })
            .collect::<Vec<_>>();
        writeln!(output, "  {},", values.join(", ")).unwrap();
    }
    output.push_str("]);\n");
    output.push_str("// ID arrays contain JS rows followed by TS rows, indexed by NodeKind.\n");
    write_ids(&mut output, "fieldRuleIds", &field_map_ids);
    writeln!(output, "export const CURSOR_FIELD_COUNT = {cursor_width};\nexport const cursorFieldIds: Readonly<Record<string, number>> = {{").unwrap();
    for (index, (_, field)) in cursor_fields.iter().enumerate() {
        writeln!(output, "  {field:?}: {index},").unwrap();
    }
    output.push_str("};\nexport const cursorRules = new Uint8Array([\n");
    for row in &cursor_rows {
        writeln!(
            output,
            "  {},",
            row.iter().map(u8::to_string).collect::<Vec<_>>().join(", ")
        )
        .unwrap();
    }
    output.push_str("]);\n");
    write_ids(&mut output, "cursorRuleIds", &cursor_map_ids);
    writeln!(output, "export const cursorRecipes: readonly (readonly [number,number,number,number,number])[] = {recipes:?};").unwrap();
    output.push_str("export const keyTables: readonly (readonly string[])[] = [\n");
    for fields in &key_tables {
        writeln!(output, "  {fields:?},").unwrap();
    }
    output.push_str("];\n");
    write_ids(&mut output, "keyIds", &key_ids);
    for name in spec::DYNAMIC_TYPES {
        assert!(
            nodes.iter().any(|ast| ast.name == *name),
            "dynamic AST type `{name}` is not a native node"
        );
    }
    output.push_str("export const staticTypes: readonly (string | null)[] = [\n");
    for ast in &nodes {
        let name = ast.name.as_str();
        let value = if spec::DYNAMIC_TYPES.contains(&name) {
            "null".into()
        } else if matches!(
            name,
            "StringLiteral"
                | "NumericLiteral"
                | "BigIntLiteral"
                | "BooleanLiteral"
                | "NullLiteral"
                | "RegExpLiteral"
        ) {
            "\"Literal\"".into()
        } else {
            format!(
                "{:?}",
                spec::TYPE_OVERRIDES
                    .iter()
                    .find(|(native, _)| *native == name)
                    .map(|(_, value)| value.to_string())
                    .unwrap_or_else(|| estree_type(name))
            )
        };
        writeln!(output, "  {value},").unwrap();
    }
    output.push_str("];\n");
    output.push_str(
        "export const dynamicTypes: readonly (readonly [number, readonly string[]])[] = [\n",
    );
    for (kind, ast) in nodes.iter().enumerate() {
        let name = ast.name.as_str();
        if !spec::DYNAMIC_TYPES.contains(&name) {
            continue;
        }
        let types = dynamic_public_types(ast, schema);
        writeln!(output, "  [{kind}, {types:?}], // {name}").unwrap();
    }
    output.push_str("];\n");
    output.push_str("export const typeKinds: Readonly<Record<string, number>> = {\n  Literal: ");
    writeln!(
        output,
        "{},",
        nodes
            .iter()
            .position(|ast| ast.name == "NullLiteral")
            .unwrap()
    )
    .unwrap();
    let mut assigned_types = BTreeSet::from(["Literal".to_owned()]);
    for (kind, ast) in nodes.iter().enumerate() {
        let name = ast.name.as_str();
        if spec::DYNAMIC_TYPES.contains(&name)
            || matches!(
                name,
                "StringLiteral"
                    | "NumericLiteral"
                    | "BigIntLiteral"
                    | "BooleanLiteral"
                    | "NullLiteral"
                    | "RegExpLiteral"
                    | "BindingIdentifier"
                    | "IdentifierName"
                    | "FunctionBody"
                    | "FormalParameter"
                    | "FormalParameterRest"
                    | "Directive"
                    | "BindingProperty"
                    | "TsGlobalDeclaration"
                    | "TsThisParameter"
                    | "TsIndexSignatureParameter"
            )
        {
            continue;
        }
        let public = spec::TYPE_OVERRIDES
            .iter()
            .find(|(native, _)| *native == name)
            .map(|(_, value)| value.to_string())
            .unwrap_or_else(|| estree_type(name));
        if !assigned_types.insert(public.clone()) {
            continue;
        }
        writeln!(output, "  {public:?}: {kind},").unwrap();
    }
    output.push_str("};\nexport const kindFlags = new Uint8Array([\n");
    for ast in &nodes {
        let name = ast.name.as_str();
        let flags = u8::from(spec::PATTERNS.contains(&name))
            | (u8::from(spec::FUNCTIONS.contains(&name)) << 1)
            | (u8::from(matches!(
                name,
                "StringLiteral"
                    | "NumericLiteral"
                    | "BigIntLiteral"
                    | "BooleanLiteral"
                    | "NullLiteral"
                    | "RegExpLiteral"
            )) << 2);
        writeln!(output, "  {flags},").unwrap();
    }
    output.push_str("]);\n");
    output.push_str("export const parserMeta: readonly AstMeta[] = [\n");
    let languages = ENUMS.iter().find(|(name, _)| *name == "Lang").unwrap().1;
    let source_types = ENUMS
        .iter()
        .find(|(name, _)| *name == "SourceType")
        .unwrap()
        .1;
    for lang in languages {
        for source_type in source_types {
            writeln!(
                output,
                "  Object.freeze({{ lang: {lang}, sourceType: {source_type} }}),"
            )
            .unwrap();
        }
    }
    output.push_str("];\n");
    RawOutput {
        path: "packages/ast2/protocol.ts".into(),
        content: output.into_bytes(),
    }
}
