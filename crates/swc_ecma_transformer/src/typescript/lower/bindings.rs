//! Bindings in generated statement-list scopes for enum emission.
use super::TypeScriptLowerer;
use rustc_hash::FxHashSet;
use swc_next_ecma_ast::{BindingPattern, Decl, Stmt, SubRange, SymbolId, VariableKind, VisitMut};

impl TypeScriptLowerer<'_, '_> {
    pub(super) fn runtime_binding_statement(
        &mut self,
        binding: swc_next_ecma_ast::BindingIdentifier,
        span: swc_next_ecma_ast::Span,
        kind: VariableKind,
        exported: bool,
    ) -> Stmt {
        let declarator = self.ast().variable_declarator(
            span,
            BindingPattern::BindingIdentifier(binding),
            None,
            None,
            false,
        );
        let declarators = self.ast().add_sub_range([declarator]);
        let declaration = self
            .ast()
            .variable_declaration(span, kind, declarators, false);
        let declaration = Decl::VariableDeclaration(declaration);
        if exported {
            self.ast().stmt_export_named_declaration(
                span,
                Some(declaration),
                SubRange::empty(),
                None,
                SubRange::empty(),
                swc_next_ecma_ast::ImportOrExportKind::Value,
            )
        } else {
            Stmt::Declaration(declaration)
        }
    }

    pub(super) fn register_runtime_binding(&mut self, owner: SymbolId) -> Option<VariableKind> {
        let lexical = self.runtime_binding_scopes.len() > 1;
        let scope = self
            .runtime_binding_scopes
            .last_mut()
            .expect("a runtime declaration is lowered from a statement list");
        if !scope.insert(owner) {
            return None;
        }
        Some(if lexical {
            VariableKind::Let
        } else {
            VariableKind::Var
        })
    }

    pub(super) fn enter_binding_scope(&mut self) {
        self.runtime_binding_scopes.push(FxHashSet::default());
    }

    pub(super) fn exit_binding_scope(&mut self) {
        self.runtime_binding_scopes
            .pop()
            .expect("a statement-list runtime binding scope exists");
    }
}
