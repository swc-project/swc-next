//! Class static block lowering, following Babel's transform-class-static-block.
//!
//! Babel source: packages/babel-plugin-transform-class-static-block/src/index.ts.
//! The named-evaluation helper comes from
//! packages/babel-helper-create-class-features-plugin/src/decorators.ts.
//!
//! BabelApiGaps records only operations used by those sources, with the original
//! call next to every TODO. These placeholders do not implement framework APIs.
//! Their Rust signatures carry explicit node IDs/scopes because the current
//! architecture has no Babel NodePath/Scope objects. Mutation must synchronize
//! AST edges, moved scope owners, bindings/references and comments in each edit.
//!
//! Existing framework blockers remain: ScopeModel registration, statement
//! injection and script helper require are TODOs; add_node supplies grow_to with
//! the pre-allocation count; register_reference returns SymbolId rather than
//! ReferenceId; callers still pass Semantic where ScopeModel is expected.
//! pushContainer/unshiftContainer defer only member-list attachment. Registering
//! the synthetic named-evaluation StaticBlock scope is still a framework gap.
//! Babel templates are constructed directly with AST builders. Pending semantic
//! work: register generated arrow scopes, move reused bodies/expressions into
//! those scopes, and register assignment targets as write references.
use rustc_hash::FxHashMap;
use swc_next_ecma_ast::{
    Accessibility, ArrowFunctionBody, AssignmentOperator, AssignmentTarget, Ast, AstWtf8, Class,
    ClassBody, ClassElement, ClassElementData, ClassType, DUMMY_SP, Expr, FormalParameterKind,
    GetSpan, NodeId, NodeKindData, PropertyDefinition, PropertyKey, ScopeId,
    SimpleAssignmentTarget, Span, StaticBlock, Stmt, SubRange, UnaryOperator,
};

use crate::{
    TransformFeatureFlags,
    context::{BoundIdentifier, ContainerOwnerChild, Mutation},
    used_names::UsedNameState,
};

pub(crate) struct StaticBlocks {
    prevent_extensions: bool,
    // Babel replaces/inserts after the class within ClassBody. The generic
    // traversal does that in exit_class, retaining only the generated tmp here.
    closures: FxHashMap<NodeId, BoundIdentifier>,
}

impl StaticBlocks {
    pub(crate) fn new(features: TransformFeatureFlags) -> Self {
        Self {
            prevent_extensions: features
                .contains(TransformFeatureFlags::PROPOSAL_PREVENT_EXTENSIONS),
            closures: FxHashMap::default(),
        }
    }

