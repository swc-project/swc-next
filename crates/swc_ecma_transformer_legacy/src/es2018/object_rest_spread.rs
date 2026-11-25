/// SWC's `object_rest_spread::Config`. All flags default to `false`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Config {
    /// Exclude symbol keys when copying object-rest properties.
    pub no_symbol: bool,
    /// Assign spread properties rather than defining them.
    pub set_property: bool,
    /// Assume property getters have no side effects.
    pub pure_getters: bool,
}

use crate::common::{TraverseCtx, build};
use std::{collections::HashMap, mem};
use swc_next_ecma_ast::*;

pub(crate) struct ObjectRestSpreadPass {
    options: Config,
    export_idents: HashMap<NodeId, Vec<BindingIdentifier>>,
}
impl ObjectRestSpreadPass {
    pub(crate) fn new(options: Config) -> Self {
        Self {
            options,
            export_idents: HashMap::new(),
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transformer/src/es2018/object_rest_spread.rs#L42>.
    fn collect_exports(
        &mut self,
        program: Program,
        ctx: &mut TraverseMutCtx<'_, '_, TraverseCtx<'_>>,
    ) {
        let ast = &*ctx.ast;
        for slot in program.body(ast).iter() {
            let statement = ast.get_node_in_sub_range(slot);
            let Some(export) = statement.as_export_named_declaration(ast) else {
                continue;
            };
            let Some(declaration) = export.declaration(ast) else {
                continue;
            };
            let Some(variable) = declaration.as_variable_declaration(ast) else {
                continue;
            };
            let declarators = variable.declarators(ast);
            let has_rest = declarators.iter().any(|slot| {
                let declarator = ast.get_node_in_sub_range(slot);
                has_object_rest_in_pat(ast, declarator.id(ast))
            });
            if !has_rest {
                continue;
            }
            let mut ids = Vec::new();
            for slot in declarators.iter() {
                let declarator = ast.get_node_in_sub_range(slot);
                build::for_each_binding_ident(ast, declarator.id(ast), |binding| ids.push(binding));
            }
            self.export_idents.insert(variable.node_id(), ids);
        }
    }
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transformer/src/es2018/object_rest_spread.rs#L427>.
    fn rewrite_exports(
        &mut self,
        program: Program,
        ctx: &mut TraverseMutCtx<'_, '_, TraverseCtx<'_>>,
    ) {
        if self.export_idents.is_empty() {
            return;
        }
        let mut body = Vec::new();
        for slot in program.body(ctx.ast).iter() {
            let statement = ctx.ast.get_node_in_sub_range(slot);
            if let Some(export) = statement.as_export_named_declaration(ctx.ast)
                && let Some(declaration) = export.declaration(ctx.ast)
                && let Some(ids) = self.export_idents.remove(&declaration.node_id())
            {
                body.push(Stmt::Declaration(declaration));
                let mut specifiers = Vec::new();
                for id in ids {
                    let local = ctx.ast.identifier_name(DUMMY_SP, id.name(ctx.ast));
                    ctx.state
                        .resolver
                        .copy_context(id.node_id(), local.node_id());
                    let name = ModuleExportName::IdentifierName(local);
                    specifiers.push(ctx.ast.export_specifier(
                        DUMMY_SP,
                        name,
                        name,
                        ImportOrExportKind::Value,
                    ));
                }
                if !specifiers.is_empty() {
                    let specifiers = ctx.ast.add_sub_range(specifiers);
                    body.push(Stmt::ExportNamedDeclaration(
                        ctx.ast.export_named_declaration(
                            export.span(ctx.ast),
                            None,
                            specifiers,
                            None,
                            SubRange::empty(),
                            ImportOrExportKind::Value,
                        ),
                    ));
                }
            } else {
                body.push(statement);
            }
        }
        let body = ctx.ast.add_sub_range(body);
        program.set_body(ctx.ast, body);
    }

    fn arrow_parameters(
        &mut self,
        arrow: ArrowFunctionExpression,
        ctx: &mut TraverseMutCtx<'_, '_, TraverseCtx<'_>>,
    ) {
        let statements = ParamCollector {
            options: self.options,
        }
        .collect(arrow.params(ctx.ast), ctx);
        if statements.is_empty() {
            return;
        }
        let body = if let Some(body) = arrow.body(ctx.ast).as_function_body(ctx.ast) {
            let statements = build::prepend_stmts(ctx.ast, statements, body.body(ctx.ast));
            body.set_body(ctx.ast, statements);
            body
        } else {
            let expression = arrow.body(ctx.ast).as_expr(ctx.ast).unwrap();
            let result = ctx.ast.stmt_return_statement(DUMMY_SP, Some(expression));
            let statements = ctx
                .ast
                .add_sub_range(statements.into_iter().chain([result]));
            ctx.ast
                .function_body(DUMMY_SP, SubRange::empty(), statements)
        };
        arrow.set_body(ctx.ast, ArrowFunctionBody::FunctionBody(body));
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transformer/src/es2018/object_rest_spread.rs#L487>.
    fn transform_for_loop(
        &self,
        left: ForStatementLeft,
        body: Stmt,
        ctx: &mut TraverseMutCtx<'_, '_, TraverseCtx<'_>>,
    ) -> Option<(ForStatementLeft, Stmt)> {
        let (pattern, declaration) = match ctx.ast.for_statement_left_data(left) {
            ForStatementLeftData::VariableDeclaration(variable) => {
                let declaration = variable.declarators(ctx.ast).get_node(ctx.ast, 0).unwrap();
                (declaration.id(ctx.ast), Some((variable, declaration)))
            }
            ForStatementLeftData::AssignmentTarget(target) => {
                (build::target_pattern(ctx.ast, target), None)
            }
        };
        if !has_object_rest_in_pat(ctx.ast, pattern) {
            return None;
        }
        let binding = if declaration.is_some() {
            ctx.state.resolver.private_binding(ctx.ast, "_ref")
        } else {
            ctx.state.insert_var(ctx.ast, "_ref")
        };
        let (left, statement) = if let Some((variable, declaration)) = declaration {
            declaration.set_id(ctx.ast, BindingPattern::BindingIdentifier(binding));
            let reference = ctx.state.resolver.reference(ctx.ast, binding);
            let mut lowerer = RestLowerer {
                ast: ctx.ast,
                context: &mut ctx.state,
                options: self.options,
                out: RestOutput::Declarations(Vec::new()),
            };
            lowerer.visit(pattern, reference);
            let RestOutput::Declarations(declarations) = lowerer.out else {
                unreachable!()
            };
            (
                left,
                build::declaration(ctx.ast, variable.kind(ctx.ast), declarations),
            )
        } else {
            let reference = ctx.state.resolver.reference(ctx.ast, binding);
            let left = ForStatementLeft::AssignmentTarget(
                AssignmentTarget::SimpleAssignmentTarget(build::target(ctx.ast, reference)),
            );
            let reference = ctx.state.resolver.reference(ctx.ast, binding);
            let mut lowerer = RestLowerer {
                ast: ctx.ast,
                context: &mut ctx.state,
                options: self.options,
                out: RestOutput::Expressions(Vec::new()),
            };
            lowerer.visit(pattern, reference);
            let result = lowerer.reference(binding);
            let RestOutput::Expressions(mut expressions) = lowerer.out else {
                unreachable!()
            };
            expressions.push(result);
            let expression = build::sequence(ctx.ast, expressions);
            (
                left,
                ctx.ast.stmt_expression_statement(DUMMY_SP, expression),
            )
        };
        let body = if let Some(block) = body.as_block_statement(ctx.ast) {
            let statements = build::prepend_stmts(ctx.ast, [statement], block.body(ctx.ast));
            block.set_body(ctx.ast, statements);
            body
        } else {
            let statements = ctx.ast.add_sub_range([statement, body]);
            Stmt::BlockStatement(ctx.ast.block_statement(DUMMY_SP, statements))
        };
        Some((left, body))
    }
}

impl<'ast> TraverseMut<'ast, TraverseCtx<'_>> for ObjectRestSpreadPass {
    fn enter_program(
        &mut self,
        node: Program,
        ctx: &mut TraverseMutCtx<'_, 'ast, TraverseCtx<'_>>,
    ) -> TraverseAction<Program> {
        self.collect_exports(node, ctx);
        TraverseAction::Continue
    }

    fn exit_program(
        &mut self,
        node: Program,
        ctx: &mut TraverseMutCtx<'_, 'ast, TraverseCtx<'_>>,
    ) -> TraverseAction<Program> {
        self.rewrite_exports(node, ctx);
        TraverseAction::Continue
    }

    fn exit_arrow_function_expression(
        &mut self,
        node: ArrowFunctionExpression,
        ctx: &mut TraverseMutCtx<'_, 'ast, TraverseCtx<'_>>,
    ) -> TraverseAction<ArrowFunctionExpression> {
        self.arrow_parameters(node, ctx);
        TraverseAction::Continue
    }
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transformer/src/es2018/object_rest_spread.rs#L64>.
    fn exit_expr(
        &mut self,
        expression: Expr,
        ctx: &mut TraverseMutCtx<'_, 'ast, TraverseCtx<'_>>,
    ) -> TraverseAction<Expr> {
        if let Some(object) = expression.as_object_expression(ctx.ast) {
            if !object.properties(ctx.ast).iter().any(|slot| {
                ctx.ast
                    .get_node_in_sub_range(slot)
                    .is_spread_element(ctx.ast)
            }) {
                return TraverseAction::Continue;
            }
            let mut helper_name = if self.options.set_property {
                "extends"
            } else {
                "object_spread"
            };
            let _ = ctx.state.helper(ctx.ast, helper_name);
            let mut arguments = Vec::new();
            let mut properties = Vec::new();
            let mut first = true;
            for slot in object.properties(ctx.ast).iter() {
                let property = ctx.ast.get_node_in_sub_range(slot);
                if let Some(spread) = property.as_spread_element(ctx.ast) {
                    if first || !properties.is_empty() {
                        let props = ctx.ast.add_sub_range(mem::take(&mut properties));
                        arguments.push(ctx.ast.expr_object_expression(DUMMY_SP, props));
                        if !first && !self.options.pure_getters {
                            let helper = ctx.state.helper(ctx.ast, "object_spread_props");
                            arguments = vec![build::call(ctx.ast, helper, arguments, false)];
                        }
                        first = false;
                    }
                    arguments.push(spread.argument(ctx.ast));
                } else {
                    if !first && properties.is_empty() && !self.options.pure_getters {
                        let helper = ctx.state.helper(ctx.ast, helper_name);
                        arguments = vec![build::call(ctx.ast, helper, arguments, false)];
                    }
                    properties.push(property);
                }
            }
            if !properties.is_empty() {
                if !self.options.pure_getters {
                    helper_name = "object_spread_props";
                }
                let properties = ctx.ast.add_sub_range(properties);
                arguments.push(ctx.ast.expr_object_expression(DUMMY_SP, properties));
            }
            let helper = ctx.state.helper(ctx.ast, helper_name);
            let call = build::call(ctx.ast, helper, arguments, false);
            call.set_span(ctx.ast, object.span(ctx.ast));
            return TraverseAction::ReplaceSkip(call);
        }
        if let Some(assignment) = expression.as_assignment_expression(ctx.ast)
            && assignment.operator(ctx.ast) == AssignmentOperator::Assign
        {
            let pattern = build::target_pattern(ctx.ast, assignment.left(ctx.ast));
            if has_object_rest_in_pat(ctx.ast, pattern) {
                let mut lowerer = RestLowerer {
                    ast: ctx.ast,
                    context: &mut ctx.state,
                    options: self.options,
                    out: RestOutput::Expressions(Vec::new()),
                };
                let binding = lowerer.declare_temp("_ref");
                lowerer.assign(
                    BindingPattern::BindingIdentifier(binding),
                    assignment.right(lowerer.ast),
                );
                let reference = lowerer.reference(binding);
                lowerer.visit(pattern, reference);
                let result = lowerer.reference(binding);
                let RestOutput::Expressions(mut expressions) = lowerer.out else {
                    unreachable!()
                };
                expressions.push(result);
                let expression = build::sequence(ctx.ast, expressions);
                expression.set_span(ctx.ast, assignment.span(ctx.ast));
                return TraverseAction::ReplaceSkip(expression);
            }
        }
        TraverseAction::Continue
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transformer/src/es2018/object_rest_spread.rs#L186>.
    fn exit_variable_declaration(
        &mut self,
        variable: VariableDeclaration,
        ctx: &mut TraverseMutCtx<'_, 'ast, TraverseCtx<'_>>,
    ) -> TraverseAction<VariableDeclaration> {
        if !variable.declarators(ctx.ast).iter().any(|slot| {
            has_object_rest_in_pat(ctx.ast, ctx.ast.get_node_in_sub_range(slot).id(ctx.ast))
        }) {
            return TraverseAction::Continue;
        }
        let mut declarations = Vec::new();
        for slot in variable.declarators(ctx.ast).iter() {
            let declaration = ctx.ast.get_node_in_sub_range(slot);
            let Some(mut init) = declaration.init(ctx.ast) else {
                declarations.push(declaration);
                continue;
            };
            if !has_object_rest_in_pat(ctx.ast, declaration.id(ctx.ast)) {
                declarations.push(declaration);
                continue;
            }
            let mut lowerer = RestLowerer {
                ast: ctx.ast,
                context: &mut ctx.state,
                options: self.options,
                out: RestOutput::Declarations(Vec::new()),
            };
            if !init.is_identifier_reference(lowerer.ast) {
                let name = build::alias_ident_for(lowerer.ast, init, "_ref");
                let binding = lowerer.declare_temp(&name);
                lowerer.assign(BindingPattern::BindingIdentifier(binding), init);
                init = lowerer.reference(binding);
            }
            lowerer.visit(declaration.id(lowerer.ast), init);
            let RestOutput::Declarations(output) = lowerer.out else {
                unreachable!()
            };
            declarations.extend(output);
        }
        let declarations = ctx.ast.add_sub_range(declarations);
        variable.set_declarators(ctx.ast, declarations);
        TraverseAction::Continue
    }
    fn exit_function(
        &mut self,
        function: Function,
        ctx: &mut TraverseMutCtx<'_, 'ast, TraverseCtx<'_>>,
    ) -> TraverseAction<Function> {
        let statements = ParamCollector {
            options: self.options,
        }
        .collect(function.params(ctx.ast), ctx);
        if !statements.is_empty() {
            let body = function.body(ctx.ast);
            let statements = build::prepend_stmts(ctx.ast, statements, body.body(ctx.ast));
            body.set_body(ctx.ast, statements);
        }
        TraverseAction::Continue
    }
    fn exit_catch_clause(
        &mut self,
        clause: CatchClause,
        ctx: &mut TraverseMutCtx<'_, 'ast, TraverseCtx<'_>>,
    ) -> TraverseAction<CatchClause> {
        if let Some(pattern) = clause.param(ctx.ast)
            && has_object_rest_in_pat(ctx.ast, pattern)
        {
            let binding = ctx.state.resolver.private_binding(ctx.ast, "_param");
            clause.set_param(ctx.ast, Some(BindingPattern::BindingIdentifier(binding)));
            let reference = ctx.state.resolver.reference(ctx.ast, binding);
            let mut lowerer = RestLowerer {
                ast: ctx.ast,
                context: &mut ctx.state,
                options: self.options,
                out: RestOutput::Declarations(Vec::new()),
            };
            lowerer.visit(pattern, reference);
            let RestOutput::Declarations(declarations) = lowerer.out else {
                unreachable!()
            };
            let statement = build::declaration(ctx.ast, VariableKind::Let, declarations);
            let block = clause.body(ctx.ast);
            let statements = build::prepend_stmts(ctx.ast, [statement], block.body(ctx.ast));
            block.set_body(ctx.ast, statements);
        }
        TraverseAction::Continue
    }
    fn exit_for_in_statement(
        &mut self,
        statement: ForInStatement,
        ctx: &mut TraverseMutCtx<'_, 'ast, TraverseCtx<'_>>,
    ) -> TraverseAction<ForInStatement> {
        if let Some((left, body)) =
            self.transform_for_loop(statement.left(ctx.ast), statement.body(ctx.ast), ctx)
        {
            statement.set_left(ctx.ast, left);
            statement.set_body(ctx.ast, body);
        }
        TraverseAction::Continue
    }
    fn exit_for_of_statement(
        &mut self,
        statement: ForOfStatement,
        ctx: &mut TraverseMutCtx<'_, 'ast, TraverseCtx<'_>>,
    ) -> TraverseAction<ForOfStatement> {
        if let Some((left, body)) =
            self.transform_for_loop(statement.left(ctx.ast), statement.body(ctx.ast), ctx)
        {
            statement.set_left(ctx.ast, left);
            statement.set_body(ctx.ast, body);
        }
        TraverseAction::Continue
    }
}

enum RestOutput {
    Declarations(Vec<VariableDeclarator>),
    Expressions(Vec<Expr>),
}
struct RestLowerer<'a, 'state, 'ast> {
    ast: &'a mut Ast<'ast>,
    context: &'a mut TraverseCtx<'state>,
    options: Config,
    out: RestOutput,
}

impl RestLowerer<'_, '_, '_> {
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transformer/src/es2018/object_rest_spread.rs#L917>.
    fn declare_temp(&mut self, name: &str) -> BindingIdentifier {
        if matches!(self.out, RestOutput::Expressions(_)) {
            self.context.insert_var(self.ast, name)
        } else {
            self.context.resolver.private_binding(self.ast, name)
        }
    }
    fn reference(&mut self, binding: BindingIdentifier) -> Expr {
        self.context.resolver.reference(self.ast, binding)
    }
    fn clone_expr(&mut self, expression: Expr) -> Expr {
        build::clone_expr(self.ast, self.context.resolver, expression)
    }

    fn assign(&mut self, pattern: BindingPattern, init: Expr) {
        match self.out {
            RestOutput::Declarations(ref mut declarations) => declarations.push(
                self.ast
                    .variable_declarator(DUMMY_SP, pattern, None, Some(init), false),
            ),
            RestOutput::Expressions(_) => {
                let target = build::assignment_target(self.ast, self.context.resolver, pattern);
                let assignment = self.ast.expr_assignment_expression(
                    DUMMY_SP,
                    target,
                    init,
                    AssignmentOperator::Assign,
                );
                if let RestOutput::Expressions(expressions) = &mut self.out {
                    expressions.push(assignment);
                }
            }
        }
    }
    fn capture_init(&mut self, init: Expr) -> (Expr, Expr) {
        if init.is_identifier_reference(self.ast) {
            return (self.clone_expr(init), init);
        }
        let binding = self.declare_temp("_ref");
        self.assign(BindingPattern::BindingIdentifier(binding), init);
        (self.reference(binding), self.reference(binding))
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transformer/src/es2018/object_rest_spread.rs#L674>.
    fn visit(&mut self, pattern: BindingPattern, init: Expr) {
        match self.ast.binding_pattern_data(pattern) {
            BindingPatternData::ObjectPattern(object) => {
                self.visit_object(object, init, Vec::new())
            }
            BindingPatternData::ArrayPattern(array) => self.split_array_pattern(array, init),
            _ => self.assign(pattern, init),
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transformer/src/es2018/object_rest_spread.rs#L678>.
    fn visit_object(
        &mut self,
        object: ObjectPattern,
        init: Expr,
        mut captured: Vec<(PropertyKey, bool)>,
    ) {
        let properties: Vec<_> = object
            .properties(self.ast)
            .iter()
            .map(|slot| self.ast.get_node_in_sub_range(slot))
            .collect();
        let rest = object.rest(self.ast);
        for &property in &properties {
            if property.computed(self.ast)
                && let Some(key) = property.key(self.ast).as_expr(self.ast)
                && !is_pure_expr(self.ast, key)
            {
                let binding = self.declare_temp("_key");
                self.assign(BindingPattern::BindingIdentifier(binding), key);
                let reference = self.reference(binding);
                property.set_key(self.ast, PropertyKey::Expr(reference));
            }
        }
        for (index, &property) in properties.iter().enumerate() {
            if rest.is_some() {
                captured.push((property.key(self.ast), property.computed(self.ast)));
            }
            if has_object_rest_in_pat(self.ast, property.value(self.ast)) {
                let binding = self.declare_temp("_ref");
                let value = property.value(self.ast);
                let nested = if let Some(assignment) = value.as_assignment_pattern(self.ast) {
                    let nested = assignment.left(self.ast);
                    assignment.set_left(self.ast, BindingPattern::BindingIdentifier(binding));
                    nested
                } else {
                    property.set_value(self.ast, BindingPattern::BindingIdentifier(binding));
                    value
                };
                let remaining = properties[index + 1..].to_vec();
                let (source, after) = if !remaining.is_empty() || rest.is_some() {
                    let (source, init) = self.capture_init(init);
                    (init, Some(source))
                } else {
                    (init, None)
                };
                let prefix = self.ast.add_sub_range(properties[..=index].iter().copied());
                object.set_properties(self.ast, prefix);
                object.set_rest(self.ast, None);
                self.assign(BindingPattern::ObjectPattern(object), source);
                let reference = self.reference(binding);
                self.visit(nested, reference);
                if let Some(after) = after {
                    let remaining = self.ast.add_sub_range(remaining);
                    let object = self
                        .ast
                        .object_pattern(object.span(self.ast), remaining, rest);
                    self.visit_object(object, after, captured);
                }
                return;
            }
        }
        if let Some(rest) = rest {
            let (source, init) = self.capture_init(init);
            object.set_rest(self.ast, None);
            if !properties.is_empty() || captured.is_empty() {
                self.assign(BindingPattern::ObjectPattern(object), init);
            }
            let call = self.make_rest_call(source, captured);
            self.assign(rest.argument(self.ast), call);
        } else {
            self.assign(BindingPattern::ObjectPattern(object), init);
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transformer/src/es2018/object_rest_spread.rs#L818>.
    fn split_array_pattern(&mut self, array: ArrayPattern, init: Expr) {
        let mut elements: Vec<_> = array
            .elements(self.ast)
            .iter()
            .map(|slot| self.ast.get_node_in_sub_range(slot))
            .collect();
        if let Some(rest) = array.rest(self.ast) {
            elements.push(Some(BindingPattern::BindingRestElement(rest)));
        }
        let Some(index) = elements.iter().position(|element| {
            element.is_some_and(|pattern| has_object_rest_in_pat(self.ast, pattern))
        }) else {
            self.assign(BindingPattern::ArrayPattern(array), init);
            return;
        };
        let binding = self.declare_temp("_ref");
        let pattern = elements[index].unwrap();
        let nested = match self.ast.binding_pattern_data(pattern) {
            BindingPatternData::BindingRestElement(rest) => {
                let nested = rest.argument(self.ast);
                rest.set_argument(self.ast, BindingPattern::BindingIdentifier(binding));
                nested
            }
            BindingPatternData::AssignmentPattern(assignment) => {
                let nested = assignment.left(self.ast);
                assignment.set_left(self.ast, BindingPattern::BindingIdentifier(binding));
                nested
            }
            _ => {
                elements[index] = Some(BindingPattern::BindingIdentifier(binding));
                pattern
            }
        };
        let remaining = elements.split_off(index + 1);
        let tail = if remaining.is_empty() {
            None
        } else {
            let tail = self.declare_temp("_rest");
            let rest = self
                .ast
                .binding_rest_element(DUMMY_SP, BindingPattern::BindingIdentifier(tail));
            elements.push(Some(BindingPattern::BindingRestElement(rest)));
            Some(tail)
        };
        let prefix = build::array_pattern(self.ast, elements);
        self.assign(prefix, init);
        let reference = self.reference(binding);
        self.visit(nested, reference);
        if let Some(tail) = tail {
            let remaining = build::array_pattern(self.ast, remaining)
                .as_array_pattern(self.ast)
                .unwrap();
            let reference = self.reference(tail);
            self.split_array_pattern(remaining, reference);
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transformer/src/es2018/object_rest_spread.rs#L1016>.
    fn make_rest_call(&mut self, source: Expr, excluded: Vec<(PropertyKey, bool)>) -> Expr {
        if excluded.is_empty() {
            let helper = self.context.helper(self.ast, "extends");
            let empty = self.ast.expr_object_expression(DUMMY_SP, SubRange::empty());
            return build::call(self.ast, helper, [empty, source], false);
        }
        let helper = self.context.helper(
            self.ast,
            if self.options.no_symbol {
                "object_without_properties_loose"
            } else {
                "object_without_properties"
            },
        );
        let impure = excluded
            .iter()
            .filter(|(key, computed)| {
                *computed && !is_lit_str(self.ast, key.as_expr(self.ast).unwrap())
            })
            .count();
        let mut keys = Vec::new();
        for (key, computed) in excluded {
            let value = if computed {
                let expression = key.as_expr(self.ast).unwrap();
                if impure == 1 && !is_lit_str(self.ast, expression) {
                    let helper = self.context.helper(self.ast, "to_property_key");
                    build::call(self.ast, helper, [expression], false)
                } else {
                    self.clone_expr(expression)
                }
            } else if let Some(id) = key.as_identifier_name(self.ast) {
                let value = self.ast.add_utf8(self.ast.get_utf8(id.name(self.ast)));
                self.ast
                    .expr_string_literal(DUMMY_SP, value.into(), AstUtf8::EMPTY)
            } else {
                let expression = key.as_expr(self.ast).unwrap();
                match self.ast.expr_data(expression) {
                    ExprData::StringLiteral(_) => self.clone_expr(expression),
                    ExprData::NumericLiteral(number) => {
                        let value = self
                            .ast
                            .add_utf8(&build::number_string(number.value(self.ast)));
                        self.ast.expr_string_literal(
                            number.span(self.ast),
                            value.into(),
                            AstUtf8::EMPTY,
                        )
                    }
                    ExprData::BigIntLiteral(bigint) => {
                        let value = bigint_decimal(self.ast.get_utf8(bigint.raw(self.ast)));
                        let value = self.ast.add_utf8(&value);
                        self.ast.expr_string_literal(
                            bigint.span(self.ast),
                            value.into(),
                            AstUtf8::EMPTY,
                        )
                    }
                    _ => unreachable!("a noncomputed destructuring key is a literal"),
                }
            };
            keys.push(Some(Argument::Expr(value)));
        }
        let keys = self.ast.add_sub_range(keys);
        let mut keys = self.ast.expr_array_expression(DUMMY_SP, keys);
        if impure > 1 {
            let callee = build::member(self.ast, keys, "map");
            let helper = self.context.helper(self.ast, "to_property_key");
            keys = build::call(self.ast, callee, [helper], false);
        }
        build::call(self.ast, helper, [source, keys], false)
    }
}

fn has_object_rest_in_pat(ast: &Ast<'_>, pattern: BindingPattern) -> bool {
    match ast.binding_pattern_data(pattern) {
        BindingPatternData::ObjectPattern(object) => {
            object.rest(ast).is_some()
                || object.properties(ast).iter().any(|slot| {
                    has_object_rest_in_pat(ast, ast.get_node_in_sub_range(slot).value(ast))
                })
        }
        BindingPatternData::ArrayPattern(array) => {
            array.elements(ast).iter().any(|slot| {
                ast.get_node_in_sub_range(slot)
                    .is_some_and(|pattern| has_object_rest_in_pat(ast, pattern))
            }) || array
                .rest(ast)
                .is_some_and(|rest| has_object_rest_in_pat(ast, rest.argument(ast)))
        }
        BindingPatternData::AssignmentPattern(assignment) => {
            has_object_rest_in_pat(ast, assignment.left(ast))
        }
        BindingPatternData::BindingRestElement(rest) => {
            has_object_rest_in_pat(ast, rest.argument(ast))
        }
        _ => false,
    }
}
fn is_lit_str(ast: &Ast<'_>, expression: Expr) -> bool {
    expression.is_string_literal(ast)
        || expression
            .as_template_literal(ast)
            .is_some_and(|template| template.expressions(ast).is_empty())
}
fn is_pure_expr(ast: &Ast<'_>, expression: Expr) -> bool {
    matches!(
        ast.expr_data(expression),
        ExprData::IdentifierReference(_)
            | ExprData::NumericLiteral(_)
            | ExprData::StringLiteral(_)
            | ExprData::BooleanLiteral(_)
            | ExprData::NullLiteral(_)
            | ExprData::BigIntLiteral(_)
            | ExprData::RegExpLiteral(_)
    ) || expression
        .as_template_literal(ast)
        .is_some_and(|template| template.expressions(ast).is_empty())
        || expression.as_member_expression(ast).is_some_and(|member| {
            !member.computed(ast)
                && member
                    .object(ast)
                    .as_identifier_reference(ast)
                    .is_some_and(|id| ast.get_utf8(id.name(ast)) == "Symbol")
        })
}
fn bigint_decimal(raw: &str) -> String {
    let raw = raw.trim_end_matches('n').replace('_', "");
    let (base, digits) = if raw.starts_with("0x") || raw.starts_with("0X") {
        (16, &raw[2..])
    } else if raw.starts_with("0o") || raw.starts_with("0O") {
        (8, &raw[2..])
    } else if raw.starts_with("0b") || raw.starts_with("0B") {
        (2, &raw[2..])
    } else {
        (10, raw.as_str())
    };
    let mut result = vec![0u32];
    for digit in digits.chars() {
        let mut carry = digit.to_digit(base).unwrap();
        for value in &mut result {
            let n = *value * base + carry;
            *value = n % 10;
            carry = n / 10;
        }
        while carry > 0 {
            result.push(carry % 10);
            carry /= 10;
        }
    }
    while result.len() > 1 && result.last() == Some(&0) {
        result.pop();
    }
    result
        .into_iter()
        .rev()
        .map(|digit| char::from_digit(digit, 10).unwrap())
        .collect()
}

struct ParamCollector {
    options: Config,
}

impl ParamCollector {
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transformer/src/es2018/object_rest_spread.rs#L1121>.
    fn collect(
        &self,
        parameters: FormalParameters,
        ctx: &mut TraverseMutCtx<'_, '_, TraverseCtx<'_>>,
    ) -> Vec<Stmt> {
        let items: Vec<_> = parameters
            .items(ctx.ast)
            .iter()
            .map(|slot| {
                ctx.ast
                    .get_node_in_sub_range(slot)
                    .as_formal_parameter(ctx.ast)
                    .unwrap()
            })
            .collect();
        let rest = parameters.rest(ctx.ast);
        if !items
            .iter()
            .any(|parameter| has_object_rest_in_pat(ctx.ast, parameter.pattern(ctx.ast).into()))
            && !rest
                .is_some_and(|rest| has_object_rest_in_pat(ctx.ast, rest.argument(ctx.ast).into()))
        {
            return Vec::new();
        }
        let mut patterns = Vec::new();
        let mut values = Vec::new();
        let mut started = false;
        for (index, &parameter) in items.iter().enumerate() {
            let pattern: BindingPattern = parameter.pattern(ctx.ast).into();
            if !started
                && pattern.is_binding_identifier(ctx.ast)
                && parameter.initializer(ctx.ast).is_none()
            {
                continue;
            }
            started = true;
            let binding = ctx
                .state
                .resolver
                .private_binding(ctx.ast, &format!("_{index}"));
            parameter.set_pattern(ctx.ast, FormalParameterPattern::BindingIdentifier(binding));
            let pattern = if let Some(initializer) = parameter.initializer(ctx.ast) {
                let undefined = build::undefined(ctx.ast);
                parameter.set_initializer(ctx.ast, Some(undefined));
                BindingPattern::AssignmentPattern(ctx.ast.assignment_pattern(
                    DUMMY_SP,
                    pattern,
                    initializer,
                ))
            } else {
                pattern
            };
            patterns.push(pattern);
            values.push(ctx.state.resolver.reference(ctx.ast, binding));
        }
        if let Some(rest) = rest {
            let binding = ctx
                .state
                .resolver
                .private_binding(ctx.ast, &format!("_{}", items.len()));
            let pattern = rest.argument(ctx.ast).into();
            rest.set_argument(ctx.ast, FormalParameterPattern::BindingIdentifier(binding));
            patterns.push(pattern);
            values.push(ctx.state.resolver.reference(ctx.ast, binding));
        }
        let mut lowerer = RestLowerer {
            ast: ctx.ast,
            context: &mut ctx.state,
            options: self.options,
            out: RestOutput::Declarations(Vec::new()),
        };
        if patterns.len() == 1 && !patterns[0].is_assignment_pattern(lowerer.ast) {
            lowerer.visit(patterns[0], values[0]);
        } else if patterns
            .iter()
            .any(|&pattern| has_object_rest_in_pat(lowerer.ast, pattern))
        {
            let array = lowerer.declare_temp("_ref");
            let elements = lowerer
                .ast
                .add_sub_range(values.into_iter().map(|value| Some(Argument::Expr(value))));
            let value = lowerer.ast.expr_array_expression(DUMMY_SP, elements);
            lowerer.assign(BindingPattern::BindingIdentifier(array), value);
            let mut current = array;
            let count = patterns.len();
            for (index, pattern) in patterns.into_iter().enumerate() {
                let (inner, initializer) =
                    if let Some(assignment) = pattern.as_assignment_pattern(lowerer.ast) {
                        (
                            assignment.left(lowerer.ast),
                            Some(assignment.right(lowerer.ast)),
                        )
                    } else {
                        (pattern, None)
                    };
                let needs_lowering = has_object_rest_in_pat(lowerer.ast, inner);
                let binding = needs_lowering.then(|| lowerer.declare_temp("_ref"));
                let element = if let Some(binding) = binding {
                    let pattern = BindingPattern::BindingIdentifier(binding);
                    if let Some(initializer) = initializer {
                        BindingPattern::AssignmentPattern(lowerer.ast.assignment_pattern(
                            DUMMY_SP,
                            pattern,
                            initializer,
                        ))
                    } else {
                        pattern
                    }
                } else {
                    pattern
                };
                let mut elements = vec![Some(element)];
                let tail = (index + 1 != count).then(|| lowerer.declare_temp("_rest"));
                if let Some(tail) = tail {
                    elements.push(Some(BindingPattern::BindingRestElement(
                        lowerer.ast.binding_rest_element(
                            DUMMY_SP,
                            BindingPattern::BindingIdentifier(tail),
                        ),
                    )));
                }
                let array_pattern = build::array_pattern(lowerer.ast, elements);
                let value = lowerer.reference(current);
                lowerer.assign(array_pattern, value);
                if let Some(tail) = tail {
                    current = tail;
                }
                if let Some(binding) = binding {
                    let value = lowerer.reference(binding);
                    lowerer.visit(inner, value);
                }
            }
        } else {
            let pattern =
                build::array_pattern(lowerer.ast, patterns.into_iter().map(Some).collect());
            let elements = lowerer
                .ast
                .add_sub_range(values.into_iter().map(|value| Some(Argument::Expr(value))));
            let value = lowerer.ast.expr_array_expression(DUMMY_SP, elements);
            lowerer.assign(pattern, value);
        }
        let RestOutput::Declarations(declarations) = lowerer.out else {
            unreachable!()
        };
        vec![build::declaration(ctx.ast, VariableKind::Let, declarations)]
    }
}
