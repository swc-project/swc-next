//! Legacy SWC TypeScript erasure, runtime lowering, and TSX import retention.

use std::collections::HashSet;

use swc_next_ecma_ast::*;

use crate::{
    common::resolver::context::Resolver,
    jsx::{JsxDirectives, JsxRuntime},
    module::ModuleType,
};

mod config;
mod retain;
mod semantic;
mod shared;
mod transform;
mod ts_enum;
mod utils;

pub use config::{
    Config as TypeScriptOptions, ImportsNotUsedAsValues, TsImportExportAssignConfig,
    TsxConfig as JsxImportOptions,
};

/// SWC's TypeScript pipeline before namespace/enum binding changes in #12455.
/// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/typescript.rs#L43>.
pub(crate) fn typescript(
    ast: &mut Ast<'_>,
    mut options: TypeScriptOptions,
    tsx: bool,
    header_end: u32,
    module_type: ModuleType,
    resolver: &mut Resolver,
) -> Vec<Diagnostic> {
    if ast.source_type() == SourceType::CommonJs {
        options.import_export_assign_config = TsImportExportAssignConfig::Classic;
    } else if module_type != ModuleType::Preserve && ast.is_module() {
        options.import_export_assign_config = TsImportExportAssignConfig::Preserve;
    }
    let pass = TypeScript { config: &options };
    if tsx {
        TypeScriptReact { typescript: pass }.process(ast, resolver, header_end)
    } else {
        pass.process(ast, resolver, HashSet::new())
    }
}

struct TypeScript<'a> {
    config: &'a TypeScriptOptions,
}

impl TypeScript<'_> {
    fn process(
        &self,
        ast: &mut Ast<'_>,
        resolver: &mut Resolver,
        id_usage: HashSet<SymbolId>,
    ) -> Vec<Diagnostic> {
        let options = self.config;
        let last_module = self.get_last_module_span(ast);
        let semantic = semantic::analyze_program(ast, resolver, options, id_usage);
        let mut transform = transform::Transform::new(ast, resolver, options, semantic);
        transform.visit_mut_program();
        let diagnostics = transform.diagnostics;
        Self::restore_esm_ctx(ast, last_module);
        let lang = match ast.lang() {
            Lang::Ts => Lang::Js,
            Lang::Tsx => Lang::Jsx,
            lang => lang,
        };
        ast.set_source_metadata(ast.source_type(), lang);
        diagnostics
    }

    fn get_last_module_span(&self, ast: &Ast<'_>) -> Option<Span> {
        let options = self.config;
        if options.flow_syntax || options.no_empty_export || !ast.is_module() {
            return None;
        }
        let body = ast.root_program().body(ast);
        for slot in body.iter().rev() {
            let statement = ast.get_node_in_sub_range(slot);
            if is_es_module_decl(ast, statement) {
                return Some(statement.span(ast));
            }
        }
        None
    }

    fn restore_esm_ctx(ast: &mut Ast<'_>, last_module: Option<Span>) {
        let program = ast.root_program();
        if let Some(span) = last_module
            && !program
                .body(ast)
                .iter()
                .any(|slot| is_es_module_decl(ast, ast.get_node_in_sub_range(slot)))
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
    }
}

struct TypeScriptReact<'a> {
    typescript: TypeScript<'a>,
}

impl TypeScriptReact<'_> {
    fn process(
        &self,
        ast: &mut Ast<'_>,
        resolver: &mut Resolver,
        header_end: u32,
    ) -> Vec<Diagnostic> {
        let options = self.typescript.config;
        let id_usage = if matches!(ast.source_type(), SourceType::Module | SourceType::CommonJs)
            && !options.verbatim_module_syntax
        {
            jsx_usage(ast, options, resolver, header_end)
        } else {
            HashSet::new()
        };
        self.typescript.process(ast, resolver, id_usage)
    }
}

fn is_es_module_decl(ast: &Ast<'_>, statement: Stmt) -> bool {
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
fn jsx_usage(
    ast: &Ast<'_>,
    options: &TypeScriptOptions,
    resolver: &Resolver,
    header_end: u32,
) -> HashSet<SymbolId> {
    let jsx = &options.jsx;
    let directives = JsxDirectives::from_header_comments(ast, header_end, None);
    let automatic = match directives.runtime {
        Some(JsxRuntime::Automatic) => true,
        Some(JsxRuntime::Classic) => false,
        Some(JsxRuntime::Preserve) | None => jsx.automatic,
    };
    if automatic {
        return HashSet::new();
    }
    let mut id_usage = HashSet::new();
    let pragma = directives
        .pragma
        .as_ref()
        .map(|(value, _)| value.as_str())
        .or(jsx.pragma.as_deref())
        .unwrap_or("React.createElement");
    let pragma_frag = directives
        .pragma_frag
        .as_ref()
        .map(|(value, _)| value.as_str())
        .or(jsx.pragma_frag.as_deref())
        .unwrap_or("React.Fragment");
    for factory in [pragma, pragma_frag] {
        id_usage.extend(id_for_jsx(ast, resolver, factory));
    }
    id_usage
}

fn id_for_jsx(ast: &Ast<'_>, resolver: &Resolver, value: &str) -> Option<SymbolId> {
    let name = value.split('.').next()?.trim();
    if name == "this" {
        return None;
    }
    resolver.lookup(ast, resolver.top_level(), name)
}
