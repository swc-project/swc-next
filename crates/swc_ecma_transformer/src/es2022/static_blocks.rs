//! ES2022 static blocks transform.
//!
//! Supports both a compact synthetic private static field lowering and a
//! closure lowering that remains correct for non-extensible classes.
use swc_next_ecma_ast::{
    Accessibility, ArrowFunctionBody, ArrowFunctionExpression, AssignmentOperator,
    AssignmentTarget, Ast, AstUtf8, AstWtf8, Class, ClassBody, ClassElement, ClassType, DUMMY_SP,
    Expr, GetSpan, NodeId, NodeKindData, PropertyDefinition, PropertyKey, ScopeId, Span,
    StaticBlock, Stmt, SubRange, UnaryOperator,
};

use crate::{
    TransformFeatureFlags,
    context::{ClassElementAction, Mutation, ScopeOptions},
};
use swc_next_ecma_semantic::{ClassArrowInfo, ClassStaticBlockLayout, scope::ScopeKind};

pub(crate) struct StaticBlocks {
    features: TransformFeatureFlags,
    expression_calls: Vec<(NodeId, Expr)>,
    pending: Vec<BlockExpression>,
    expressions: Vec<Expr>,
}

struct ClassTransform<'visit, 'c, 's, 'a> {
    ctx: &'visit mut Mutation<'c, 's, 'a>,
    pending: &'visit mut Vec<BlockExpression>,
    expressions: &'visit mut Vec<Expr>,
    generated_names: &'visit mut Vec<AstUtf8>,
}

