use crate::context::Mutation;
use swc_next_ecma_ast::{
    Ast, AstUtf8, BindingPattern, DUMMY_SP, Decl, Expr, GetSpan, IdentifierReference, MetaProperty,
    NodeId, NodeKindData, ScopeId, Stmt, ThisExpression, UnaryOperator, VariableDeclaration,
    VariableKind,
};
use swc_next_ecma_semantic::{ReferenceSpace, Semantic};

#[derive(Default)]
pub struct CaptureFrame {
    pub(crate) this: Option<AstUtf8>,
    pub(crate) arguments: Option<AstUtf8>,
    pub(crate) new_target: Option<AstUtf8>,
    pub(crate) deferred_this: Vec<AstUtf8>,
}

impl CaptureFrame {
    pub(crate) fn is_empty(&self) -> bool {
        self.this.is_none()
            && self.arguments.is_none()
            && self.new_target.is_none()
            && self.deferred_this.is_empty()
    }
}

/// Hoists the lexical function environment used by converted arrow functions.
///
/// This is the flat-AST equivalent of `swc_ecma_utils::function::FnEnvHoister`.
/// It owns capture state, while callers retain ownership of AST traversal.
pub struct FnEnvHoister {
    frames: Vec<CaptureFrame>,
    in_arrow: bool,
}

impl FnEnvHoister {
    pub fn new() -> Self {
        Self {
            frames: vec![CaptureFrame::default()],
            in_arrow: false,
        }
    }

    pub fn set_in_arrow(&mut self, in_arrow: bool) -> bool {
        std::mem::replace(&mut self.in_arrow, in_arrow)
    }

    pub fn push_frame(&mut self) {
        self.frames.push(CaptureFrame::default());
    }

    pub fn push_property_frame(&mut self) {
        let parent = self.frames.last().expect("a parent capture frame exists");
        self.frames.push(CaptureFrame {
            arguments: parent.arguments,
            new_target: parent.new_target,
            ..CaptureFrame::default()
        });
    }

    pub fn take_frame(&mut self) -> CaptureFrame {
        self.frames
            .pop()
            .expect("a function environment frame exists")
    }

    pub(crate) fn capture_this(&mut self, ctx: &mut Mutation<'_, '_, '_>) -> AstUtf8 {
        if let Some(name) = self.frames.last().and_then(|frame| frame.this) {
            return name;
        }
        let name = generate_capture_name("_this", ctx);
        self.frames.last_mut().expect("a capture frame exists").this = Some(name);
        name
    }

    fn capture_arguments(&mut self, ctx: &mut Mutation<'_, '_, '_>) -> AstUtf8 {
        if let Some(name) = self.frames.last().and_then(|frame| frame.arguments) {
            return name;
        }
        let name = generate_capture_name("_arguments", ctx);
        self.frames
            .last_mut()
            .expect("a capture frame exists")
            .arguments = Some(name);
        name
    }

    fn capture_new_target(&mut self, ctx: &mut Mutation<'_, '_, '_>) -> AstUtf8 {
        if let Some(name) = self.frames.last().and_then(|frame| frame.new_target) {
            return name;
        }
        let name = generate_capture_name("_newtarget", ctx);
        self.frames
            .last_mut()
            .expect("a capture frame exists")
            .new_target = Some(name);
        name
    }

    fn is_unresolved_arguments(
        &self,
        node: IdentifierReference,
        ast: &Ast<'_>,
        semantic: &Semantic<'_>,
    ) -> bool {
        if ast.get_utf8(node.name(ast)) != "arguments" {
            return false;
        }
        let Some(reference_id) = semantic.reference_of(node.node_id()) else {
            return false;
        };
        let reference = semantic.reference(reference_id);
        reference.flags.space == ReferenceSpace::Value && reference.symbol.is_none()
    }

    fn is_new_target(&self, node: MetaProperty, ast: &Ast<'_>) -> bool {
        let meta = node.meta(ast);
        let property = node.property(ast);
        ast.get_utf8(meta.name(ast)) == "new" && ast.get_utf8(property.name(ast)) == "target"
    }

    pub fn transform_this_expression(
        &mut self,
        node: ThisExpression,
        ctx: &mut Mutation<'_, '_, '_>,
    ) -> Option<Expr> {
        if !self.in_arrow {
            return None;
        }
        let name = self.capture_this(ctx);
        let replacement = ctx.generated_reference(name, node.span(ctx.ast()));
        Some(Expr::IdentifierReference(replacement))
    }

    pub fn visit_identifier_reference(
        &mut self,
        node: IdentifierReference,
        ctx: &mut Mutation<'_, '_, '_>,
    ) {
        if self.in_arrow && self.is_unresolved_arguments(node, ctx.ast(), ctx.semantic()) {
            let name = self.capture_arguments(ctx);
            ctx.retarget_generated_reference(node, name);
        }
    }

    pub fn transform_meta_property(
        &mut self,
        node: MetaProperty,
        new_target_is_undefined: bool,
        ctx: &mut Mutation<'_, '_, '_>,
    ) -> Option<Expr> {
        if self.in_arrow && self.is_new_target(node, ctx.ast()) {
            if new_target_is_undefined {
                let span = node.span(ctx.ast());
                let zero = Expr::NumericLiteral(ctx.add_node(ctx.current_scope(), |ast| {
                    ast.numeric_literal(DUMMY_SP, 0.0, None)
                }));
                return Some(Expr::UnaryExpression(
                    ctx.add_node(ctx.current_scope(), |ast| {
                        ast.unary_expression(span, zero, UnaryOperator::Void)
                    }),
                ));
            }
            let name = self.capture_new_target(ctx);
            let replacement = ctx.generated_reference(name, node.span(ctx.ast()));
            Some(Expr::IdentifierReference(replacement))
        } else {
            None
        }
    }

