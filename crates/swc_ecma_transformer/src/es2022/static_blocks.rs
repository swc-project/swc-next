//! ES2022 static blocks transform.
//!
//! Retains the original private-field and non-extensible-class closure lowering.
//! Synchronizes scopes through TransformCtx and reads class information locally.
use swc_next_ecma_ast::{
    Accessibility, ArrowFunctionBody, ArrowFunctionExpression, AssignmentOperator,
    AssignmentTarget, Ast, AstWtf8, Class, ClassBody, ClassElement, ClassType, DUMMY_SP, Expr,
    FormalParameterKind, GetSpan, NodeId, NodeKindData, PropertyDefinition, PropertyKey, ScopeId,
    Span, StaticBlock, Stmt, SubRange, TraverseAction, UnaryOperator,
};

use crate::{
    context::{BoundIdentifier, ContainerOwnerChild, TransformCtx},
    used_names::UsedNameState,
};
use swc_next_ecma_semantic::scope::{ScopeFlags, ScopeKind};

pub(crate) struct StaticBlocks {
    prevent_extensions: bool,
    class_calls: Vec<(NodeId, Expr)>,
}

impl StaticBlocks {
    pub(crate) fn new(prevent_extensions: bool) -> Self {
        Self {
            prevent_extensions,
            class_calls: Vec::new(),
        }
    }