impl<'a> ClassTransform<'_, '_, '_, 'a> {
    fn next_closure_name(&mut self) -> AstUtf8 {
        self.ctx.generate_var_name("_staticBlock")
    }

    /// Folds blocks that precede a static field into that field's initializer.
    /// Trailing blocks are moved into a closure captured by the last static
    /// field and invoked after the class has finished evaluating.
    ///
    /// ```text
    /// class C { static x = 1; static { use(this); } }
    ///
    /// var _staticBlock;
    /// class C { static x = (_staticBlock = () => use(this), 1); }
    /// _staticBlock();
    /// ```
    fn transform_class_supporting_non_extensible_class(
        &mut self,
        class: Class,
        is_expression: bool,
        inferred_name: Option<InferredName>,
    ) -> Option<(Expr, AstUtf8)> {
        let body = class.body(self.ctx.ast());
        let layout = self.ctx.take_static_block_layout(class.node_id());
        debug_assert!(self.pending.is_empty());
        let mut trailing = layout.last_static_property.is_none();
        let mut first_source = true;
        let mut expression_closure = None;
        let mut trailing_closure = None;
        let mut closure_name = None;
        let mut assignment_span = DUMMY_SP;

        {
            let Self {
                ctx,
                pending,
                expressions,
                generated_names,
            } = self;
            ctx.filter_map_class_elements(body, |ctx, element| {
                let mut transform = ClassTransform {
                    ctx,
                    pending,
                    expressions,
                    generated_names,
                };
                if let Some(block) = element.as_static_block(transform.ctx.ast()) {
                    let Some(mut block) = transform.block_to_expression(block) else {
                        return ClassElementAction::Delete;
                    };
                    if trailing && closure_name.is_none() {
                        expression_closure =
                            transform.prepare_trailing_closure(block, &layout, inferred_name);
                        closure_name = Some(transform.next_closure_name());
                        let first = transform.pending.first().copied().unwrap_or(block);
                        if first.block_body.is_none() {
                            assignment_span = first.span;
                        }
                    }
                    if trailing && expression_closure.is_none() {
                        // The single trailing statement block becomes the closure itself.
                        transform.pending.push(block);
                        trailing_closure =
                            Some(transform.build_trailing_closure(is_expression, None));
                        transform.pending.clear();
                    } else {
                        let parent = if trailing {
                            expression_closure
                                .expect("trailing expression scope exists")
                                .1
                        } else {
                            transform.ctx.current_scope()
                        };
                        block.expression =
                            Some(transform.materialize_block(block, first_source, parent));
                        block.block_body = None;
                        transform.pending.push(block);
                    }
                    first_source = false;
                    return ClassElementAction::Delete;
                }

                if let Some(property) = static_property(element, transform.ctx.ast()) {
                    if !transform.pending.is_empty() {
                        transform.prepend_pending_to_initializer(property);
                    }
                    first_source = true;
                    trailing = Some(property) == layout.last_static_property;
                }
                ClassElementAction::Unchanged
            });
        }

        let Some(closure_name) = closure_name else {
            debug_assert!(self.pending.is_empty());
            return None;
        };
        let closure = trailing_closure
            .unwrap_or_else(|| self.build_trailing_closure(is_expression, expression_closure));
        let assignment = self.build_assignment(closure_name, closure, assignment_span);
        if let Some(property) = layout.last_static_property {
            self.prepend_one_to_initializer(property, assignment);
        } else {
            let private_name = self.ctx.next_private_name(class.node_id());
            let property = self.build_fallback_property(private_name, assignment);
            let index = body.body(self.ctx.ast()).len();
            self.ctx
                .insert_class_element(body, index, ClassElement::PropertyDefinition(property));
        }
        self.pending.clear();
        Some((self.build_call(closure_name), closure_name))
    }

    fn prepare_trailing_closure(
        &mut self,
        first_block: BlockExpression,
        layout: &ClassStaticBlockLayout,
        inferred_name: Option<InferredName>,
    ) -> Option<(ArrowFunctionExpression, ScopeId)> {
        let expression_closure = if layout.trailing_blocks != 1
            || first_block.block_body.is_none()
            || (inferred_name.is_some() && layout.first_static_property.is_none())
        {
            Some(self.create_trailing_arrow(layout.trailing_lexical_super))
        } else {
            None
        };
        if let Some(name) = inferred_name {
            let scope = if layout.first_static_property.is_none() {
                expression_closure
                    .expect("name restoration has an expression closure")
                    .1
            } else {
                self.ctx.current_scope()
            };
            let name = self.prepare_inferred_name(name, scope);
            let target = Expr::ThisExpression(
                self.ctx
                    .add_node(scope, |ast| ast.this_expression(DUMMY_SP)),
            );
            let set_name = self.build_set_function_name(target, name);
            if let Some(property) = layout.first_static_property {
                self.prepend_one_to_initializer(property, set_name);
            } else {
                self.pending.push(BlockExpression {
                    expression: Some(set_name),
                    block_body: None,
                    span: DUMMY_SP,
                    source_block: None,
                });
            }
        }
        expression_closure
    }

    fn block_to_expression(&mut self, block: StaticBlock) -> Option<BlockExpression> {
        let statements = block.body(self.ctx.ast());
        let span = block.span(self.ctx.ast());
        if statements.is_empty()
            && !self
                .ctx
                .semantic()
                .class()
                .static_block_has_comments(block.node_id())
        {
            return None;
        }
        if let Some(expression) = single_expression_statement(statements, self.ctx.ast()) {
            return Some(BlockExpression {
                expression: Some(expression),
                block_body: None,
                span,
                source_block: Some(block.node_id()),
            });
        }
        Some(BlockExpression {
            expression: None,
            block_body: Some(statements),
            span,
            source_block: Some(block.node_id()),
        })
    }

    fn materialize_block(&mut self, block: BlockExpression, first: bool, parent: ScopeId) -> Expr {
        if let Some(expression) = block.expression {
            if let Some(source) = block.source_block {
                inline_static_block_expression(source, parent, self.ctx);
            }
            return expression;
        }
        let source = block.source_block.expect("statement block has an owner");
        let lexical_super = self
            .ctx
            .semantic()
            .class()
            .static_block_preserves_lexical_super(source);
        let mut expression = self.build_iife(
            block.span,
            block.block_body.expect("statement block has a body"),
            lexical_super,
            source,
            parent,
        );
        if self
            .ctx
            .semantic()
            .class()
            .static_block_has_comments(source)
        {
            let span = if first { DUMMY_SP } else { block.span };
            expression = Expr::ParenthesizedExpression(
                self.ctx
                    .add_node(parent, |ast| ast.parenthesized_expression(span, expression)),
            );
        }
        expression
    }

    fn create_trailing_arrow(&mut self, lexical_super: bool) -> (ArrowFunctionExpression, ScopeId) {
        self.ctx.add_scope_node(
            ScopeOptions {
                parent: self.ctx.current_scope(),
                kind: ScopeKind::Function,
                reuse: None,
                environment: Some(ClassArrowInfo {
                    property_initializer: Some(
                        self.ctx.semantic().scope(self.ctx.current_scope()).node,
                    ),
                    lexical_super,
                    static_block_environment: true,
                }),
            },
            |ast| {
                let body = ArrowFunctionBody::FunctionBody(ast.function_body(
                    DUMMY_SP,
                    SubRange::empty(),
                    SubRange::empty(),
                ));
                let params = ast.formal_parameters(
                    DUMMY_SP,
                    SubRange::empty(),
                    None,
                    swc_next_ecma_ast::FormalParameterKind::FormalParameters,
                );
                ast.arrow_function_expression(DUMMY_SP, false, false, None, params, None, body)
            },
        )
    }

    fn build_iife(
        &mut self,
        span: Span,
        statements: SubRange<Stmt>,
        preserve_lexical_super: bool,
        source: NodeId,
        parent: ScopeId,
    ) -> Expr {
        let scope = self
            .ctx
            .scope_for_node(source)
            .expect("static block scope exists");
        let body = ArrowFunctionBody::FunctionBody(self.ctx.add_node(scope, |ast| {
            ast.function_body(span, SubRange::empty(), statements)
        }));
        let arrow = self.build_arrow(span, body, false, preserve_lexical_super, source, parent);
        Expr::CallExpression(self.ctx.add_node(parent, |ast| {
            ast.call_expression(span, arrow, None, SubRange::empty(), false)
        }))
    }

    fn build_trailing_closure(
        &mut self,
        needs_completion_value: bool,
        expression_closure: Option<(ArrowFunctionExpression, ScopeId)>,
    ) -> Expr {
        if expression_closure.is_none() {
            let block = self.pending[0];
            let source = block.source_block.expect("block has a source scope");
            let scope = self
                .ctx
                .scope_for_node(source)
                .expect("source scope exists");
            let statements = block.block_body.expect("checked above");
            let statements = if needs_completion_value {
                let mut body = Vec::with_capacity(statements.len() + 1);
                body.extend(
                    statements
                        .iter()
                        .map(|id| self.ctx.ast().get_node_in_sub_range(id)),
                );
                let this = Expr::ThisExpression(
                    self.ctx
                        .add_node(scope, |ast| ast.this_expression(DUMMY_SP)),
                );
                body.push(Stmt::ReturnStatement(self.ctx.add_node(scope, |ast| {
                    ast.return_statement(DUMMY_SP, Some(this))
                })));
                self.ctx.add_sub_range(body)
            } else {
                statements
            };
            let body = ArrowFunctionBody::FunctionBody(self.ctx.add_node(scope, |ast| {
                ast.function_body(DUMMY_SP, SubRange::empty(), statements)
            }));
            let preserve_lexical_super = self
                .ctx
                .semantic()
                .class()
                .static_block_preserves_lexical_super(
                    block
                        .source_block
                        .expect("block expression has a source block"),
                );
            let replacement = self.build_arrow(
                DUMMY_SP,
                body,
                false,
                preserve_lexical_super,
                source,
                self.ctx.current_scope(),
            );
            return replacement;
        }

        let (arrow, scope) = expression_closure.expect("expression closure was prepared");
        self.expressions.clear();
        self.expressions.extend(self.pending.iter().map(|block| {
            block
                .expression
                .expect("block scope was transferred before deletion")
        }));
        if needs_completion_value {
            self.expressions.push(Expr::ThisExpression(
                self.ctx
                    .add_node(scope, |ast| ast.this_expression(DUMMY_SP)),
            ));
        }
        let expression = if self.expressions.len() == 1 {
            self.expressions[0]
        } else {
            let items = self.ctx.add_sub_range(self.expressions.iter().copied());
            Expr::SequenceExpression(
                self.ctx
                    .add_node(scope, |ast| ast.sequence_expression(DUMMY_SP, items)),
            )
        };
        self.ctx.set_arrow_expression_body(arrow, expression);
        Expr::ArrowFunctionExpression(arrow)
    }

    fn build_arrow(
        &mut self,
        span: Span,
        body: ArrowFunctionBody,
        expression: bool,
        lexical_super: bool,
        source: NodeId,
        parent: ScopeId,
    ) -> Expr {
        let reuse = self
            .ctx
            .scope_for_node(source)
            .expect("static block scope exists");
        build_static_block_arrow(
            span,
            body,
            expression,
            lexical_super,
            reuse,
            parent,
            self.ctx,
        )
        .0
    }

    fn sequence_or_single(&mut self) -> Expr {
        if self.expressions.len() == 1 {
            self.expressions[0]
        } else {
            let expressions = self.ctx.add_sub_range(self.expressions.iter().copied());
            Expr::SequenceExpression(self.ctx.add_node(self.ctx.current_scope(), |ast| {
                ast.sequence_expression(DUMMY_SP, expressions)
            }))
        }
    }

    fn prepend_pending_to_initializer(&mut self, property: PropertyDefinition) {
        self.expressions.clear();
        self.expressions.extend(self.pending.iter().map(|block| {
            block
                .expression
                .expect("block scope was transferred before deletion")
        }));
        self.pending.clear();
        self.prepend_expressions_to_initializer(property);
    }

    fn prepend_one_to_initializer(&mut self, property: PropertyDefinition, expression: Expr) {
        self.expressions.clear();
        self.expressions.push(expression);
        self.prepend_expressions_to_initializer(property);
    }

    fn prepend_expressions_to_initializer(&mut self, property: PropertyDefinition) {
        if let Some(value) = property.value(self.ctx.ast()) {
            let value = if anonymous_function_or_class(value, self.ctx.ast()) {
                inferred_property_key_name(
                    property.key(self.ctx.ast()),
                    property.computed(self.ctx.ast()),
                    property.node_id(),
                    self.ctx,
                )
                .map(|name| self.prepare_inferred_name(name, self.ctx.current_scope()))
                .map_or(value, |name| self.build_set_function_name(value, name))
            } else {
                value
            };
            self.ctx.wrap_property_value(property, value);
            self.expressions.push(value);
            let expressions = self.ctx.add_sub_range(self.expressions.iter().copied());
            let sequence =
                Expr::SequenceExpression(self.ctx.add_node(self.ctx.current_scope(), |ast| {
                    ast.sequence_expression(DUMMY_SP, expressions)
                }));
            self.ctx.wrap_property_value(property, sequence);
            return;
        }

        let last = self
            .expressions
            .pop()
            .expect("at least one expression must be prepended");
        let last = Expr::UnaryExpression(self.ctx.add_node(self.ctx.current_scope(), |ast| {
            ast.unary_expression(DUMMY_SP, last, UnaryOperator::Void)
        }));
        self.expressions.push(last);
        let value = self.sequence_or_single();
        self.ctx.set_property_value(property, value);
    }

    fn build_assignment(&mut self, name: AstUtf8, value: Expr, span: Span) -> Expr {
        self.ctx
            .assign_generated(name, value, span, self.ctx.current_scope())
    }

    fn build_call(&mut self, name: AstUtf8) -> Expr {
        call_generated_after_class(name, self.ctx)
    }

    fn build_fallback_property(
        &mut self,
        key: PropertyKey,
        initializer: Expr,
    ) -> PropertyDefinition {
        self.ctx.add_node(self.ctx.current_scope(), |ast| {
            ast.property_definition(
                DUMMY_SP,
                SubRange::empty(),
                key,
                None,
                Some(initializer),
                false,
                true,
                false,
                false,
                false,
                false,
                false,
                false,
                false,
                Accessibility::None,
            )
        })
    }

    fn build_set_function_name(&mut self, target: Expr, name: InferredName) -> Expr {
        let scope = self
            .ctx
            .semantic()
            .scope_outside(target.node_id())
            .expect("name target is registered");
        let name = match name {
            InferredName::String(name) => Expr::StringLiteral(self.ctx.add_node(scope, |ast| {
                ast.string_literal(DUMMY_SP, name, ast.empty_utf8())
            })),
            InferredName::Expr(name) => name,
            InferredName::Computed { .. } => {
                unreachable!("computed names are prepared before building the helper call")
            }
        };
        crate::helpers::call_set_function_name(target, name, self.ctx)
    }

    fn prepare_inferred_name(&mut self, name: InferredName, scope: ScopeId) -> InferredName {
        let InferredName::Computed { parent, key } = name else {
            return name;
        };
        let temp = self.ctx.generate_var_name("_computedKey");
        self.generated_names.push(temp);
        let key_scope = self
            .ctx
            .semantic()
            .scope_outside(key.node_id())
            .expect("computed key is registered");
        let key = crate::helpers::call_to_property_key(key, self.ctx);
        self.ctx.wrap_property_key(parent, PropertyKey::Expr(key));
        let assignment = self.ctx.assign_generated(temp, key, DUMMY_SP, key_scope);
        let assignment = PropertyKey::Expr(assignment);
        self.ctx.wrap_property_key(parent, assignment);
        InferredName::Expr(Expr::IdentifierReference(
            self.ctx.generated_reference_in_scope(temp, DUMMY_SP, scope),
        ))
    }
}

