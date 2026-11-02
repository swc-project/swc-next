use crate::{
    Accessibility, AssignmentOperator, Ast, AstNode, AstUtf8, AstWtf8, BinaryOperator, ClassType,
    FormalParameterKind, FunctionType, ImportOrExportKind, ImportPhase, LogicalOperator,
    MethodDefinitionKind, NodeId, PropertyKind, SourceType, Span, TsMappedTypeModifier,
    TsMethodSignatureKind, TsModuleDeclarationKind, TsTypeOperatorKind, UnaryOperator,
    UpdateOperator, VariableKind, Visit, VisitMut, VisitMutWith, VisitWith,
};

/// Controls the current node's enter or exit phase.
///
/// `Skip` ends the remaining typed hooks for this visit; in enter it also skips
/// children and exit. `Stop` additionally stops the enclosing traversal.
/// Replacements end the old node's visit and are attached by the parent before
/// optionally visiting the new node from enter.
#[must_use]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TraverseAction<T> {
    #[default]
    Continue,
    Skip,
    Stop,
    ReplaceContinue(T),
    ReplaceSkip(T),
    ReplaceStop(T),
}

impl<T> TraverseAction<T> {
    /// Lift a replacement into its containing field or variant type.
    #[inline]
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> TraverseAction<U> {
        match self {
            Self::Continue => TraverseAction::Continue,
            Self::Skip => TraverseAction::Skip,
            Self::Stop => TraverseAction::Stop,
            Self::ReplaceContinue(node) => TraverseAction::ReplaceContinue(f(node)),
            Self::ReplaceSkip(node) => TraverseAction::ReplaceSkip(f(node)),
            Self::ReplaceStop(node) => TraverseAction::ReplaceStop(f(node)),
        }
    }
}

/// Untyped result of a generated `walk_mut_*` function.
///
/// Every AST handle wraps a [`NodeId`], so walks share this representation and
/// enum dispatch passes inner results through without re-tagging them. The
/// public [`TraverseAction`] is restored only at the `traverse_mut_with`
/// boundary.
#[derive(Clone, Copy)]
pub(crate) enum WalkMut {
    /// All hooks ran, and no descendant stopped.
    Continue,
    /// A hook skipped the rest of the node.
    Skip,
    /// A hook stopped the traversal.
    Stop,
    ReplaceContinue(NodeId),
    ReplaceSkip(NodeId),
    ReplaceStop(NodeId),
    /// All hooks ran, and a descendant stopped the traversal.
    Stopped,
}

impl WalkMut {
    /// Result of an enter hook that did not return `Continue`.
    #[inline]
    pub(crate) fn enter<T: AstNode>(action: TraverseAction<T>) -> Self {
        match action {
            TraverseAction::Continue => Self::Continue,
            TraverseAction::Skip => Self::Skip,
            TraverseAction::Stop => Self::Stop,
            TraverseAction::ReplaceContinue(node) => Self::ReplaceContinue(node.node_id()),
            TraverseAction::ReplaceSkip(node) => Self::ReplaceSkip(node.node_id()),
            TraverseAction::ReplaceStop(node) => Self::ReplaceStop(node.node_id()),
        }
    }

    /// Result of an exit hook that did not return `Continue`. `children` is
    /// `Stopped` when a descendant stopped the traversal. The stop is
    /// preserved, but the exit hook can still replace its node.
    #[inline]
    pub(crate) fn exit<T: AstNode>(action: TraverseAction<T>, children: Self) -> Self {
        let stopped = matches!(children, Self::Stopped);
        match action {
            TraverseAction::ReplaceContinue(node) | TraverseAction::ReplaceSkip(node)
                if stopped =>
            {
                Self::ReplaceStop(node.node_id())
            }
            TraverseAction::Continue | TraverseAction::Skip if stopped => Self::Stop,
            action => Self::enter(action),
        }
    }

    /// Convert to the owning slot's action. A skip is consumed at the node
    /// boundary.
    #[inline]
    pub(crate) fn into_action<T>(self, wrap: impl FnOnce(NodeId) -> T) -> TraverseAction<T> {
        match self {
            Self::Continue | Self::Skip => TraverseAction::Continue,
            Self::Stop | Self::Stopped => TraverseAction::Stop,
            Self::ReplaceContinue(node) => TraverseAction::ReplaceContinue(wrap(node)),
            Self::ReplaceSkip(node) => TraverseAction::ReplaceSkip(wrap(node)),
            Self::ReplaceStop(node) => TraverseAction::ReplaceStop(wrap(node)),
        }
    }
}