    fn build_capture_declaration(
        &mut self,
        frame: CaptureFrame,
        scope: ScopeId,
        derived_constructor: bool,
        ctx: &mut Mutation<'_, '_, '_>,
    ) -> Option<VariableDeclaration> {
        if frame.is_empty() {
            return None;
        }

        let mut declarators = Vec::with_capacity(3 + frame.deferred_this.len());
        for name in frame.deferred_this {
            let id = BindingPattern::BindingIdentifier(ctx.generated_declaration(name));
            declarators.push(ctx.add_node(scope, |ast| {
                ast.variable_declarator(DUMMY_SP, id, None, None, false)
            }));
        }
        if let Some(name) = frame.this {
            let id = BindingPattern::BindingIdentifier(ctx.generated_declaration(name));
            let init = (!derived_constructor).then(|| {
                Expr::ThisExpression(ctx.add_node(scope, |ast| ast.this_expression(DUMMY_SP)))
            });
            declarators.push(ctx.add_node(scope, |ast| {
                ast.variable_declarator(DUMMY_SP, id, None, init, false)
            }));
        }
        if let Some(name) = frame.arguments {
            let id = BindingPattern::BindingIdentifier(ctx.generated_declaration(name));
            let init = Expr::IdentifierReference(ctx.create_unbound_reference("arguments", scope));
            declarators.push(ctx.add_node(scope, |ast| {
                ast.variable_declarator(DUMMY_SP, id, None, Some(init), false)
            }));
        }
        if let Some(name) = frame.new_target {
            let id = BindingPattern::BindingIdentifier(ctx.generated_declaration(name));
            let new_name = ctx.intern_utf8("new");
            let target_name = ctx.intern_utf8("target");
            let meta = ctx.add_node(scope, |ast| ast.identifier_name(DUMMY_SP, new_name));
            let property = ctx.add_node(scope, |ast| ast.identifier_name(DUMMY_SP, target_name));
            let init = Expr::MetaProperty(
                ctx.add_node(scope, |ast| ast.meta_property(DUMMY_SP, meta, property)),
            );
            declarators.push(ctx.add_node(scope, |ast| {
                ast.variable_declarator(DUMMY_SP, id, None, Some(init), false)
            }));
        }

        let declarators = ctx.add_sub_range(declarators);
        Some(ctx.add_node(scope, |ast| {
            ast.variable_declaration(DUMMY_SP, VariableKind::Var, declarators, false)
        }))
    }

    pub(crate) fn enqueue_captures(
        &mut self,
        container: NodeId,
        frame: CaptureFrame,
        derived_constructor: bool,
        ctx: &mut Mutation<'_, '_, '_>,
    ) {
        let scope = ctx
            .semantic()
            .scope_at(container)
            .expect("capture container is registered");
        let Some(declaration) =
            self.build_capture_declaration(frame, scope, derived_constructor, ctx)
        else {
            return;
        };
        let statement = Stmt::Declaration(Decl::VariableDeclaration(declaration));
        ctx.inject_statement(container, statement);
    }

    pub fn wrap_generated_arrow(
        &mut self,
        value: Expr,
        frame: CaptureFrame,
        ctx: &mut Mutation<'_, '_, '_>,
    ) -> Expr {
        let this_name = frame.this;
        let parent = self
            .frames
            .last_mut()
            .expect("a parent capture frame exists");
        if parent.arguments.is_none() {
            parent.arguments = frame.arguments;
        }
        if parent.new_target.is_none() {
            parent.new_target = frame.new_target;
        }
        parent.deferred_this.extend(frame.deferred_this);
        let Some(this_name) = this_name else {
            return value;
        };
        parent.deferred_this.push(this_name);
        let scope = ctx
            .semantic()
            .scope_outside(value.node_id())
            .expect("wrapped arrow has an outer scope");
        let this = Expr::ThisExpression(ctx.add_node(scope, |ast| ast.this_expression(DUMMY_SP)));
        let assignment = ctx.assign_generated(this_name, this, DUMMY_SP, scope);
        let items = ctx.add_sub_range([assignment, value]);
        Expr::SequenceExpression(
            ctx.add_node(scope, |ast| ast.sequence_expression(value.span(ast), items)),
        )
    }
}

fn generate_capture_name(base: &str, ctx: &mut Mutation<'_, '_, '_>) -> AstUtf8 {
    use NodeKindData;
    let owner = ctx
        .ancestors()
        .find(|node| {
            matches!(
                ctx.ast().kind_data(*node),
                NodeKindData::Function(_) | NodeKindData::StaticBlock(_) | NodeKindData::Program(_)
            )
        })
        .expect("capture has a function environment");
    let body = match ctx.ast().kind_data(owner) {
        NodeKindData::Function(function) => Some(function.body(ctx.ast()).node_id()),
        _ => None,
    };
    let scope = ctx
        .semantic()
        .scope_at(body.unwrap_or(owner))
        .expect("capture owner has a scope");
    let binding = ctx.generate_binding(
        base,
        scope,
        swc_next_ecma_semantic::SymbolFlags::FUNCTION_SCOPED_VAR,
    );
    binding.declaration.name(ctx.ast())
}