impl StaticBlocks {
    pub(crate) fn new(features: TransformFeatureFlags) -> Self {
        Self {
            features,
            expression_calls: Vec::new(),
            pending: Vec::new(),
            expressions: Vec::new(),
        }
    }

    pub(super) fn enter_class_body(&mut self, body: ClassBody, ctx: &mut Mutation<'_, '_, '_>) {
        // A function body's `var` scope is not visible while default
        // parameters are evaluated. Keep these class expressions self-
        // contained instead of hoisting the closure binding into the body.
        let in_formal_parameters =
            ctx.ancestors()
                .find_map(|ancestor| match ctx.ast().kind_data(ancestor) {
                    NodeKindData::FormalParameters(_) => Some(true),
                    NodeKindData::Function(_) | NodeKindData::ArrowFunctionExpression(_) => {
                        Some(false)
                    }
                    _ => None,
                })
                == Some(true);
        let class = match ctx.ast().kind_data(
            ctx.parent()
                .expect("a class body must have its class as the parent"),
        ) {
            NodeKindData::Class(class) => class,
            _ => unreachable!("a class body must have its class as the parent"),
        };
        debug_assert_eq!(class.body(ctx.ast()).node_id(), body.node_id());
        let has_static_blocks = ctx
            .semantic()
            .class()
            .static_block_layout(class.node_id())
            .map_or_else(
                || {
                    body.body(ctx.ast()).iter().any(|id| {
                        ctx.ast()
                            .get_node_in_sub_range(id)
                            .as_static_block(ctx.ast())
                            .is_some()
                    })
                },
                |layout| layout.has_static_blocks,
            );
        if !has_static_blocks {
            return;
        }
        if !self
            .features
            .contains(TransformFeatureFlags::PROPOSAL_PREVENT_EXTENSIONS)
            || in_formal_parameters
        {
            transform_class_as_private_fields(body, class.node_id(), ctx);
            return;
        }

        let is_expression = class.class_type(ctx.ast()) == ClassType::ClassExpression;
        let inferred_name = if is_expression {
            let ancestors = ctx.ancestors().skip(2).collect::<Vec<_>>();
            inferred_name(class, ancestors, ctx)
        } else {
            None
        };
        let mut generated_names = Vec::new();
        let call = ClassTransform {
            ctx,
            pending: &mut self.pending,
            expressions: &mut self.expressions,
            generated_names: &mut generated_names,
        }
        .transform_class_supporting_non_extensible_class(
            class,
            is_expression,
            inferred_name,
        );

        let Some((call, closure_name)) = call else {
            return;
        };
        for name in generated_names {
            ctx.inject_var(name);
        }
        if is_expression {
            ctx.inject_var(closure_name);
            self.expression_calls.push((class.node_id(), call));
        } else {
            insert_call_after_class(call, closure_name, ctx);
        }
    }