/// Attach replacements before visiting them, and consume local skips without
/// propagating them to the owning node. Only a stop escapes the child slot.
#[inline]
pub(crate) fn traverse_mut_slot<'a, State, N, V>(
    mut node: N,
    traverser: &mut V,
    ctx: &mut TraverseMutCtx<'_, 'a, State>,
    mut attach: impl FnMut(&mut TraverseMutCtx<'_, 'a, State>, N),
) -> bool
where
    N: Copy + crate::TraverseMutWith<'a, State>,
    V: ?Sized + crate::TraverseMut<'a, State>,
{
    loop {
        match node.traverse_mut_with(traverser, ctx) {
            TraverseAction::Continue | TraverseAction::Skip => return false,
            TraverseAction::Stop => return true,
            TraverseAction::ReplaceContinue(replacement) => {
                attach(ctx, replacement);
                node = replacement;
            }
            TraverseAction::ReplaceSkip(replacement) => {
                attach(ctx, replacement);
                return false;
            }
            TraverseAction::ReplaceStop(replacement) => {
                attach(ctx, replacement);
                return true;
            }
        }
    }
}

/// Shared AST and mutable user state passed to every [`crate::Traverse`] hook.
///
/// AST mutation requires [`TraverseMutCtx`]:
pub struct TraverseCtx<'traverse, 'ast, State = ()> {
    pub state: State,
    pub ast: &'traverse Ast<'ast>,
}

impl<'traverse, 'ast, State> TraverseCtx<'traverse, 'ast, State> {
    #[inline]
    pub fn new(ast: &'traverse Ast<'ast>, state: State) -> Self {
        Self { state, ast }
    }
}

/// Mutable AST state passed to every [`crate::TraverseMut`] hook.
pub struct TraverseMutCtx<'traverse, 'ast, State = ()> {
    pub state: State,
    pub ast: &'traverse mut Ast<'ast>,
}

impl<'traverse, 'ast, State> TraverseMutCtx<'traverse, 'ast, State> {
    #[inline]
    pub fn new(ast: &'traverse mut Ast<'ast>, state: State) -> Self {
        Self { state, ast }
    }
}

macro_rules! dummy_visit_mut_impl {
    ($ty:ty) => {
        impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for $ty {
            #[inline]
            fn visit_with(self, _visitor: &mut V) {}

            #[inline]
            fn visit_children_with(self, _visitor: &mut V) {}
        }

        impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for $ty {
            #[inline]
            fn visit_mut_with(self, _visitor: &mut V) -> Option<Self> {
                None
            }

            #[inline]
            fn visit_mut_children_with(self, _visitor: &mut V) -> Option<Self> {
                None
            }
        }
    };
}

dummy_visit_mut_impl!(bool);
dummy_visit_mut_impl!(f64);
dummy_visit_mut_impl!(Span);
dummy_visit_mut_impl!(AstUtf8);
dummy_visit_mut_impl!(AstWtf8);
dummy_visit_mut_impl!(ImportPhase);
dummy_visit_mut_impl!(SourceType);
dummy_visit_mut_impl!(MethodDefinitionKind);
dummy_visit_mut_impl!(Accessibility);
dummy_visit_mut_impl!(ClassType);
dummy_visit_mut_impl!(BinaryOperator);
dummy_visit_mut_impl!(LogicalOperator);
dummy_visit_mut_impl!(UnaryOperator);
dummy_visit_mut_impl!(UpdateOperator);
dummy_visit_mut_impl!(AssignmentOperator);
dummy_visit_mut_impl!(VariableKind);
dummy_visit_mut_impl!(FunctionType);
dummy_visit_mut_impl!(FormalParameterKind);
dummy_visit_mut_impl!(PropertyKind);
dummy_visit_mut_impl!(ImportOrExportKind);
dummy_visit_mut_impl!(TsTypeOperatorKind);
dummy_visit_mut_impl!(TsMethodSignatureKind);
dummy_visit_mut_impl!(TsModuleDeclarationKind);
dummy_visit_mut_impl!(TsMappedTypeModifier);

