use super::*;

use crate::binder::{DeclarationConflictCandidate, SymbolTracker};

pub(super) fn check_prepared_redeclaration(
    builder: &mut SemanticBuilder<'_, '_>,
    prepared: PreparedDeclaration,
) {
    let name = builder.ast.get_wtf8(prepared.name()).as_bytes();
    builder.check_redeclaration(prepared.node(), name, prepared);
}

#[cold]
#[inline(never)]
fn has_class_function_conflict(
    symbols: &SymbolTracker<'_>,
    candidate: DeclarationConflictCandidate,
    incoming: SymbolFlags,
    existing: SymbolFlags,
) -> bool {
    if incoming.contains(SymbolFlags::CLASS) {
        return !incoming.contains(SymbolFlags::AMBIENT)
            && existing.contains(SymbolFlags::FUNCTION);
    }
    incoming.contains(SymbolFlags::FUNCTION)
        && existing.contains(SymbolFlags::CLASS)
        && symbols.has_non_ambient_class_declaration(candidate)
}

impl<'traverse, 'a> SemanticBuilder<'traverse, 'a> {
    #[inline(always)]
    pub(super) fn check_redeclaration(
        &mut self,
        node: NodeId,
        name: &[u8],
        prepared: PreparedDeclaration,
    ) {
        let target = prepared.target();
        let flags = prepared.flags();
        let excludes = prepared.excludes();
        debug_assert!(!name.is_empty() || flags.contains(SymbolFlags::ENUM_MEMBER));
        for candidate in prepared.conflict_candidates() {
            let existing_id = candidate.symbol;
            let (existing_flags, existing_node) = self.symbols.conflict_candidate_info(candidate);
            if self.ast.is_ts()
                && (flags.contains(SymbolFlags::EXPORTED)
                    || existing_flags.contains(SymbolFlags::EXPORTED))
                && self
                    .symbols
                    .has_same_scope_export_visibility_conflict(prepared, existing_id)
            {
                self.report_redeclaration(self.ast, name, node, existing_node, existing_flags);
                return;
            }
            let import_value_conflict = !self.ast.is_ts()
                && self.ast.is_module()
                && target == ScopeId::MODULE
                && ((flags.contains(SymbolFlags::IMPORT) && existing_flags.in_value_space())
                    || (flags.in_value_space() && existing_flags.contains(SymbolFlags::IMPORT)));
            let declaration_conflict = existing_flags.intersects(excludes) || import_value_conflict;
            let invalid_class_function_merge = !declaration_conflict
                && self.ast.is_ts()
                && flags.intersects(SymbolFlags::FUNCTION | SymbolFlags::CLASS)
                && existing_flags.intersects(SymbolFlags::FUNCTION | SymbolFlags::CLASS)
                && has_class_function_conflict(&self.symbols, candidate, flags, existing_flags);
            if declaration_conflict || invalid_class_function_merge {
                self.report_redeclaration(self.ast, name, node, existing_node, existing_flags);
                return;
            }

            // https://tc39.es/ecma262/#sec-parameter-lists-static-semantics-early-errors
            // Duplicate parameters are allowed in sloppy simple-parameter-list
            // functions. Re-add the conflict for strict, unique, arrow, and
            // non-simple cases.
            if existing_flags.contains(SymbolFlags::PARAMETER)
                && flags.contains(SymbolFlags::PARAMETER)
                && let Some(parameters) = find_formal_parameters(self)
            {
                let kind = parameters.kind(self.ast);
                let must_be_unique = self.is_strict()
                    || matches!(
                        kind,
                        swc_next_ecma_ast::FormalParameterKind::UniqueFormalParameters
                            | swc_next_ecma_ast::FormalParameterKind::ArrowFormalParameters
                    )
                    || ecmascript::find_non_simple_parameter(self.ast, parameters).is_some();
                if must_be_unique {
                    self.report_redeclaration(self.ast, name, node, existing_node, existing_flags);
                }
            }
        }

        if let Some(existing) = self.symbols.hoisting_conflict(prepared, name) {
            self.report_redeclaration(self.ast, name, node, existing.node, existing.flags);
            return;
        }

        let flags = prepared.flags();
        if !flags.contains(SymbolFlags::FUNCTION_SCOPED_VAR) {
            self.check_outer_parameter_conflict(node, name, prepared);
        }
        if flags.is_hoisting_var() {
            self.check_hoisting_var_conflict(node, name, prepared);
        }
    }