    pub(super) fn exit_class<'a>(&mut self, class: Class, ctx: &mut Mutation<'_, '_, 'a>) {
        if self
            .expression_calls
            .last()
            .is_none_or(|(class_id, _)| *class_id != class.node_id())
        {
            return;
        }
        let (_, call) = self
            .expression_calls
            .pop()
            .expect("matching class expression call exists");
        let expressions = ctx.add_sub_range([Expr::Class(class), call]);
        let scope = ctx
            .semantic()
            .scope_outside(class.node_id())
            .expect("class has an outer scope");
        let replacement = Expr::SequenceExpression(
            ctx.add_node(scope, |ast| ast.sequence_expression(DUMMY_SP, expressions)),
        );
        ctx.wrap_current_expr(replacement);
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.expression_calls.is_empty()
    }
}

fn insert_call_after_class<'a>(call: Expr, closure_name: AstUtf8, ctx: &mut Mutation<'_, '_, 'a>) {
    let scope = ctx
        .semantic()
        .scope_at(call.node_id())
        .expect("call is registered");
    let call = Stmt::ExpressionStatement(
        ctx.add_node(scope, |ast| ast.expression_statement(DUMMY_SP, call)),
    );
    ctx.inject_var(closure_name);
    ctx.inject_statement_after_current(call);
}