    pub(super) fn enter_class_body(&mut self, body: ClassBody, ctx: &mut TransformCtx<'_, '_>) {
        let class = match ctx
            .ast()
            .kind_data(ctx.parent().expect("class body parent"))
        {
            NodeKindData::Class(class) => class,
            _ => unreachable!("class body parent"),
        };
        let layout = ClassLayout::read(body, ctx);
        if !layout.has_static_blocks {
            return;
        }
        let mut private_names = private_names(body, ctx);
        // Retain the original fallback: body var bindings are invisible to
        // parameter initializers, which must remain self-contained.
        let in_formal_parameters =
            ctx.ancestors()
                .find_map(|node| match ctx.ast().kind_data(node) {
                    NodeKindData::FormalParameters(_) => Some(true),
                    NodeKindData::Function(_) | NodeKindData::ArrowFunctionExpression(_) => {
                        Some(false)
                    }
                    _ => None,
                })
                == Some(true);
        if !self.prevent_extensions || in_formal_parameters {
            transform_class_as_private_fields(body, &mut private_names, ctx);
            return;
        }
        let name = if class.class_type(ctx.ast()) == ClassType::ClassExpression {
            let ancestors = ctx.ancestors().skip(2).collect::<Vec<_>>();
            inferred_name(class, ancestors, ctx)
        } else {
            None
        };
        if matches!(name, Some(InferredName::Computed { parent, .. })
            if matches!(ctx.ast().kind_data(parent), NodeKindData::PropertyDefinition(property) if !property.r#static(ctx.ast())))
            && layout.trailing_blocks != 0
        {
            // Instance initialization can occur after another evaluation has
            // overwritten a hoisted computed-key temporary.
            transform_class_as_private_fields(body, &mut private_names, ctx);
            return;
        }
        let call = transform_supporting_non_extensible_class(
            class,
            name,
            &layout,
            &mut private_names,
            ctx,
        );
        if let Some(call) = call {
            self.class_calls.push((class.node_id(), call));
        }
    }

    pub(super) fn exit_class(
        &mut self,
        class: Class,
        ctx: &mut TransformCtx<'_, '_>,
    ) -> TraverseAction<Expr> {
        if self
            .class_calls
            .last()
            .is_none_or(|(id, _)| *id != class.node_id())
        {
            return TraverseAction::Continue;
        }
        let (_, call) = self.class_calls.pop().expect("matching class call");
        if class.class_type(ctx.ast()) == ClassType::ClassExpression {
            let expressions = ctx.ast.add_sub_range([Expr::Class(class), call]);
            let sequence = Expr::SequenceExpression(
                ctx.add_node(|ast| ast.sequence_expression(DUMMY_SP, expressions)),
            );
            TraverseAction::ReplaceContinue(sequence)
        } else {
            let statement = Stmt::ExpressionStatement(
                ctx.add_node(|ast| ast.expression_statement(DUMMY_SP, call)),
            );
            ctx.insert_after(class.node_id(), statement);
            TraverseAction::Continue
        }
    }
}

/// Fold blocks before fields into their initializers. Capture trailing blocks
/// in the last static field and call the closure after class evaluation.
fn transform_supporting_non_extensible_class(
    class: Class,
    name: Option<InferredName>,
    layout: &ClassLayout,
    names: &mut UsedNameState<'_>,
    ctx: &mut TransformCtx<'_, '_>,
) -> Option<Expr> {
    let body = class.body(ctx.ast());
    let scope = ctx.current_scope();
    let is_expression = class.class_type(ctx.ast()) == ClassType::ClassExpression;
    let mut trailing = layout.last_static_property.is_none();
    let mut first_source = true;
    let mut pending = Vec::new();
    let mut expression_closure = None;
    let mut block_closure = None;
    let mut closure_binding = None;
    let mut assignment_span = DUMMY_SP;
    let mut index = 0;

    while let Some(member) = ctx.container_class_body_mut(body).get(index).copied() {
        if let Some(block) = member.as_static_block(ctx.ast()) {
            if block.body(ctx.ast()).is_empty() && !block_has_comments(block, ctx.ast()) {
                ctx.remove_scope(
                    ctx.semantic()
                        .scope_of(block.node_id())
                        .expect("static block scope"),
                );
                ctx.container_class_body_mut(body).remove(index);
                continue;
            }
            let single_expression =
                single_expression_statement(block.body(ctx.ast()), ctx.ast()).is_some();
            if trailing && closure_binding.is_none() {
                if layout.trailing_blocks != 1
                    || single_expression
                    || (name.is_some() && layout.first_static_property.is_none())
                {
                    expression_closure = Some(create_trailing_arrow(scope, ctx));
                }
                if let Some(name) = name {
                    let name_scope = if layout.first_static_property.is_some() {
                        scope
                    } else {
                        expression_closure.expect("name restoration closure").1
                    };
                    let name = prepare_inferred_name(name, name_scope, ctx);
                    let target =
                        Expr::ThisExpression(ctx.add_node(|ast| ast.this_expression(DUMMY_SP)));
                    let set_name = set_function_name(target, name, name_scope, ctx);
                    if let Some(property) = layout.first_static_property {
                        prepend_to_initializer(property, vec![set_name], ctx);
                    } else {
                        pending.push(set_name);
                    }
                }
                closure_binding = Some(ctx.generate_var(scope, "_staticBlock"));
                if single_expression && pending.is_empty() {
                    assignment_span = block.span(ctx.ast());
                }
            }
            if trailing && expression_closure.is_none() {
                block_closure = Some(block_to_arrow(block, is_expression, scope, ctx));
            } else {
                let parent = expression_closure.map_or(scope, |(_, scope)| scope);
                pending.push(materialize_block(block, first_source, parent, ctx));
            }
            first_source = false;
            ctx.container_class_body_mut(body).remove(index);
            continue;
        }
        if let Some(property) = static_property(member, ctx.ast()) {
            if !pending.is_empty() {
                prepend_to_initializer(property, std::mem::take(&mut pending), ctx);
            }
            first_source = true;
            trailing = Some(property) == layout.last_static_property;
        }
        index += 1;
    }

    let binding = closure_binding?;
    let closure = if let Some(closure) = block_closure {
        closure
    } else {
        let (arrow, _) = expression_closure.expect("trailing expression closure");
        if is_expression {
            pending.push(Expr::ThisExpression(
                ctx.add_node(|ast| ast.this_expression(DUMMY_SP)),
            ));
        }
        let value = sequence_or_single(pending, ctx);
        let body = arrow
            .body(ctx.ast())
            .as_function_body(ctx.ast())
            .expect("placeholder body");
        debug_assert!(body.body(ctx.ast()).is_empty());
        debug_assert!(body.directives(ctx.ast()).is_empty());
        arrow.set_body(ctx.ast, ArrowFunctionBody::Expr(value));
        arrow.set_expression(ctx.ast, true);
        Expr::ArrowFunctionExpression(arrow)
    };
    let assignment = binding.assign(ctx, scope, assignment_span, closure);
    if let Some(property) = layout.last_static_property {
        prepend_to_initializer(property, vec![assignment], ctx);
    } else {
        let key = private_name(names, ctx);
        let field = private_field(key, assignment, DUMMY_SP, ctx);
        ctx.push_container(ContainerOwnerChild::ClassBody((body, field)));
    }
    Some(call_generated_after_class(binding, ctx))
}

fn materialize_block(
    block: StaticBlock,
    first: bool,
    parent: ScopeId,
    ctx: &mut TransformCtx<'_, '_>,
) -> Expr {
    if let Some(expression) = single_expression_statement(block.body(ctx.ast()), ctx.ast()) {
        inline_static_block_expression(block.node_id(), parent, ctx);
        return expression;
    }
    let span = block.span(ctx.ast());
    let arrow = block_to_arrow(block, false, parent, ctx);
    let mut expression =
        Expr::CallExpression(ctx.add_node(|ast| {
            ast.call_expression(span, arrow, None, SubRange::empty(), false, false)
        }));
    if block_has_comments(block, ctx.ast()) {
        let span = if first { DUMMY_SP } else { span };
        expression = Expr::ParenthesizedExpression(
            ctx.add_node(|ast| ast.parenthesized_expression(span, expression)),
        );
    }
    ctx.inherits_comments(expression, block)
}

fn create_trailing_arrow(
    parent: ScopeId,
    ctx: &mut TransformCtx<'_, '_>,
) -> (ArrowFunctionExpression, ScopeId) {
    let arrow = ctx.add_node(|ast| {
        let body = ast.function_body(DUMMY_SP, SubRange::empty(), SubRange::empty());
        let params = ast.formal_parameters(
            DUMMY_SP,
            SubRange::empty(),
            None,
            FormalParameterKind::ArrowFormalParameters,
        );
        ast.arrow_function_expression(
            DUMMY_SP,
            false,
            false,
            None,
            params,
            None,
            ArrowFunctionBody::FunctionBody(body),
        )
    });
    let scope = ctx.create_child_scope(
        parent,
        arrow.node_id(),
        ScopeKind::Function,
        ScopeFlags {
            strict: true,
            ..Default::default()
        },
    );
    ctx.move_subtree_to_scope(arrow.params(ctx.ast()), parent, scope);
    ctx.move_subtree_to_scope(arrow.body(ctx.ast()), parent, scope);
    (arrow, scope)
}

fn block_to_arrow(
    block: StaticBlock,
    completion: bool,
    parent: ScopeId,
    ctx: &mut TransformCtx<'_, '_>,
) -> Expr {
    let scope = ctx
        .semantic()
        .scope_of(block.node_id())
        .expect("static block scope");
    let mut statements = block.body(ctx.ast());
    if completion {
        let mut body: Vec<_> = statements
            .iter()
            .map(|slot| ctx.ast().get_node_in_sub_range(slot))
            .collect();
        let value = Expr::ThisExpression(ctx.add_node(|ast| ast.this_expression(DUMMY_SP)));
        body.push(Stmt::ReturnStatement(
            ctx.add_node(|ast| ast.return_statement(DUMMY_SP, Some(value))),
        ));
        statements = ctx.ast.add_sub_range(body);
    }
    let span = block.span(ctx.ast());
    let body = ArrowFunctionBody::FunctionBody(
        ctx.add_node(|ast| ast.function_body(span, SubRange::empty(), statements)),
    );
    let arrow = build_static_block_arrow(span, body, false, scope, parent, ctx).0;
    ctx.inherits_comments(arrow, block)
}

fn sequence_or_single(expressions: Vec<Expr>, ctx: &mut TransformCtx<'_, '_>) -> Expr {
    debug_assert!(!expressions.is_empty());
    if expressions.len() == 1 {
        return expressions[0];
    }
    let items = ctx.ast.add_sub_range(expressions);
    Expr::SequenceExpression(ctx.add_node(|ast| ast.sequence_expression(DUMMY_SP, items)))
}

fn prepend_to_initializer(
    property: PropertyDefinition,
    mut expressions: Vec<Expr>,
    ctx: &mut TransformCtx<'_, '_>,
) {
    let scope = ctx.current_scope();
    if let Some(value) = property.value(ctx.ast()) {
        let value = if anonymous_function_or_class(value, ctx.ast()) {
            if let Some(name) = inferred_property_key_name(
                property.key(ctx.ast()),
                property.computed(ctx.ast()),
                property.node_id(),
                scope,
                ctx,
            ) {
                let name = prepare_inferred_name(name, scope, ctx);
                set_function_name(value, name, scope, ctx)
            } else {
                value
            }
        } else {
            value
        };
        expressions.push(value);
        let sequence = sequence_or_single(expressions, ctx);
        property.set_value(ctx.ast, Some(sequence));
    } else {
        let last = expressions.pop().expect("initializer prefix");
        expressions.push(Expr::UnaryExpression(ctx.add_node(|ast| {
            ast.unary_expression(DUMMY_SP, last, UnaryOperator::Void)
        })));
        let value = sequence_or_single(expressions, ctx);
        property.set_value(ctx.ast, Some(value));
    }
}

fn set_function_name(
    target: Expr,
    name: InferredName,
    scope: ScopeId,
    ctx: &mut TransformCtx<'_, '_>,
) -> Expr {
    let name = match name {
        InferredName::String(name) => Expr::StringLiteral(
            ctx.add_node(|ast| ast.string_literal(DUMMY_SP, name, ast.empty_utf8())),
        ),
        InferredName::Expr(name) => name,
        InferredName::Computed { .. } => unreachable!("computed name must be prepared"),
    };
    ctx.call_set_function_name(target, name, scope)
}

fn prepare_inferred_name(
    name: InferredName,
    scope: ScopeId,
    ctx: &mut TransformCtx<'_, '_>,
) -> InferredName {
    let InferredName::Computed {
        parent,
        key,
        key_scope,
    } = name
    else {
        return name;
    };
    let binding = ctx.generate_var(key_scope, "_computedKey");
    let key = ctx.call_to_property_key(key, key_scope);
    let assignment = binding.assign(ctx, key_scope, DUMMY_SP, key);
    let key = PropertyKey::Expr(assignment);
    match ctx.ast().kind_data(parent) {
        NodeKindData::PropertyDefinition(property) => property.set_key(ctx.ast, key),
        NodeKindData::ObjectProperty(property) => property.set_key(ctx.ast, key),
        _ => unreachable!("property key owner"),
    }
    InferredName::Expr(Expr::IdentifierReference(
        binding.reference_read(ctx, scope, DUMMY_SP),
    ))
}

fn private_field(
    key: PropertyKey,
    value: Expr,
    span: Span,
    ctx: &mut TransformCtx<'_, '_>,
) -> ClassElement {
    ClassElement::PropertyDefinition(ctx.add_node(|ast| {
        ast.property_definition(
            span,
            SubRange::empty(),
            key,
            None,
            Some(value),
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

fn inferred_name(
    class: Class,
    ancestors: impl IntoIterator<Item = NodeId>,
    ctx: &mut TransformCtx<'_, '_>,
) -> Option<InferredName> {
    if class.id(ctx.ast()).is_some() {
        return None;
    }
    let key_scope = ctx
        .semantic()
        .scope(ctx.current_scope())
        .parent
        .expect("class parent scope");
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
                let name = inferred_property_key_name(
                    node.key(ctx.ast()),
                    node.computed(ctx.ast()),
                    node.node_id(),
                    key_scope,
                    ctx,
                );
                // A noncomputed __proto__: value sets the object's prototype;
                // unlike an ordinary property, it does not perform named evaluation.
                if !node.computed(ctx.ast())
                    && matches!(name, Some(InferredName::String(name))
                        if ctx.ast().get_wtf8(name).as_bytes() == b"__proto__")
                {
                    None
                } else {
                    name
                }
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
                    key_scope,
                    ctx,
                )
            }
            _ => None,
        };
    }
    None
}

fn property_key_name(key: PropertyKey, ctx: &mut TransformCtx<'_, '_>) -> Option<InferredName> {
    if let Some(literal) = key
        .as_expr(ctx.ast())
        .and_then(|expr| expr.as_string_literal(ctx.ast()))
    {
        return Some(InferredName::String(literal.value(ctx.ast())));
    }
    if let Some(identifier) = key.as_identifier_name(ctx.ast()) {
        return Some(InferredName::String(identifier.name(ctx.ast()).into()));
    }
    if let Some(private) = key
        .as_expr(ctx.ast())
        .and_then(|expr| expr.as_private_identifier(ctx.ast()))
    {
        let name = format!("#{}", ctx.ast().get_utf8(private.name(ctx.ast())));
        return Some(InferredName::String(ctx.ast.add_wtf8(name.as_str().into())));
    }
    if let Some(number) = key
        .as_expr(ctx.ast())
        .and_then(|expr| expr.as_numeric_literal(ctx.ast()))
    {
        let value = zmij_ecma::Buffer::new()
            .format(number.value(ctx.ast()))
            .to_owned();
        return Some(InferredName::String(
            ctx.ast.add_wtf8(value.as_str().into()),
        ));
    }
    if let Some(bigint) = key
        .as_expr(ctx.ast())
        .and_then(|expr| expr.as_big_int_literal(ctx.ast()))
    {
        let value = bigint.value(ctx.ast())?.to_str_radix(10);
        return Some(InferredName::String(
            ctx.ast.add_wtf8(value.as_str().into()),
        ));
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
    Computed {
        parent: NodeId,
        key: Expr,
        /// Evaluation scope of the original key, separate from the generated name read.
        key_scope: ScopeId,
    },
}

fn inferred_property_key_name(
    key: PropertyKey,
    computed: bool,
    parent: NodeId,
    key_scope: ScopeId,
    ctx: &mut TransformCtx<'_, '_>,
) -> Option<InferredName> {
    property_key_name(key, ctx).or_else(|| {
        computed
            .then(|| key.as_expr(ctx.ast()))
            .flatten()
            .map(|key| InferredName::Computed {
                parent,
                key,
                key_scope,
            })
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
    names: &mut UsedNameState<'_>,
    ctx: &mut TransformCtx<'_, '_>,
) {
    let mut index = 0;
    while let Some(member) = ctx.container_class_body_mut(body).get(index).copied() {
        if let Some(block) = member.as_static_block(ctx.ast()) {
            if block.body(ctx.ast()).is_empty() && !block_has_comments(block, ctx.ast()) {
                ctx.remove_scope(
                    ctx.semantic()
                        .scope_of(block.node_id())
                        .expect("static block scope"),
                );
                ctx.container_class_body_mut(body).remove(index);
                continue;
            }
            let key = private_name(names, ctx);
            let value = if let Some(expression) =
                single_expression_statement(block.body(ctx.ast()), ctx.ast())
            {
                inline_static_block_expression(block.node_id(), ctx.current_scope(), ctx);
                expression
            } else {
                let span = block.span(ctx.ast());
                let arrow = block_to_arrow(block, false, ctx.current_scope(), ctx);
                Expr::CallExpression(ctx.add_node(|ast| {
                    ast.call_expression(span, arrow, None, SubRange::empty(), false, false)
                }))
            };
            let field = private_field(key, value, block.span(ctx.ast()), ctx);
            ctx.container_class_body_mut(body)[index] = field;
        }
        index += 1;
    }
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

fn call_generated_after_class(name: BoundIdentifier, ctx: &mut TransformCtx<'_, '_>) -> Expr {
    let scope = ctx
        .semantic()
        .scope(ctx.current_scope())
        .parent
        .expect("class parent scope");
    let callee = name.reference_read(ctx, scope, DUMMY_SP);
    Expr::CallExpression(ctx.add_node(|ast| {
        ast.call_expression(
            DUMMY_SP,
            Expr::IdentifierReference(callee),
            None,
            SubRange::empty(),
            false,
            false,
        )
    }))
}

fn build_static_block_arrow(
    span: Span,
    body: ArrowFunctionBody,
    expression: bool,
    reuse: ScopeId,
    parent: ScopeId,
    ctx: &mut TransformCtx<'_, '_>,
) -> (Expr, ScopeId) {
    let arrow = ctx.add_node(|ast| {
        let params = ast.formal_parameters(
            DUMMY_SP,
            SubRange::empty(),
            None,
            FormalParameterKind::ArrowFormalParameters,
        );
        ast.arrow_function_expression(span, expression, false, None, params, None, body)
    });
    let original = ctx.semantic().scope(reuse);
    ctx.convert_scope(
        reuse,
        arrow.node_id(),
        ScopeKind::Function,
        ScopeFlags {
            arguments_barrier: false,
            ..original.flags
        },
    );
    if original.parent != Some(parent) {
        ctx.reparent_scope(reuse, parent);
    }
    (Expr::ArrowFunctionExpression(arrow), reuse)
}

fn inline_static_block_expression(block: NodeId, parent: ScopeId, ctx: &mut TransformCtx<'_, '_>) {
    let block = static_block(block, ctx.ast());
    let scope = ctx
        .semantic()
        .scope_of(block.node_id())
        .expect("static block scope");
    let statement = block
        .body(ctx.ast())
        .get_node(ctx.ast(), 0)
        .expect("single expression statement");
    let expression = statement
        .as_expression_statement(ctx.ast())
        .expect("expression statement")
        .expression(ctx.ast());
    ctx.move_subtree_to_scope(expression, scope, parent);
    ctx.remove_scope(scope);
    ctx.inherits_comments(expression, statement);
    ctx.inherits_comments(expression, block);
}

fn static_block(node: NodeId, ast: &Ast<'_>) -> StaticBlock {
    match ast.kind_data(node) {
        NodeKindData::StaticBlock(block) => block,
        _ => unreachable!("static block source"),
    }
}

fn block_has_comments(block: StaticBlock, ast: &Ast<'_>) -> bool {
    let span = block.span(ast);
    ast.comments().iter().any(|comment| {
        comment.attached_to == span.start
            || comment.attached_to == span.end
            || (span.start <= comment.span.start && comment.span.end <= span.end)
    })
}

#[derive(Default)]
struct ClassLayout {
    has_static_blocks: bool,
    first_static_property: Option<PropertyDefinition>,
    last_static_property: Option<PropertyDefinition>,
    trailing_blocks: usize,
}

impl ClassLayout {
    fn read(body: ClassBody, ctx: &TransformCtx<'_, '_>) -> Self {
        let mut layout = Self::default();
        for member in ctx.container_class_body(body) {
            if let Some(property) = static_property(member, ctx.ast()) {
                layout.first_static_property.get_or_insert(property);
                layout.last_static_property = Some(property);
                layout.trailing_blocks = 0;
            } else if let Some(block) = member.as_static_block(ctx.ast()) {
                layout.has_static_blocks = true;
                if !block.body(ctx.ast()).is_empty() || block_has_comments(block, ctx.ast()) {
                    layout.trailing_blocks += 1;
                }
            }
        }
        layout
    }
}

fn private_names<'a>(body: ClassBody, ctx: &TransformCtx<'_, 'a>) -> UsedNameState<'a> {
    let mut names = UsedNameState::new(ctx.ast().allocator());
    for member in ctx.container_class_body(body) {
        let key = match ctx.ast().class_element_data(member) {
            swc_next_ecma_ast::ClassElementData::PropertyDefinition(property) => {
                property.key(ctx.ast())
            }
            swc_next_ecma_ast::ClassElementData::MethodDefinition(method) => method.key(ctx.ast()),
            swc_next_ecma_ast::ClassElementData::TsMethodDefinition(property) => {
                property.key(ctx.ast())
            }
            _ => continue,
        };
        if let Some(id) = key
            .as_expr(ctx.ast())
            .and_then(|expr| expr.as_private_identifier(ctx.ast()))
        {
            names.observe(ctx.ast().get_utf8(id.name(ctx.ast())).as_bytes());
        }
    }
    names
}

fn private_name(names: &mut UsedNameState<'_>, ctx: &mut TransformCtx<'_, '_>) -> PropertyKey {
    let name = ctx.ast.add_utf8(names.next_name("_", 2));
    PropertyKey::Expr(Expr::PrivateIdentifier(
        ctx.add_node(|ast| ast.private_identifier(DUMMY_SP, name)),
    ))
}
