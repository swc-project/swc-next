//! Legacy SWC TypeScript erasure, runtime lowering, and TSX import retention.

mod options;

pub use options::{
    ImportsNotUsedAsValues, JsxImportOptions, TsImportExportAssignConfig, TypeScriptOptions,
};

mod classes;
mod flow;
mod modules;
mod retain;
mod runtime;
mod semantic;
mod transform;
mod ts_enum;
mod utils;

use std::collections::HashSet;
use swc_next_ecma_ast::*;

use crate::resolver::{Id, Resolver};

/// SWC's TypeScript pipeline before namespace/enum binding changes in #12455.
/// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/typescript.rs#L43>.
pub(crate) fn transform(
    ast: &mut Ast<'_>,
    options: &TypeScriptOptions,
    tsx: bool,
    resolver: &mut Resolver,
) -> Vec<Diagnostic> {
    let program = ast.root_program();
    let last_module = (!options.flow_syntax && !options.no_empty_export && ast.is_module())
        .then(|| {
            program
                .body(ast)
                .iter()
                .rev()
                .map(|slot| ast.get_node_in_sub_range(slot))
                .find(|&statement| es_module(ast, statement))
                .map(|statement| statement.span(ast))
        })
        .flatten();
    let seed = if tsx && ast.is_module() && !options.verbatim_module_syntax {
        jsx_usage(ast, options, resolver)
    } else {
        HashSet::new()
    };
    let semantic = semantic::analyze(ast, resolver, options, seed);
    let mut transform = transform::Transform::new(ast, resolver, options, semantic);
    transform.program();
    let diagnostics = transform.diagnostics;
    if let Some(span) = last_module
        && !program
            .body(ast)
            .iter()
            .any(|slot| es_module(ast, ast.get_node_in_sub_range(slot)))
    {
        let mut body: Vec<_> = program
            .body(ast)
            .iter()
            .map(|slot| ast.get_node_in_sub_range(slot))
            .collect();
        body.push(Stmt::ExportNamedDeclaration(ast.export_named_declaration(
            span,
            None,
            SubRange::empty(),
            None,
            SubRange::empty(),
            ImportOrExportKind::Value,
        )));
        let body = ast.add_sub_range(body);
        program.set_body(ast, body);
    }
    let lang = match ast.lang() {
        Lang::Ts => Lang::Js,
        Lang::Tsx => Lang::Jsx,
        lang => lang,
    };
    ast.set_source_metadata(ast.source_type(), lang);
    resolver.register(ast);
    diagnostics
}

fn es_module(ast: &Ast<'_>, statement: Stmt) -> bool {
    match ast.stmt_data(statement) {
        StmtData::ImportDeclaration(_)
        | StmtData::ExportDefaultDeclaration(_)
        | StmtData::ExportAllDeclaration(_) => true,
        StmtData::ExportNamedDeclaration(export) => !export
            .declaration(ast)
            .is_some_and(|declaration| declaration.is_ts_import_equals_declaration(ast)),
        _ => false,
    }
}

/// SWC TSX conservatively retains both configured factory roots, even when no
/// element or fragment needs them. Leading pragma comments add their roots.
/// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/typescript.rs#L186>.
fn jsx_usage(ast: &Ast<'_>, options: &TypeScriptOptions, resolver: &Resolver) -> HashSet<Id> {
    let root = |value: &str| -> Option<Id> {
        let name = value.split('.').next()?.trim();
        (name != "this").then(|| Id {
            name: name.into(),
            scope: resolver.top_level(),
        })
    };
    let mut usage = HashSet::new();
    if let Some(id) = root(
        options
            .jsx
            .pragma
            .as_deref()
            .unwrap_or("React.createElement"),
    ) {
        usage.insert(id);
    }
    if let Some(id) = root(
        options
            .jsx
            .pragma_frag
            .as_deref()
            .unwrap_or("React.Fragment"),
    ) {
        usage.insert(id);
    }
    let program = ast.root_program();
    // SWC's module span starts at its first token; Next's program span starts
    // at byte zero, so include the first directive/body attachment explicitly.
    let start = program
        .directives(ast)
        .iter()
        .map(|slot| ast.get_node_in_sub_range(slot).span(ast).start)
        .chain(
            program
                .body(ast)
                .iter()
                .map(|slot| ast.get_node_in_sub_range(slot).span(ast).start),
        )
        .next()
        .unwrap_or(program.span(ast).start);
    for comment in ast.comments() {
        if (comment.attached_to != start && comment.attached_to != program.span(ast).start)
            || comment.position != CommentPosition::Leading
            || comment.kind != CommentKind::Block
        {
            continue;
        }
        for line in ast.get_utf8(comment.value(ast.source().as_bytes())).lines() {
            let line = line.trim().trim_start_matches('*').trim();
            if !line.starts_with("@jsx") {
                continue;
            }
            let mut words = line.split_whitespace();
            while let Some(directive) = words.next() {
                let value = words.next();
                if matches!(directive, "@jsx" | "@jsxFrag")
                    && let Some(value) = value
                    && let Some(id) = root(value)
                {
                    usage.insert(id);
                }
            }
        }
    }
    usage
}