    #[inline(always)]
    fn check_outer_parameter_conflict(
        &mut self,
        node: NodeId,
        name: &[u8],
        prepared: PreparedDeclaration,
    ) {
        let target = prepared.target();
        let excludes = prepared.excludes();
        // A split body environment keeps the early errors: lexical body names
        // may not duplicate parameter names (15.2.1) or catch parameter names
        // (14.15.1) one scope up:
        //
        //   function f(a = 1) { let a }   // still an error
        //   function f(a = 1) { var a }   // legal, var stays out
        let target_scope = self.scope.get(target);
        let Some(outer) = target_scope.parent else {
            return;
        };
        let outer_holds_parameters = match target_scope.kind {
            ScopeKind::FunctionBody => true,
            ScopeKind::Block => match self.ast.kind_data(self.scope.get(outer).node) {
                NodeKindData::CatchClause(catch) => {
                    catch.body(self.ast).node_id() == target_scope.node
                }
                _ => false,
            },
            _ => false,
        };
        if !outer_holds_parameters {
            return;
        }
        let Some(symbol_id) = self.symbols.own_binding(outer, name) else {
            return;
        };
        let existing = self.symbols.symbol(symbol_id);
        if existing
            .flags
            .intersects(SymbolFlags::PARAMETER | SymbolFlags::CATCH_VAR)
            && existing.flags.intersects(excludes)
        {
            self.report_redeclaration(
                self.ast,
                name,
                node,
                self.symbols.first_decl_of(symbol_id),
                existing.flags,
            );
        }
    }

    #[inline(always)]
    fn check_hoisting_var_conflict(
        &mut self,
        node: NodeId,
        name: &[u8],
        prepared: PreparedDeclaration,
    ) {
        let target = prepared.target();
        let flags = prepared.flags();
        // Per 14.2.1, a hoisting var conflicts with block-scoped names
        // in any intermediate block it passes through.
        if flags.is_hoisting_var() {
            for scope_id in self.scope.ancestors(Some(self.current_scope)) {
                if scope_id == target {
                    break;
                }
                let Some(symbol_id) = self.symbols.own_binding(scope_id, name) else {
                    continue;
                };
                let existing = self.symbols.symbol(symbol_id);
                if existing.flags.is_block_scoped_like() {
                    self.report_redeclaration(
                        self.ast,
                        name,
                        node,
                        self.symbols.first_decl_of(symbol_id),
                        existing.flags,
                    );
                    break;
                }
                if existing.flags.contains(SymbolFlags::CATCH_VAR) {
                    // https://tc39.es/ecma262/#sec-variablestatements-in-catch-blocks
                    // (14.15.1): a var colliding with a catch parameter is a
                    // syntax error, except for the Annex B.3.4 simple-binding
                    // web-compat carve-out.
                    let catch_node = self.scope.get(scope_id).node;
                    let NodeKindData::CatchClause(catch) = self.ast.kind_data(catch_node) else {
                        unreachable!()
                    };
                    let simple_parameter = catch.param(self.ast).is_some_and(|parameter| {
                        self.ast.node_kind(parameter.node_id()) == NodeKind::BindingIdentifier
                    });
                    if !simple_parameter {
                        self.report_redeclaration(
                            self.ast,
                            name,
                            node,
                            self.symbols.first_decl_of(symbol_id),
                            existing.flags,
                        );
                        break;
                    }
                }
            }
        }
    }
}

fn find_formal_parameters(
    builder: &SemanticBuilder<'_, '_>,
) -> Option<swc_next_ecma_ast::FormalParameters> {
    for ancestor in builder.ancestors() {
        match builder.ast.kind_data(ancestor) {
            NodeKindData::FormalParameters(parameters) => {
                return Some(parameters);
            }
            NodeKindData::Program(_)
            | NodeKindData::Function(_)
            | NodeKindData::ArrowFunctionExpression(_) => {
                return None;
            }
            _ => {}
        }
    }
    None
}
