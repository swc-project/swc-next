use super::{TypeScriptOptions, analysis::TypeScriptAnalysis};
use crate::{context::CommentState, uid::UidGenerator};
use rustc_hash::{FxHashMap, FxHashSet};
use swc_next_ecma_ast::{Ast, Program, SourceType, SymbolId, VisitMut};
use swc_next_ecma_semantic::Semantic;

mod bindings;
mod enums;
mod erase;
mod lexical_env;
mod modules;
mod names;
mod namespaces;
mod parameters;
mod runtime;
mod statements;
mod visit;

use namespaces::NamespaceState;
use statements::StatementExpansion;

pub(super) struct TypeScriptLowerer<'visit, 'ast> {
    ast: &'visit mut Ast<'ast>,
    options: &'visit TypeScriptOptions,
    analysis: TypeScriptAnalysis<'ast>,
    semantic: &'visit Semantic<'ast>,
    semantic_node_count: usize,
    comments: CommentState,
    runtime_binding_scopes: Vec<FxHashSet<SymbolId>>,
    enum_member_symbols: FxHashMap<SymbolId, FxHashSet<SymbolId>>,
    namespaces: NamespaceState,
    // Lazy index; None as a map value means unresolved or conflicting owners.
    source_name_owners: Option<FxHashMap<&'ast [u8], Option<SymbolId>>>,
    uid_generator: UidGenerator<'ast>,
}

impl<'visit, 'ast> TypeScriptLowerer<'visit, 'ast> {
    pub(super) fn new(
        options: &'visit TypeScriptOptions,
        analysis: TypeScriptAnalysis<'ast>,
        ast: &'visit mut Ast<'ast>,
        semantic: &'visit Semantic<'ast>,
    ) -> Self {
        let semantic_node_count = ast.node_count();
        let mut uid_generator = UidGenerator::new(ast.allocator());
        for entry in semantic.iter_symbols() {
            uid_generator.observe(ast.get_wtf8(entry.symbol.name).as_bytes());
        }
        for entry in semantic.iter_references() {
            uid_generator.observe(ast.get_utf8(entry.reference.name).as_bytes());
        }
        if !options.jsx.automatic {
            for name in [
                options
                    .jsx
                    .pragma
                    .as_deref()
                    .unwrap_or("React.createElement"),
                options
                    .jsx
                    .pragma_frag
                    .as_deref()
                    .unwrap_or("React.Fragment"),
            ] {
                uid_generator.observe(name.split('.').next().unwrap().as_bytes());
            }
        }
        Self {
            ast,
            options,
            analysis,
            semantic,
            semantic_node_count,
            comments: CommentState::default(),
            runtime_binding_scopes: Vec::new(),
            enum_member_symbols: FxHashMap::default(),
            namespaces: NamespaceState::default(),
            source_name_owners: None,
            uid_generator,
        }
    }

    pub(super) fn lower_program(mut self, program: Program) {
        let body = program.body(self.ast());
        if let Some(body) = self.lower_statement_list(body) {
            program.set_body(self.ast(), body);
        }
        if self.ast_ref().source_type() == SourceType::Module
            && !self.has_module_indicator(program.body(self.ast_ref()))
        {
            self.append_esm_marker(program);
        }

        self.comments.flush(self.ast);
    }

    pub(super) fn ast_ref(&self) -> &Ast<'ast> {
        self.ast
    }
}