fn inferred_name(
    class: Class,
    ancestors: impl IntoIterator<Item = NodeId>,
    ctx: &mut Mutation<'_, '_, '_>,
) -> Option<InferredName> {
    if class.id(ctx.ast()).is_some() {
        return None;
    }
    let mut value = class.node_id();
    for parent in ancestors {
        let transparent_value = match ctx.ast().kind_data(parent) {
            NodeKindData::ParenthesizedExpression(node) => Some(node.expression(ctx.ast())),
            NodeKindData::TsAsExpression(node) => Some(node.expression(ctx.ast())),
            NodeKindData::TsSatisfiesExpression(node) => Some(node.expression(ctx.ast())),
            NodeKindData::TsTypeAssertion(node) => Some(node.expression(ctx.ast())),
            NodeKindData::TsNonNullExpression(node) => Some(node.expression(ctx.ast())),
            NodeKindData::TsInstantiationExpression(node) => Some(node.expression(ctx.ast())),
            _ => None,
        };
        if transparent_value.is_some_and(|expression| expression.node_id() == value) {
            value = parent;
            continue;
        }

        return match ctx.ast().kind_data(parent) {
            NodeKindData::VariableDeclarator(node)
                if node
                    .init(ctx.ast())
                    .is_some_and(|init| init.node_id() == value) =>
            {
                node.id(ctx.ast())
                    .as_binding_identifier(ctx.ast())
                    .map(|identifier| InferredName::String(identifier.name(ctx.ast()).into()))
            }
            NodeKindData::ObjectProperty(node) if node.value(ctx.ast()).node_id() == value => {
                inferred_property_key_name(
                    node.key(ctx.ast()),
                    node.computed(ctx.ast()),
                    node.node_id(),
                    ctx,
                )
            }
            NodeKindData::AssignmentExpression(node)
                if matches!(
                    node.operator(ctx.ast()),
                    AssignmentOperator::Assign
                        | AssignmentOperator::LogicalAndAssign
                        | AssignmentOperator::LogicalOrAssign
                        | AssignmentOperator::NullishAssign
                ) && node.right(ctx.ast()).node_id() == value =>
            {
                assignment_target_name(node.left(ctx.ast()), ctx.ast())
            }
            NodeKindData::AssignmentPattern(node) if node.right(ctx.ast()).node_id() == value => {
                let left = node.left(ctx.ast());
                left.as_binding_identifier(ctx.ast())
                    .map(|identifier| InferredName::String(identifier.name(ctx.ast()).into()))
                    .or_else(|| {
                        left.as_simple_assignment_target(ctx.ast())?
                            .as_identifier_reference(ctx.ast())
                            .map(|identifier| {
                                InferredName::String(identifier.name(ctx.ast()).into())
                            })
                    })
            }
            NodeKindData::PropertyDefinition(node)
                if node
                    .value(ctx.ast())
                    .is_some_and(|init| init.node_id() == value) =>
            {
                inferred_property_key_name(
                    node.key(ctx.ast()),
                    node.computed(ctx.ast()),
                    node.node_id(),
                    ctx,
                )
            }
            _ => None,
        };
    }
    None
}

