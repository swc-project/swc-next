//! Bindings in generated statement-list scopes, shared by enums and namespaces.
use super::TypeScriptLowerer;
use rustc_hash::FxHashSet;
use swc_next_ecma_ast::{
    BindingPattern, Decl, DeclData, ImportOrExportKind, Stmt, StmtData, SubRange, SymbolId,
    VariableKind, VisitMut,
};
use swc_next_ecma_semantic::SymbolFlags;

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

    pub(super) fn register_scope_runtime_bindings(&mut self, statements: SubRange<Stmt>) {
        let ast = &*self.ast;
        let semantic = self.semantic;
        let scope = self
            .runtime_binding_scopes
            .last_mut()
            .expect("runtime declarations belong to an active statement-list scope");
        // A symbol may merge across namespace bodies, but only declarations in
        // this output scope can supply its binding. Register before lowering to
        // include later function declarations and avoid scanning redeclarations.
        for statement_id in statements.iter() {
            let statement = ast.get_node_in_sub_range(statement_id);
            let declaration = match ast.stmt_data(statement) {
                StmtData::Declaration(declaration) => Some(declaration),
                StmtData::ExportNamedDeclaration(export)
                    if export.export_kind(ast) == ImportOrExportKind::Value =>
                {
                    export.declaration(ast)
                }
                _ => None,
            };
            let Some(declaration) = declaration else {
                continue;
            };
            let binding = match ast.decl_data(declaration) {
                DeclData::Function(function) => function.id(ast),
                DeclData::Class(class) if !class.declare(ast) => class.id(ast),
                _ => None,
            };
            let Some(symbol) = binding.and_then(|binding| semantic.symbol_of(binding.node_id()))
            else {
                continue;
            };
            let flags = semantic.symbol(symbol).flags;
            if flags.intersects(SymbolFlags::VALUE_MODULE | SymbolFlags::NAMESPACE_MODULE) {
                scope.insert(symbol);
            }
        }
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