    pub(crate) fn enter_class_body<'a>(
        &mut self,
        class_body: ClassBody,
        ctx: &mut Mutation<'_, '_, 'a>,
    ) {
        if !ctx
            .container_class_body(class_body)
            .any(|el| el.as_static_block(ctx.ast).is_some())
        {
            return;
        }
        let parent_path = ctx
            .parent()
            .and_then(|node| match ctx.ast.kind_data(node) {
                NodeKindData::Class(class) => Some(class),
                _ => None,
            })
            .expect("class body parent");
        let scope = ctx.semantic().scope_at(class_body.node_id()).unwrap();

        // The repository's ordinary ES2022 mode retains private-field lowering;
        // prevent_extensions selects the Babel algorithm below.
        if !self.prevent_extensions {
            let mut private_names = Self::private_names(ctx.container_class_body(class_body), ctx);
            // Snapshot member handles before replacing their attachments.
            let elements: Vec<_> = ctx.container_class_body(class_body).collect();
            for path in elements {
                if let Some(block) = path.as_static_block(ctx.ast) {
                    let value = Self::block_to_expression(block, scope, ctx);
                    let static_block_private_id = Self::generate_uid(&mut private_names);
                    let field = Self::private_field(
                        scope,
                        static_block_private_id,
                        value,
                        block.span(ctx.ast),
                        ctx,
                    );
                    ctx.replace_with(path.node_id(), None, field.node_id());
                }
            }
            return;
        }

        // Run on ClassBody and not on class so that if @babel/helper-create-class-features-plugin
        // is enabled it can generate optimized output without passing from the intermediate
        // private fields representation.
        // If needed, add the name to the class
        Self::named_evaluation_visitor(class_body, parent_path, ctx);

        let mut pending_static_blocks = Vec::new();
        let mut last_static_prop = None;
        // Borrow the member list briefly so lowering can also borrow ctx.
        let mut index = 0;
        while let Some(path) = ctx.container_class_body_mut(class_body).get(index).copied() {
            if let Some(block) = path.as_static_block(ctx.ast) {
                pending_static_blocks.push(block);
                // Babel: pendingStaticBlocks.push(path.node); path.remove().
                // The arena keeps the detached block available for reuse.
                // TODO: synchronize its scope owner when moving its contents.
                ctx.container_class_body_mut(class_body).remove(index);
                continue;
            }
            index += 1;
            if let Some(prop) = path
                .as_property_definition(ctx.ast)
                .filter(|prop| prop.r#static(ctx.ast))
            {
                last_static_prop = Some(prop);
                if !pending_static_blocks.is_empty() {
                    // push static blocks right before the initializer of the next
                    // static field in the class.
                    let expressions =
                        Self::blocks_to_expressions(&pending_static_blocks, scope, ctx);
                    Self::prepend_to_initializer(prop, expressions, ctx);
                    pending_static_blocks.clear();
                }
            }
        }

        if !pending_static_blocks.is_empty() {
            // if there are static blocks not followed by a static field where
            // we can inject them, wrap them in a function and push it in the
            // last static field in the class (or create a new one, if needed).
            // After the class body runs, call the function to run the remaining
            // static blocks bodies.
            let tmp = ctx.generate_declared_uid_identifier(scope, "staticBlock");
            let needs_completion_value =
                parent_path.class_type(ctx.ast) == ClassType::ClassExpression;
            let arrow = if pending_static_blocks.len() > 1
                || (pending_static_blocks[0].body(ctx.ast).len() == 1
                    && ctx
                        .ast
                        .get_node_in_sub_range(
                            pending_static_blocks[0].body(ctx.ast).first().unwrap(),
                        )
                        .as_expression_statement(ctx.ast)
                        .is_some())
            {
                let mut expressions =
                    Self::blocks_to_expressions(&pending_static_blocks, scope, ctx);
                if needs_completion_value {
                    expressions.push(Self::this_expression(scope, ctx));
                }
                let arrow_body = Self::maybe_sequence_expression(scope, expressions, ctx);
                // template.expression.ast`${tmp} = () => ${arrowBody}`:
                // build the expression-bodied arrow before the assignment.
                ctx.add_node(scope, |ast| {
                    Self::arrow_function_expression(
                        ast,
                        ArrowFunctionBody::Expr(arrow_body),
                        DUMMY_SP,
                    )
                })
            } else {
                Self::block_to_arrow(pending_static_blocks[0], scope, needs_completion_value, ctx)
            };
            let init = Self::assignment(scope, tmp, arrow, ctx);
            if let Some(prop) = last_static_prop {
                Self::prepend_to_initializer(prop, vec![init], ctx);
            } else {
                // If there are no static fields at all, it's safe to inject a
                // new private properties before running the static blocks because
                // there is no code that could have already made the class
                // non-extensible.
                let mut private_names =
                    Self::private_names(ctx.container_class_body(class_body), ctx);
                let static_block_private_id = Self::generate_uid(&mut private_names);
                let field =
                    Self::private_field(scope, static_block_private_id, init, DUMMY_SP, ctx);
                ctx.push_container(ContainerOwnerChild::ClassBody((class_body, field)));
            }
            self.closures.insert(parent_path.node_id(), tmp);
        }
    }

    /// Predicate passed to Babel's buildNamedEvaluationVisitor.
    fn needs_name(class_path: Class, ctx: &Mutation<'_, '_, '_>) -> bool {
        if class_path.class_type(ctx.ast) != ClassType::ClassExpression {
            return false;
        }
        for el in ctx.container_class_body(class_path.body(ctx.ast)).rev() {
            if el.as_static_block(ctx.ast).is_some() {
                return true;
            }
            if el
                .as_property_definition(ctx.ast)
                .is_some_and(|prop| prop.r#static(ctx.ast))
            {
                break;
            }
        }
        false
    }

    fn named_evaluation_visitor(
        class_body: ClassBody,
        class_path: Class,
        ctx: &mut Mutation<'_, '_, '_>,
    ) {
        if class_path.id(ctx.ast).is_some() || !Self::needs_name(class_path, ctx) {
            return;
        }
        let Some(name_node) = Self::named_evaluation_name(class_path, ctx) else {
            return;
        };
        let scope = ctx.semantic().scope_at(class_body.node_id()).unwrap();
        let target = Self::this_expression(scope, ctx);
        let expression = ctx.call_set_function_name(target, name_node);
        let statement = Self::expression_statement(scope, expression, ctx);
        let body = ctx.ast.add_sub_range([statement]);
        let block = ctx.add_node(scope, |ast| ast.static_block(DUMMY_SP, body));
        // classPath.get("body").unshiftContainer("body", t.staticBlock([...]))
        ctx.unshift_container(ContainerOwnerChild::ClassBody((
            class_body,
            ClassElement::StaticBlock(block),
        )));
    }

    pub(crate) fn exit_class(&mut self, class: Class, ctx: &mut Mutation<'_, '_, '_>) {
        let Some(tmp) = self.closures.remove(&class.node_id()) else {
            return;
        };
        let scope = ctx.semantic().scope_outside(class.node_id()).unwrap();
        // t.cloneNode(tmp): BoundIdentifier::reference supplies a fresh read
        // reference to the same symbol, without cloning a binding declaration.
        let callee = Expr::IdentifierReference(tmp.reference(ctx, scope, DUMMY_SP));
        let static_block_closure_call = Self::call(scope, callee, DUMMY_SP, ctx);
        if class.class_type(ctx.ast) == ClassType::ClassExpression {
            // We don't use .insertAfter() because we don't need to insert the
            // tmp variable to preserve the class as the result value, because
            // the call will already return the class itself.
            let expressions = vec![Expr::Class(class), static_block_closure_call];
            let replacement = Self::maybe_sequence_expression(scope, expressions, ctx);
            ctx.replace_with(class.node_id(), None, replacement.node_id());
        } else {
            let statement = Self::expression_statement(scope, static_block_closure_call, ctx);
            ctx.insert_after(class.node_id(), statement);
        }
    }

    fn maybe_sequence_expression(
        scope: ScopeId,
        expressions: Vec<Expr>,
        ctx: &mut Mutation<'_, '_, '_>,
    ) -> Expr {
        assert!(!expressions.is_empty());
        if expressions.len() == 1 {
            return expressions[0];
        }
        let expressions = ctx.ast.add_sub_range(expressions);
        Expr::SequenceExpression(
            ctx.add_node(scope, |ast| ast.sequence_expression(DUMMY_SP, expressions)),
        )
    }

    fn this_expression(scope: ScopeId, ctx: &mut Mutation<'_, '_, '_>) -> Expr {
        Expr::ThisExpression(ctx.add_node(scope, |ast| ast.this_expression(DUMMY_SP)))
    }

    fn name_literal(scope: ScopeId, name: &str, ctx: &mut Mutation<'_, '_, '_>) -> Expr {
        let value = ctx.ast.add_wtf8(name.into());
        Self::string_value(scope, value, ctx)
    }

    fn call(scope: ScopeId, callee: Expr, span: Span, ctx: &mut Mutation<'_, '_, '_>) -> Expr {
        Expr::CallExpression(ctx.add_node(scope, |ast| {
            ast.call_expression(span, callee, None, SubRange::empty(), false)
        }))
    }

    fn expression_statement(scope: ScopeId, value: Expr, ctx: &mut Mutation<'_, '_, '_>) -> Stmt {
        Stmt::ExpressionStatement(
            ctx.add_node(scope, |ast| ast.expression_statement(DUMMY_SP, value)),
        )
    }

    /// The assignment in Babel's template.expression.ast`${tmp} = () => ${arrowBody}`,
    /// also used for the computed-key assignment in the named-evaluation helper.
    fn assignment(
        scope: ScopeId,
        tmp: BoundIdentifier,
        value: Expr,
        ctx: &mut Mutation<'_, '_, '_>,
    ) -> Expr {
        let name = tmp.identifier.name(ctx.ast);
        // TODO(Semantic): register the assignment target against tmp.symbol as
        // a write reference. BoundIdentifier::reference currently registers reads.
        ctx.add_node(scope, |ast| {
            let reference = ast.identifier_reference(DUMMY_SP, name);
            let left = AssignmentTarget::SimpleAssignmentTarget(
                SimpleAssignmentTarget::IdentifierReference(reference),
            );
            Expr::AssignmentExpression(ast.assignment_expression(
                DUMMY_SP,
                left,
                value,
                AssignmentOperator::Assign,
            ))
        })
    }

    fn map_last<T>(mut arr: Vec<T>, f: impl FnOnce(T) -> T) -> Vec<T> {
        if let Some(value) = arr.pop() {
            arr.push(f(value));
        }
        arr
    }

    fn prepend_to_initializer(
        prop: PropertyDefinition,
        mut expressions: Vec<Expr>,
        ctx: &mut Mutation<'_, '_, '_>,
    ) {
        let scope = ctx
            .semantic()
            .scope_at(prop.node_id())
            .expect("field scope");
        let expressions = if let Some(value) = prop.value(ctx.ast) {
            expressions.push(value);
            expressions
        } else {
            Self::map_last(expressions, |expr| {
                Expr::UnaryExpression(ctx.add_node(scope, |ast| {
                    ast.unary_expression(DUMMY_SP, expr, UnaryOperator::Void)
                }))
            })
        };
        let value = Self::maybe_sequence_expression(scope, expressions, ctx);
        // Babel assigns prop.value directly. The current architecture requires
        // the synchronized equivalent of propPath.get("value").replaceWith(value).
        ctx.replace_with(prop.node_id(), Some("value"), value.node_id());
    }

    fn blocks_to_expressions(
        blocks: &[StaticBlock],
        scope: ScopeId,
        ctx: &mut Mutation<'_, '_, '_>,
    ) -> Vec<Expr> {
        blocks
            .iter()
            .map(|&block| Self::block_to_expression(block, scope, ctx))
            .collect()
    }

    /// The map callback in Babel's blocksToExpressions, extracted for reuse by
    /// the repository's ordinary private-field lowering mode.
    fn block_to_expression(
        block: StaticBlock,
        scope: ScopeId,
        ctx: &mut Mutation<'_, '_, '_>,
    ) -> Expr {
        let body = block.body(ctx.ast);
        if body.len() == 1 {
            let statement = ctx.ast.get_node_in_sub_range(body.first().unwrap());
            if let Some(statement) = statement.as_expression_statement(ctx.ast) {
                // We special-case the single expression case to avoid the iife, since
                // it's common.
                let expression = statement.expression(ctx.ast);
                return expression;
            }
        }
        let arrow = Self::block_to_arrow(block, scope, false, ctx);
        Self::call(scope, arrow, block.span(ctx.ast), ctx)
    }

    /// The block-bodied arrow in template.expression.ast`(() => { ${body} })()`.
    /// Also serves template.expression.ast`${tmp} = () => ${arrowBody}` when
    /// arrowBody is a blockStatement, appending return this when needed.
    fn block_to_arrow(
        block: StaticBlock,
        scope: ScopeId,
        needs_completion_value: bool,
        ctx: &mut Mutation<'_, '_, '_>,
    ) -> Expr {
        let span = block.span(ctx.ast);
        let body = block.body(ctx.ast);
        // TODO(Semantic): transfer the reused statements from the static-block
        // scope to the generated arrow scope, preserving bindings/references.
        ctx.add_node(scope, |ast| {
            let mut statements: Vec<_> = body
                .iter()
                .map(|slot| ast.get_node_in_sub_range(slot))
                .collect();
            if needs_completion_value {
                let value = Expr::ThisExpression(ast.this_expression(DUMMY_SP));
                statements.push(Stmt::ReturnStatement(
                    ast.return_statement(DUMMY_SP, Some(value)),
                ));
            }
            let statements = ast.add_sub_range(statements);
            let arrow_body = ast.function_body(span, SubRange::empty(), statements);
            Self::arrow_function_expression(ast, ArrowFunctionBody::FunctionBody(arrow_body), span)
        })
    }

    /// Construct the arrow in Babel's expression templates directly.
    fn arrow_function_expression(
        ast: &mut Ast<'_>,
        arrow_body: ArrowFunctionBody,
        span: Span,
    ) -> Expr {
        // TODO(Semantic): register the arrow's function scope and assign its
        // parameters/body to that scope. add_node only records allocation scope.
        let expression = arrow_body.as_expr(ast).is_some();
        let params = ast.formal_parameters(
            DUMMY_SP,
            SubRange::empty(),
            None,
            FormalParameterKind::ArrowFormalParameters,
        );
        Expr::ArrowFunctionExpression(
            ast.arrow_function_expression(span, expression, false, None, params, None, arrow_body),
        )
    }

    /// t.classPrivateProperty(staticBlockRef, init, [], true)
    fn private_field(
        scope: ScopeId,
        name: &str,
        value: Expr,
        span: Span,
        ctx: &mut Mutation<'_, '_, '_>,
    ) -> ClassElement {
        let name = ctx.ast.add_utf8(name);
        ctx.add_node(scope, |ast| {
            let static_block_ref =
                PropertyKey::Expr(Expr::PrivateIdentifier(ast.private_identifier(span, name)));
            ClassElement::PropertyDefinition(ast.property_definition(
                span,
                SubRange::empty(),
                static_block_ref,
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
            ))
        })
    }

    /// Generate a uid that is not in `deny_list`.
    fn generate_uid<'n>(deny_list: &'n mut UsedNameState<'_>) -> &'n str {
        deny_list.next_name("_", 2)
    }

    /// The privateNames collection loop before generateUid(privateNames).
    fn private_names<'a>(
        elements: impl IntoIterator<Item = ClassElement>,
        ctx: &Mutation<'_, '_, 'a>,
    ) -> UsedNameState<'a> {
        let mut names = UsedNameState::new(ctx.ast.allocator());
        for path in elements {
            let key = match ctx.ast.class_element_data(path) {
                ClassElementData::PropertyDefinition(prop) => Some(prop.key(ctx.ast)),
                ClassElementData::MethodDefinition(method) => Some(method.key(ctx.ast)),
                ClassElementData::TsMethodDefinition(method) => Some(method.key(ctx.ast)),
                _ => None,
            };
            if let Some(key) = key.and_then(|key| {
                key.as_expr(ctx.ast)
                    .and_then(|expr| expr.as_private_identifier(ctx.ast))
            }) {
                names.observe(ctx.ast.get_utf8(key.name(ctx.ast)).as_bytes());
            }
        }
        names
    }

    /// Equivalent name selection for Babel's imported buildNamedEvaluationVisitor,
    /// implemented in babel-helper-create-class-features-plugin/src/decorators.ts.
    /// This helper adapts its initializer-parent visitor dispatch to the current
    /// traversal's ancestor stack, including transparent TypeScript wrappers.
    /// Property names and computed-key memoization are handled by property_name.
    fn named_evaluation_name(class: Class, ctx: &mut Mutation<'_, '_, '_>) -> Option<Expr> {
        let scope = ctx
            .semantic()
            .scope_at(class.node_id())
            .expect("class scope");
        let (parent, initializer) = {
            let mut initializer = class.node_id();
            let mut ancestors = ctx.ancestors().skip(2);
            loop {
                let node = ancestors.next()?;
                if matches!(
                    ctx.ast.kind_data(node),
                    NodeKindData::TsAsExpression(_)
                        | NodeKindData::TsSatisfiesExpression(_)
                        | NodeKindData::TsNonNullExpression(_)
                        | NodeKindData::TsTypeAssertion(_)
                ) {
                    initializer = node;
                } else {
                    break (node, initializer);
                }
            }
        };
        let identifier = match ctx.ast.kind_data(parent) {
            NodeKindData::VariableDeclarator(declarator)
                if declarator
                    .init(ctx.ast)
                    .is_some_and(|value| value.node_id() == initializer) =>
            {
                declarator
                    .id(ctx.ast)
                    .as_binding_identifier(ctx.ast)
                    .map(|id| id.name(ctx.ast))
            }
            NodeKindData::AssignmentPattern(pattern)
                if pattern.right(ctx.ast).node_id() == initializer =>
            {
                pattern
                    .left(ctx.ast)
                    .as_binding_identifier(ctx.ast)
                    .map(|id| id.name(ctx.ast))
            }
            NodeKindData::AssignmentExpression(assignment)
                if assignment.right(ctx.ast).node_id() == initializer
                    && matches!(
                        assignment.operator(ctx.ast),
                        AssignmentOperator::Assign
                            | AssignmentOperator::LogicalAndAssign
                            | AssignmentOperator::LogicalOrAssign
                            | AssignmentOperator::NullishAssign
                    ) =>
            {
                assignment
                    .left(ctx.ast)
                    .as_simple_assignment_target(ctx.ast)
                    .and_then(|left| left.as_identifier_reference(ctx.ast))
                    .map(|id| id.name(ctx.ast))
            }
            NodeKindData::PropertyDefinition(property)
                if property
                    .value(ctx.ast)
                    .is_some_and(|value| value.node_id() == initializer) =>
            {
                return Self::property_name(
                    parent,
                    property.key(ctx.ast),
                    property.computed(ctx.ast),
                    scope,
                    ctx,
                );
            }
            NodeKindData::ObjectProperty(property)
                if property.value(ctx.ast).node_id() == initializer =>
            {
                let key = property.key(ctx.ast);
                // We listen on ObjectExpression so that we don't have to visit
                // the object properties under object patterns. ObjectPattern has
                // its own node kinds in this AST.
                // 13.2.5.5 RS: PropertyDefinitionEvaluation
                let proto = !property.computed(ctx.ast)
                    && !property.shorthand(ctx.ast)
                    && match ctx.ast.kind_data(key.node_id()) {
                        NodeKindData::IdentifierName(id) => {
                            ctx.ast.get_utf8(id.name(ctx.ast)) == "__proto__"
                        }
                        NodeKindData::StringLiteral(string) => {
                            ctx.ast.get_wtf8(string.value(ctx.ast)).as_bytes() == b"__proto__"
                        }
                        _ => false,
                    };
                if proto {
                    return None;
                }
                return Self::property_name(parent, key, property.computed(ctx.ast), scope, ctx);
            }
            _ => None,
        }?;
        Some(Self::name_literal(scope, ctx.ast.get_utf8(identifier), ctx))
    }

    /// Adapts buildNamedEvaluationVisitor's property visitors and
    /// handleComputedProperty in babel-helper-create-class-features-plugin/src/decorators.ts.
    fn property_name(
        owner: NodeId,
        key: PropertyKey,
        computed: bool,
        scope: ScopeId,
        ctx: &mut Mutation<'_, '_, '_>,
    ) -> Option<Expr> {
        match ctx.ast.kind_data(key.node_id()) {
            NodeKindData::IdentifierName(id) => Some(Self::name_literal(
                scope,
                ctx.ast.get_utf8(id.name(ctx.ast)),
                ctx,
            )),
            NodeKindData::PrivateIdentifier(id) => Some(Self::name_literal(
                scope,
                &format!("#{}", ctx.ast.get_utf8(id.name(ctx.ast))),
                ctx,
            )),
            NodeKindData::StringLiteral(string) => {
                let value = string.value(ctx.ast);
                Some(Self::string_value(scope, value, ctx))
            }
            NodeKindData::NumericLiteral(_) | NodeKindData::BigIntLiteral(_) => {
                // Babel's literal.value + "" uses ECMAScript number/BigInt
                // conversion, not Rust's Display or the raw source spelling.
                let value = Self::literal_value_to_string(key, ctx);
                if computed {
                    let key_scope = ctx.semantic().scope_at(key.node_id()).expect("key scope");
                    let replacement = Self::string_value(key_scope, value, ctx);
                    ctx.replace_with(owner, Some("key"), replacement.node_id());
                }
                Some(Self::string_value(scope, value, ctx))
            }
            _ if computed => {
                // Evaluate and convert the key once. The memoized property key
                // also names the anonymous class, including Symbol keys.
                let expression = key.as_expr(ctx.ast).expect("computed key expression");
                let key_scope = ctx
                    .semantic()
                    .scope_outside(expression.node_id())
                    .expect("key scope");
                // Babel's handleComputedProperty assumes a non-null result:
                // const ref = propertyPath.scope.maybeGenerateMemoised(key)!;
                let temp = ctx
                    .maybe_generate_memoised(key_scope, expression)
                    .expect("Babel handleComputedProperty requires a memoised key");
                let converted = ctx.call_to_property_key(expression);
                let assignment = Self::assignment(key_scope, temp, converted, ctx);
                ctx.replace_with(owner, Some("key"), assignment.node_id());
                Some(Expr::IdentifierReference(
                    temp.reference(ctx, scope, DUMMY_SP),
                ))
            }
            _ => None,
        }
    }

    fn string_value(scope: ScopeId, value: AstWtf8, ctx: &mut Mutation<'_, '_, '_>) -> Expr {
        Expr::StringLiteral(ctx.add_node(scope, |ast| {
            ast.string_literal(DUMMY_SP, value, ast.empty_utf8())
        }))
    }
    /// Equivalent to the JavaScript expression `key.value + ""` in Babel's
    /// handleComputedProperty and property visitors. This is a language-value
    /// conversion gap, not a Babel Scope or NodePath API. Numeric literals use
    /// ECMAScript Number formatting; BigInt raw radix/separators need decoding.
    fn literal_value_to_string(_key: PropertyKey, _ctx: &mut Mutation<'_, '_, '_>) -> AstWtf8 {
        todo!("Babel key.value string coercion: canonical Number/BigInt conversion")
    }
}

/// TODOs for Babel operations used by this transform. All source calls are in
/// babel-plugin-transform-class-static-block/src/index.ts unless noted otherwise.
/// Explicit node IDs/scopes replace Babel's NodePath/Scope receivers; they are
/// signature adaptations, not additional transform-specific framework operations.
#[allow(unused_variables)]
trait BabelApiGaps {
    /// Babel: classBody.parentPath.replaceWith(t.sequenceExpression([classBody.parent, call])),
    /// propertyPath.get("key").replaceWith(...) in decorators.ts, and prop.value = ... .
    /// field=None means path.replaceWith; Some(field) means path.get(field).replaceWith.
    /// Update the attachment edge, retaining the original node when wrapped.
    /// Class-element replacements must also update any logical member list.
    fn replace_with(&mut self, path: NodeId, field: Option<&str>, replacement: NodeId) {
        todo!("Babel NodePath.replaceWith: replace an attachment with registered syntax")
    }
}

impl BabelApiGaps for Mutation<'_, '_, '_> {}