fn property_key_name(key: PropertyKey, ctx: &mut Mutation<'_, '_, '_>) -> Option<InferredName> {
    if let Some(literal) = key
        .as_expr(ctx.ast())
        .and_then(|key| key.as_string_literal(ctx.ast()))
    {
        return Some(InferredName::String(literal.value(ctx.ast())));
    }
    if let Some(identifier) = key.as_identifier_name(ctx.ast()) {
        return Some(InferredName::String(identifier.name(ctx.ast()).into()));
    }
    if let Some(private) = key
        .as_expr(ctx.ast())
        .and_then(|key| key.as_private_identifier(ctx.ast()))
    {
        let name = format!("#{}", ctx.ast().get_utf8(private.name(ctx.ast())));
        return Some(InferredName::String(ctx.intern_wtf8(&name)));
    }
    if let Some(number) = key
        .as_expr(ctx.ast())
        .and_then(|key| key.as_numeric_literal(ctx.ast()))
    {
        let value = Expr::NumericLiteral(ctx.add_node(ctx.current_scope(), |ast| {
            ast.numeric_literal(DUMMY_SP, number.value(ast), number.raw(ast))
        }));
        return Some(InferredName::Expr(value));
    }
    if let Some(bigint) = key
        .as_expr(ctx.ast())
        .and_then(|key| key.as_big_int_literal(ctx.ast()))
    {
        let value = Expr::BigIntLiteral(ctx.add_node(ctx.current_scope(), |ast| {
            ast.big_int_literal(DUMMY_SP, bigint.raw(ast))
        }));
        return Some(InferredName::Expr(value));
    }
    None
}

