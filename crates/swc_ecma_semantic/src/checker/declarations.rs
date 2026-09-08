use super::*;

pub(super) fn check_prepared_redeclaration(
    checker: &mut Checker<'_, '_>,
    prepared: PreparedDeclaration,
) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    let name = context.ast.get_wtf8(prepared.name()).as_bytes();
    checks.check_redeclaration(prepared.node(), name, prepared, context);
}

impl<'traverse, 'a> CheckerState<'traverse, 'a> {
    #[inline(always)]
    pub(super) fn check_redeclaration(
        &mut self,
        node: NodeId,
        name: &[u8],
        prepared: PreparedDeclaration,
        context: &mut SemanticVisitCtx<'_, 'a>,
    ) {
        let target = prepared.target();
        let flags = prepared.flags();
        let excludes = prepared.excludes();
        debug_assert!(!name.is_empty() || flags.contains(SymbolFlags::ENUM_MEMBER));
        for candidate in prepared.conflict_candidates() {
            let existing_id = candidate.symbol;
            let (existing_flags, existing_node) =
                context.state.symbols.conflict_candidate_info(candidate);
            if context.ast.is_ts()
                && (flags.contains(SymbolFlags::EXPORTED)
                    || existing_flags.contains(SymbolFlags::EXPORTED))
                && context
                    .state
                    .symbols
                    .has_same_scope_export_visibility_conflict(prepared, existing_id)
            {
                self.report_redeclaration(context.ast, name, node, existing_node, existing_flags);
                return;
            }
            let import_value_conflict = !context.ast.is_ts()
                && context.ast.is_module()
                && target == ScopeId::MODULE
                && ((flags.contains(SymbolFlags::IMPORT) && existing_flags.in_value_space())
                    || (flags.in_value_space() && existing_flags.contains(SymbolFlags::IMPORT)));
            let declaration_conflict = existing_flags.intersects(excludes) || import_value_conflict;
            let invalid_class_function_merge = !declaration_conflict
                && context.ast.is_ts()
                && flags.intersects(SymbolFlags::FUNCTION | SymbolFlags::CLASS)
                && existing_flags.intersects(SymbolFlags::FUNCTION | SymbolFlags::CLASS)
                && context
                    .state
                    .symbols
                    .has_non_ambient_class_declaration(candidate, flags);
            if (declaration_conflict || invalid_class_function_merge)
                && !prepared.allows_annex_b_function_duplicate(existing_flags)
            {
                self.report_redeclaration(context.ast, name, node, existing_node, existing_flags);
                return;
            }

            // https://tc39.es/ecma262/#sec-parameter-lists-static-semantics-early-errors
            // Duplicate parameters are allowed in sloppy simple-parameter-list
            // functions. Re-add the conflict for strict, unique, arrow, and
            // non-simple cases.
            if existing_flags.contains(SymbolFlags::PARAMETER)
                && flags.contains(SymbolFlags::PARAMETER)
                && let Some(parameters) = find_formal_parameters(context)
            {
                let kind = parameters.kind(context.ast);
                let must_be_unique = context.state.scope.is_strict()
                    || matches!(
                        kind,
                        swc_next_ecma_ast::FormalParameterKind::UniqueFormalParameters
                            | swc_next_ecma_ast::FormalParameterKind::ArrowFormalParameters
                    )
                    || ecmascript::find_non_simple_parameter(context.ast, parameters).is_some();
                if must_be_unique {
                    self.report_redeclaration(
                        context.ast,
                        name,
                        node,
                        existing_node,
                        existing_flags,
                    );
                }
            }
        }

        let flags = prepared.flags();
        if !flags.intersects(SymbolFlags::FUNCTION_SCOPED_VAR | SymbolFlags::FUNCTION) {
            self.check_outer_parameter_conflict(node, name, prepared, context);
        }
        if flags.is_hoisting_var() {
            self.check_hoisting_var_conflict(node, name, prepared, context);
        }
    }

