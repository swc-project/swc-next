use swc_next_ecma_ast::*;

use super::{TraverseCtx, build};

#[derive(Default)]
pub(crate) struct VarDeclarationsStore {
    variables: Vec<Vec<VariableDeclarator>>,
}

impl VarDeclarationsStore {
    pub(crate) fn insert_var_declarator(&mut self, declaration: VariableDeclarator) {
        self.variables.last_mut().unwrap().push(declaration);
    }

    fn record_entering_stmts(&mut self) {
        self.variables.push(Vec::new());
    }

    fn get_var_statement(&mut self, ast: &mut Ast<'_>) -> Option<Stmt> {
        let variables = self.variables.pop().unwrap();
        (!variables.is_empty()).then(|| build::declaration(ast, VariableKind::Var, variables))
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transformer/src/common/var_declarations.rs#L268>.
    fn insert_into_stmts(&mut self, ast: &mut Ast<'_>, body: SubRange<Stmt>) -> SubRange<Stmt> {
        let Some(declaration) = self.get_var_statement(ast) else {
            return body;
        };
        let mut statements: Vec<_> = body
            .iter()
            .map(|slot| ast.get_node_in_sub_range(slot))
            .collect();
        let index = statements
            .iter()
            .take_while(|statement| {
                statement
                    .as_expression_statement(ast)
                    .is_some_and(|statement| statement.expression(ast).is_string_literal(ast))
            })
            .count();
        statements.insert(index, declaration);
        ast.add_sub_range(statements)
    }
}

/// SWC composes the lowering hook with VarDeclarations in a single traversal.
/// <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transformer/src/lib.rs#L90>.
pub(crate) fn hook_pass_with_var_declarations<'ast, 'r>(
    hook: impl TraverseMut<'ast, TraverseCtx<'r>>,
    ctx: &mut TraverseMutCtx<'_, 'ast, TraverseCtx<'r>>,
) {
    let program = ctx.ast.root_program();
    let _ = program.traverse_mut_with(
        &mut VarDeclarations {
            hook,
            scope_arrow_exprs: true,
        },
        ctx,
    );
}

/// Inserts queued declarations while the selected transform visits the same AST.
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transformer/src/common/var_declarations.rs#L49>.
pub(crate) struct VarDeclarations<H> {
    pub(crate) hook: H,
    // SWC's private-property-in-object pass manages statement lists only.
    pub(crate) scope_arrow_exprs: bool,
}

impl<'ast, 'r, H> TraverseMut<'ast, TraverseCtx<'r>> for VarDeclarations<H>
where
    H: TraverseMut<'ast, TraverseCtx<'r>>,
{
    fn enter_program(
        &mut self,
        node: Program,
        ctx: &mut TraverseMutCtx<'_, 'ast, TraverseCtx<'r>>,
    ) -> TraverseAction<Program> {
        let action = self.hook.enter_program(node, ctx);
        if action == TraverseAction::Continue {
            ctx.state.var_declarations.record_entering_stmts();
        }
        action
    }

    fn exit_program(
        &mut self,
        node: Program,
        ctx: &mut TraverseMutCtx<'_, 'ast, TraverseCtx<'r>>,
    ) -> TraverseAction<Program> {
        let action = self.hook.exit_program(node, ctx);
        let body = ctx
            .state
            .var_declarations
            .insert_into_stmts(ctx.ast, node.body(ctx.ast));
        node.set_body(ctx.ast, body);
        action
    }

    fn enter_arrow_function_body(
        &mut self,
        node: ArrowFunctionBody,
        ctx: &mut TraverseMutCtx<'_, 'ast, TraverseCtx<'r>>,
    ) -> TraverseAction<ArrowFunctionBody> {
        if !self.scope_arrow_exprs {
            return TraverseAction::Continue;
        }
        let Some(expression) = node.as_expr(ctx.ast) else {
            return TraverseAction::Continue;
        };
        // A replacement can bypass the union's exit hook, so close the scope
        // before returning the replacement to the arrow's body slot.
        ctx.state.var_declarations.record_entering_stmts();
        let action = expression.traverse_mut_with(self, ctx);
        let expression = match action {
            TraverseAction::ReplaceContinue(node)
            | TraverseAction::ReplaceSkip(node)
            | TraverseAction::ReplaceStop(node) => node,
            _ => expression,
        };
        let body = if let Some(declaration) = ctx.state.var_declarations.get_var_statement(ctx.ast)
        {
            let result = ctx.ast.stmt_return_statement(DUMMY_SP, Some(expression));
            let statements = ctx.ast.add_sub_range([declaration, result]);
            ArrowFunctionBody::FunctionBody(ctx.ast.function_body(
                DUMMY_SP,
                SubRange::empty(),
                statements,
            ))
        } else {
            ArrowFunctionBody::Expr(expression)
        };
        match action {
            TraverseAction::Stop | TraverseAction::ReplaceStop(_) => {
                TraverseAction::ReplaceStop(body)
            }
            _ => TraverseAction::ReplaceSkip(body),
        }
    }

    fn exit_arrow_function_expression(
        &mut self,
        node: ArrowFunctionExpression,
        ctx: &mut TraverseMutCtx<'_, 'ast, TraverseCtx<'r>>,
    ) -> TraverseAction<ArrowFunctionExpression> {
        let action = self.hook.exit_arrow_function_expression(node, ctx);
        if self.scope_arrow_exprs {
            node.set_expression(ctx.ast, node.body(ctx.ast).is_expr(ctx.ast));
        }
        action
    }

    fn enter_block_statement(
        &mut self,
        _: BlockStatement,
        ctx: &mut TraverseMutCtx<'_, 'ast, TraverseCtx<'r>>,
    ) -> TraverseAction<BlockStatement> {
        ctx.state.var_declarations.record_entering_stmts();
        TraverseAction::Continue
    }

    fn exit_block_statement(
        &mut self,
        node: BlockStatement,
        ctx: &mut TraverseMutCtx<'_, 'ast, TraverseCtx<'r>>,
    ) -> TraverseAction<BlockStatement> {
        let body = ctx
            .state
            .var_declarations
            .insert_into_stmts(ctx.ast, node.body(ctx.ast));
        node.set_body(ctx.ast, body);
        TraverseAction::Continue
    }

    fn enter_function_body(
        &mut self,
        _: FunctionBody,
        ctx: &mut TraverseMutCtx<'_, 'ast, TraverseCtx<'r>>,
    ) -> TraverseAction<FunctionBody> {
        ctx.state.var_declarations.record_entering_stmts();
        TraverseAction::Continue
    }

    fn exit_function_body(
        &mut self,
        node: FunctionBody,
        ctx: &mut TraverseMutCtx<'_, 'ast, TraverseCtx<'r>>,
    ) -> TraverseAction<FunctionBody> {
        let body = ctx
            .state
            .var_declarations
            .insert_into_stmts(ctx.ast, node.body(ctx.ast));
        node.set_body(ctx.ast, body);
        TraverseAction::Continue
    }

    fn enter_switch_case(
        &mut self,
        node: SwitchCase,
        ctx: &mut TraverseMutCtx<'_, 'ast, TraverseCtx<'r>>,
    ) -> TraverseAction<SwitchCase> {
        // The case test precedes the statement-list hook in SWC.
        let test = node.test(ctx.ast).traverse_mut_with(self, ctx);
        match test {
            TraverseAction::ReplaceContinue(test) | TraverseAction::ReplaceSkip(test) => {
                node.set_test(ctx.ast, test);
            }
            TraverseAction::ReplaceStop(test) => {
                node.set_test(ctx.ast, test);
                return TraverseAction::Stop;
            }
            TraverseAction::Stop => return TraverseAction::Stop,
            _ => {}
        }
        ctx.state.var_declarations.record_entering_stmts();
        let statements = node.consequent(ctx.ast);
        let action = statements.traverse_mut_with(self, ctx);
        let body = ctx
            .state
            .var_declarations
            .insert_into_stmts(ctx.ast, statements);
        node.set_consequent(ctx.ast, body);
        if matches!(action, TraverseAction::Stop) {
            TraverseAction::Stop
        } else {
            TraverseAction::Skip
        }
    }

    fn enter_static_block(
        &mut self,
        _: StaticBlock,
        ctx: &mut TraverseMutCtx<'_, 'ast, TraverseCtx<'r>>,
    ) -> TraverseAction<StaticBlock> {
        ctx.state.var_declarations.record_entering_stmts();
        TraverseAction::Continue
    }

    fn exit_static_block(
        &mut self,
        node: StaticBlock,
        ctx: &mut TraverseMutCtx<'_, 'ast, TraverseCtx<'r>>,
    ) -> TraverseAction<StaticBlock> {
        let body = ctx
            .state
            .var_declarations
            .insert_into_stmts(ctx.ast, node.body(ctx.ast));
        node.set_body(ctx.ast, body);
        TraverseAction::Continue
    }

    fn enter_expr(
        &mut self,
        node: Expr,
        ctx: &mut TraverseMutCtx<'_, 'ast, TraverseCtx<'r>>,
    ) -> TraverseAction<Expr> {
        self.hook.enter_expr(node, ctx)
    }

    fn exit_expr(
        &mut self,
        node: Expr,
        ctx: &mut TraverseMutCtx<'_, 'ast, TraverseCtx<'r>>,
    ) -> TraverseAction<Expr> {
        self.hook.exit_expr(node, ctx)
    }

    fn enter_class(
        &mut self,
        node: Class,
        ctx: &mut TraverseMutCtx<'_, 'ast, TraverseCtx<'r>>,
    ) -> TraverseAction<Class> {
        self.hook.enter_class(node, ctx)
    }

    fn exit_class(
        &mut self,
        node: Class,
        ctx: &mut TraverseMutCtx<'_, 'ast, TraverseCtx<'r>>,
    ) -> TraverseAction<Class> {
        self.hook.exit_class(node, ctx)
    }

    fn enter_assignment_pattern(
        &mut self,
        node: AssignmentPattern,
        ctx: &mut TraverseMutCtx<'_, 'ast, TraverseCtx<'r>>,
    ) -> TraverseAction<AssignmentPattern> {
        self.hook.enter_assignment_pattern(node, ctx)
    }

    fn enter_formal_parameter(
        &mut self,
        node: FormalParameter,
        ctx: &mut TraverseMutCtx<'_, 'ast, TraverseCtx<'r>>,
    ) -> TraverseAction<FormalParameter> {
        self.hook.enter_formal_parameter(node, ctx)
    }

    fn exit_variable_declaration(
        &mut self,
        node: VariableDeclaration,
        ctx: &mut TraverseMutCtx<'_, 'ast, TraverseCtx<'r>>,
    ) -> TraverseAction<VariableDeclaration> {
        self.hook.exit_variable_declaration(node, ctx)
    }

    fn exit_function(
        &mut self,
        node: Function,
        ctx: &mut TraverseMutCtx<'_, 'ast, TraverseCtx<'r>>,
    ) -> TraverseAction<Function> {
        self.hook.exit_function(node, ctx)
    }

    fn exit_catch_clause(
        &mut self,
        node: CatchClause,
        ctx: &mut TraverseMutCtx<'_, 'ast, TraverseCtx<'r>>,
    ) -> TraverseAction<CatchClause> {
        self.hook.exit_catch_clause(node, ctx)
    }

    fn exit_for_in_statement(
        &mut self,
        node: ForInStatement,
        ctx: &mut TraverseMutCtx<'_, 'ast, TraverseCtx<'r>>,
    ) -> TraverseAction<ForInStatement> {
        self.hook.exit_for_in_statement(node, ctx)
    }

    fn exit_for_of_statement(
        &mut self,
        node: ForOfStatement,
        ctx: &mut TraverseMutCtx<'_, 'ast, TraverseCtx<'r>>,
    ) -> TraverseAction<ForOfStatement> {
        self.hook.exit_for_of_statement(node, ctx)
    }

    fn exit_property_definition(
        &mut self,
        node: PropertyDefinition,
        ctx: &mut TraverseMutCtx<'_, 'ast, TraverseCtx<'r>>,
    ) -> TraverseAction<PropertyDefinition> {
        self.hook.exit_property_definition(node, ctx)
    }
}