fn assignment_target_name(target: AssignmentTarget, ast: &Ast<'_>) -> Option<InferredName> {
    let target = target.as_simple_assignment_target(ast)?;
    target
        .as_identifier_reference(ast)
        .map(|identifier| InferredName::String(identifier.name(ast).into()))
}

#[derive(Clone, Copy)]
enum InferredName {
    String(AstWtf8),
    Expr(Expr),
    Computed { parent: NodeId, key: Expr },
}

fn inferred_property_key_name(
    key: PropertyKey,
    computed: bool,
    parent: NodeId,
    ctx: &mut Mutation<'_, '_, '_>,
) -> Option<InferredName> {
    property_key_name(key, ctx).or_else(|| {
        computed
            .then(|| key.as_expr(ctx.ast()))
            .flatten()
            .map(|key| InferredName::Computed { parent, key })
    })
}

fn anonymous_function_or_class(value: Expr, ast: &Ast<'_>) -> bool {
    match ast.kind_data(value.node_id()) {
        NodeKindData::Function(function) => function.id(ast).is_none(),
        NodeKindData::ArrowFunctionExpression(_) => true,
        NodeKindData::Class(class) => class.id(ast).is_none(),
        NodeKindData::ParenthesizedExpression(expression) => {
            anonymous_function_or_class(expression.expression(ast), ast)
        }
        NodeKindData::TsAsExpression(expression) => {
            anonymous_function_or_class(expression.expression(ast), ast)
        }
        NodeKindData::TsSatisfiesExpression(expression) => {
            anonymous_function_or_class(expression.expression(ast), ast)
        }
        NodeKindData::TsTypeAssertion(expression) => {
            anonymous_function_or_class(expression.expression(ast), ast)
        }
        NodeKindData::TsNonNullExpression(expression) => {
            anonymous_function_or_class(expression.expression(ast), ast)
        }
        NodeKindData::TsInstantiationExpression(expression) => {
            anonymous_function_or_class(expression.expression(ast), ast)
        }
        _ => false,
    }
}

#[derive(Clone, Copy)]
struct BlockExpression {
    expression: Option<Expr>,
    block_body: Option<SubRange<Stmt>>,
    span: Span,
    source_block: Option<NodeId>,
}

fn static_property(element: ClassElement, ast: &Ast<'_>) -> Option<PropertyDefinition> {
    element.as_property_definition(ast).filter(|property| {
        property.r#static(ast) && !property.declare(ast) && !property.r#abstract(ast)
    })
}

/// Replaces each static block in place with a synthetic private static field
/// whose initializer evaluates the block body.
///
/// ```text
/// class C { static { initialize(); } }
///
/// class C { static #_ = initialize(); }
/// ```
fn transform_class_as_private_fields(
    body: ClassBody,
    class: NodeId,
    ctx: &mut Mutation<'_, '_, '_>,
) {
    ctx.filter_map_class_elements(body, |ctx, element| {
        if let Some(block) = element.as_static_block(ctx.ast()) {
            if block.body(ctx.ast()).is_empty() {
                let has_attached_comments = ctx
                    .semantic()
                    .class()
                    .static_block_has_comments(block.node_id());
                if !has_attached_comments {
                    return ClassElementAction::Delete;
                }
            }
            let name = ctx.next_private_name(class);
            ClassElementAction::Update(convert_block_to_private_field(block, name, ctx))
        } else {
            ClassElementAction::Unchanged
        }
    });
}

fn convert_block_to_private_field(
    block: StaticBlock,
    key: PropertyKey,
    ctx: &mut Mutation<'_, '_, '_>,
) -> ClassElement {
    let expression = convert_block_to_expression(block, ctx);
    ClassElement::PropertyDefinition(ctx.add_node(ctx.current_scope(), |ast| {
        ast.property_definition(
            block.span(ast),
            SubRange::empty(),
            key,
            None,
            Some(expression),
            false,
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            Accessibility::None,
        )
    }))
}