#[cfg(test)]
mod tests {
    use swc_next_allocator::Allocator;

    use crate::{
        Ast, BinaryExpression, BinaryOperator, DUMMY_SP, Expr, ExpressionStatement,
        IdentifierReference, NodeKind, TraverseAction, TraverseMut, TraverseMutCtx,
        TraverseMutWith,
    };

    /// Stops at the first identifier and records the hooks that still run.
    struct StopAtIdentifier {
        replace_binary: bool,
        hooks: Vec<&'static str>,
    }

    impl<'a> TraverseMut<'a> for StopAtIdentifier {
        fn enter_identifier_reference(
            &mut self,
            _node: IdentifierReference,
            _ctx: &mut TraverseMutCtx<'_, 'a>,
        ) -> TraverseAction<IdentifierReference> {
            self.hooks.push("enter_identifier_reference");
            TraverseAction::Stop
        }

        fn exit_binary_expression(
            &mut self,
            _node: BinaryExpression,
            ctx: &mut TraverseMutCtx<'_, 'a>,
        ) -> TraverseAction<BinaryExpression> {
            self.hooks.push("exit_binary_expression");
            if self.replace_binary {
                let left = Expr::NullLiteral(ctx.ast.null_literal(DUMMY_SP));
                let right = Expr::NullLiteral(ctx.ast.null_literal(DUMMY_SP));
                let replacement =
                    ctx.ast
                        .binary_expression(DUMMY_SP, left, right, BinaryOperator::Add);
                TraverseAction::ReplaceContinue(replacement)
            } else {
                TraverseAction::Continue
            }
        }

        fn exit_expr(
            &mut self,
            _node: Expr,
            _ctx: &mut TraverseMutCtx<'_, 'a>,
        ) -> TraverseAction<Expr> {
            self.hooks.push("exit_expr");
            TraverseAction::Continue
        }

        fn exit_expression_statement(
            &mut self,
            _node: ExpressionStatement,
            _ctx: &mut TraverseMutCtx<'_, 'a>,
        ) -> TraverseAction<ExpressionStatement> {
            self.hooks.push("exit_expression_statement");
            TraverseAction::Continue
        }
    }

    fn statement(ast: &mut Ast<'_>) -> ExpressionStatement {
        let a = ast.add_utf8("a");
        let a = Expr::IdentifierReference(ast.identifier_reference(DUMMY_SP, a));
        let b = ast.add_utf8("b");
        let b = Expr::IdentifierReference(ast.identifier_reference(DUMMY_SP, b));
        let binary = ast.binary_expression(DUMMY_SP, a, b, BinaryOperator::Add);
        ast.expression_statement(DUMMY_SP, Expr::BinaryExpression(binary))
    }

    fn traverse(
        ast: &mut Ast<'_>,
        stmt: ExpressionStatement,
        replace_binary: bool,
    ) -> (TraverseAction<ExpressionStatement>, Vec<&'static str>) {
        let mut traverser = StopAtIdentifier {
            replace_binary,
            hooks: Vec::new(),
        };
        let mut ctx = TraverseMutCtx::new(ast, ());
        let action = stmt.traverse_mut_with(&mut traverser, &mut ctx);
        (action, traverser.hooks)
    }

    #[test]
    fn traverse_mut_descendant_stop() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("", &allocator);

        let stmt = statement(&mut ast);
        let (action, hooks) = traverse(&mut ast, stmt, false);
        assert_eq!(action, TraverseAction::Stop);
        assert_eq!(
            hooks,
            [
                "enter_identifier_reference",
                "exit_binary_expression",
                "exit_expr",
                "exit_expression_statement"
            ]
        );

        let stmt = statement(&mut ast);
        let original = stmt.expression(&ast).node_id();
        let (action, hooks) = traverse(&mut ast, stmt, true);
        assert_eq!(action, TraverseAction::Stop);
        assert_eq!(
            hooks,
            [
                "enter_identifier_reference",
                "exit_binary_expression",
                "exit_expression_statement"
            ]
        );
        let replacement = stmt.expression(&ast).node_id();
        assert_ne!(replacement, original);
        assert_eq!(ast.node_kind(replacement), NodeKind::BinaryExpression);
    }
}