    #[inline(always)]
    fn check_outer_parameter_conflict(
        &mut self,
        node: NodeId,
        name: &[u8],
        prepared: PreparedDeclaration,
        context: &mut SemanticVisitCtx<'_, 'a>,
    ) {
        let target = prepared.target();
        let flags = prepared.flags();
        let excludes = prepared.excludes();
        // A split body environment keeps the early errors: lexical body names
        // may not duplicate parameter names (15.2.1) or catch parameter names
        // (14.15.1) one scope up:
        //
        //   function f(a = 1) { let a }   // still an error
        //   function f(a = 1) { var a }   // legal, var stays out
        if !flags.intersects(SymbolFlags::FUNCTION_SCOPED_VAR | SymbolFlags::FUNCTION) {
            let target_scope = context.state.scope.get(target);
            let outer = target_scope.parent;
            let outer_holds_parameters = outer.is_some_and(|outer| match target_scope.kind {
                ScopeKind::FunctionBody => true,
                ScopeKind::Block => {
                    match context.ast.kind_data(context.state.scope.get(outer).node) {
                        NodeKindData::CatchClause(catch) => {
                            catch.body(context.ast).node_id() == target_scope.node
                        }
                        _ => false,
                    }
                }
                _ => false,
            });
            if outer_holds_parameters
                && let Some(outer) = outer
                && let Some(symbol_id) = context.state.symbols.own_binding(outer, name)
            {
                let existing = context.state.symbols.symbol(symbol_id);
                if existing
                    .flags
                    .intersects(SymbolFlags::PARAMETER | SymbolFlags::CATCH_VAR)
                    && existing.flags.intersects(excludes)
                {
                    self.report_redeclaration(
                        context.ast,
                        name,
                        node,
                        context.state.symbols.first_decl_of(symbol_id),
                        existing.flags,
                    );
                }
            }
        }
    }

    #[inline(always)]
    fn check_hoisting_var_conflict(
        &mut self,
        node: NodeId,
        name: &[u8],
        prepared: PreparedDeclaration,
        context: &mut SemanticVisitCtx<'_, 'a>,
    ) {
        let target = prepared.target();
        let flags = prepared.flags();
        // Per 14.2.1, a hoisting var conflicts with block-scoped names
        // in any intermediate block it passes through.
        if flags.is_hoisting_var() {
            for scope_id in context
                .state
                .scope
                .ancestors(Some(context.state.scope.current))
            {
                if scope_id == target {
                    break;
                }
                let Some(symbol_id) = context.state.symbols.own_binding(scope_id, name) else {
                    continue;
                };
                let existing = context.state.symbols.symbol(symbol_id);
                if existing.flags.is_block_scoped_like() {
                    self.report_redeclaration(
                        context.ast,
                        name,
                        node,
                        context.state.symbols.first_decl_of(symbol_id),
                        existing.flags,
                    );
                    break;
                }
                if existing.flags.contains(SymbolFlags::CATCH_VAR) {
                    // https://tc39.es/ecma262/#sec-variablestatements-in-catch-blocks
                    // (14.15.1): a var colliding with a catch parameter is a
                    // syntax error, except for the Annex B.3.4 simple-binding
                    // web-compat carve-out.
                    let catch_node = context.state.scope.get(scope_id).node;
                    let NodeKindData::CatchClause(catch) = context.ast.kind_data(catch_node) else {
                        unreachable!()
                    };
                    let simple_parameter = catch.param(context.ast).is_some_and(|parameter| {
                        context.ast.node_kind(parameter.node_id()) == NodeKind::BindingIdentifier
                    });
                    if !simple_parameter {
                        self.report_redeclaration(
                            context.ast,
                            name,
                            node,
                            context.state.symbols.first_decl_of(symbol_id),
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
    context: &SemanticVisitCtx<'_, '_>,
) -> Option<swc_next_ecma_ast::FormalParameters> {
    for ancestor in context.ancestors() {
        match context.ast.kind_data(ancestor) {
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
