use super::Mutation;
use crate::state::TransformerState;
use swc_next_ecma_ast::{BindingIdentifier, DUMMY_SP, ScopeId};
use swc_next_ecma_ast::{NodeId, TraverseMut, TraverseMutCtx, TraverseMutWith};
use swc_next_ecma_semantic::SymbolFlags;

struct AtNode<'f, 's, 'a> {
    target: NodeId,
    run: &'f mut dyn FnMut(&mut Mutation<'_, 's, 'a>),
}

impl<'s, 'a> TraverseMut<'a, &mut TransformerState<'s, 'a>> for AtNode<'_, 's, 'a> {
    fn enter_node(
        &mut self,
        node: NodeId,
        ctx: &mut TraverseMutCtx<'_, 'a, &mut TransformerState<'s, 'a>>,
    ) {
        ctx.state.path.push(node);
        if node != self.target {
            return;
        }
        (self.run)(&mut Mutation {
            ast: ctx.ast,
            state: ctx.state,
        });
    }

    fn exit_node(
        &mut self,
        node: NodeId,
        ctx: &mut TraverseMutCtx<'_, 'a, &mut TransformerState<'s, 'a>>,
    ) {
        debug_assert_eq!(ctx.state.path.pop(), Some(node));
    }
}

impl<'s, 'a> Mutation<'_, 's, 'a> {
    pub(crate) fn create_named_binding(
        &mut self,
        name: &str,
        scope: ScopeId,
        flags: SymbolFlags,
    ) -> BindingIdentifier {
        assert!(
            self.state.semantic.scope(scope).active,
            "binding scope must be active"
        );
        assert!(
            self.state
                .semantic
                .binding(scope, name.as_bytes())
                .is_none(),
            "binding name must be available"
        );
        let name = self.ast.add_utf8(name);
        self.observe_name(name);
        let node = self.ast.binding_identifier(DUMMY_SP, name);
        self.state
            .semantic
            .register_binding(self.ast, node, scope, flags);
        node
    }

    /// Exercise mutations with a path maintained by the real traversal engine.
    pub(crate) fn at_node<R>(
        &mut self,
        target: NodeId,
        run: impl FnOnce(&mut Mutation<'_, 's, 'a>) -> R,
    ) -> R {
        let mut run = Some(run);
        let mut result = None;
        let mut callback = |ctx: &mut Mutation<'_, 's, 'a>| {
            result = Some(run.take().unwrap()(ctx));
        };
        let mut visitor = AtNode {
            target,
            run: &mut callback,
        };
        let program = self.ast().root_program();
        let mut ctx = TraverseMutCtx::new(self.ast, &mut *self.state);
        program.traverse_mut_with(&mut visitor, &mut ctx);
        result.expect("test target must be reachable")
    }
}