fn convert_block_to_expression(block: StaticBlock, ctx: &mut Mutation<'_, '_, '_>) -> Expr {
    let statements = block.body(ctx.ast());
    if let Some(expression) = single_expression_statement(statements, ctx.ast()) {
        inline_static_block_expression(block.node_id(), ctx.current_scope(), ctx);
        expression
    } else {
        let preserve_lexical_super = ctx
            .semantic()
            .class()
            .static_block_preserves_lexical_super(block.node_id());
        wrap_statements_in_arrow_function_iife(
            statements,
            block.span(ctx.ast()),
            preserve_lexical_super,
            block.node_id(),
            ctx,
        )
    }
}

fn wrap_statements_in_arrow_function_iife(
    statements: SubRange<Stmt>,
    span: Span,
    preserve_lexical_super: bool,
    source: NodeId,
    ctx: &mut Mutation<'_, '_, '_>,
) -> Expr {
    let scope = ctx
        .scope_for_node(source)
        .expect("static block scope exists");
    let body = ArrowFunctionBody::FunctionBody(ctx.add_node(scope, |ast| {
        ast.function_body(span, SubRange::empty(), statements)
    }));
    let (arrow, _) = build_static_block_arrow(
        span,
        body,
        false,
        preserve_lexical_super,
        scope,
        ctx.current_scope(),
        ctx,
    );
    Expr::CallExpression(ctx.add_node(ctx.current_scope(), |ast| {
        ast.call_expression(span, arrow, None, SubRange::empty(), false)
    }))
}

fn single_expression_statement(statements: SubRange<Stmt>, ast: &Ast<'_>) -> Option<Expr> {
    if statements.len() != 1 {
        return None;
    }
    statements
        .get_node(ast, 0)
        .expect("single statement must exist")
        .as_expression_statement(ast)
        .map(|statement| statement.expression(ast))
}

fn call_generated_after_class(name: AstUtf8, ctx: &mut Mutation<'_, '_, '_>) -> Expr {
    use swc_next_ecma_ast::{DUMMY_SP, Expr, NodeKindData, SubRange};
    let mut ancestors = ctx.ancestors();
    let _ = ancestors.find(|node| matches!(ctx.ast().kind_data(*node), NodeKindData::Class(_)));
    let scope = ancestors
        .find_map(|owner| ctx.scope_for_node(owner))
        .expect("class has an enclosing scope");
    drop(ancestors);
    let callee = ctx.generated_reference_in_scope(name, DUMMY_SP, scope);
    Expr::CallExpression(ctx.add_node(scope, |ast| {
        ast.call_expression(
            DUMMY_SP,
            Expr::IdentifierReference(callee),
            None,
            SubRange::empty(),
            false,
        )
    }))
}

fn build_static_block_arrow(
    span: Span,
    body: ArrowFunctionBody,
    expression: bool,
    lexical_super: bool,
    reuse: ScopeId,
    parent: ScopeId,
    ctx: &mut Mutation<'_, '_, '_>,
) -> (Expr, ScopeId) {
    let options = ScopeOptions {
        parent,
        kind: ScopeKind::Function,
        reuse: Some(reuse),
        environment: Some(ClassArrowInfo {
            property_initializer: Some(ctx.semantic().scope(ctx.current_scope()).node),
            lexical_super,
            static_block_environment: true,
        }),
    };
    let (arrow, scope) = ctx.add_scope_node(options, |ast| {
        let params = ast.formal_parameters(
            DUMMY_SP,
            SubRange::empty(),
            None,
            swc_next_ecma_ast::FormalParameterKind::FormalParameters,
        );
        ast.arrow_function_expression(span, expression, false, None, params, None, body)
    });
    (Expr::ArrowFunctionExpression(arrow), scope)
}

fn inline_static_block_expression(block: NodeId, parent: ScopeId, ctx: &mut Mutation<'_, '_, '_>) {
    ctx.merge_scope(
        block,
        parent,
        ClassArrowInfo {
            property_initializer: Some(ctx.semantic().scope(ctx.current_scope()).node),
            static_block_environment: true,
            ..ClassArrowInfo::default()
        },
    );
}
