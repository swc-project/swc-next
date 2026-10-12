use swc_next_ecma_ast::{
    ArrowFunctionExpression, Ast, BlockStatement, ClassBody, ClassType, Decl,
    ExportDefaultDeclarationKind, Expr, Function, FunctionBody, NodeId, Program, StaticBlock,
    SwitchCase, TraverseAction, TraverseMut, TraverseMutCtx, TraverseMutWith, TsModuleBlock,
};
use swc_next_ecma_semantic::ScopeModel;

use crate::{
    assumptions::Assumptions,
    context::{ContainerOwner, Mutate},
    state::TransformerState,
};

mod comments;
mod context;
mod helpers;
mod state;
mod used_names;

pub mod assumptions;
pub mod es2022;
pub mod normalize;
pub mod proposal;

bitflags::bitflags! {
    #[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
    pub(crate) struct TransformFeatureFlags: u8 {
        const ES2022_STATIC_BLOCKS = 1 << 0;
        const PROPOSAL_PREVENT_EXTENSIONS = 1 << 1;
    }
}

#[derive(Default)]
pub struct TransformerOptions {
    pub assumptions: Assumptions,
    pub es2022: es2022::ES2022Options,
    pub proposal: proposal::ProposalOptions,
}

impl TransformerOptions {
    fn transform_features(&self) -> TransformFeatureFlags {
        let mut features = TransformFeatureFlags::empty();
        features.set(
            TransformFeatureFlags::ES2022_STATIC_BLOCKS,
            self.es2022.static_blocks,
        );
        features.set(
            TransformFeatureFlags::PROPOSAL_PREVENT_EXTENSIONS,
            self.proposal.prevent_extensions,
        );
        features
    }
}

pub struct Transformer {
    x0_es2022: es2022::ES2022,
}

impl Transformer {
    /// Transform an AST that was normalized before semantic analysis.
    /// Call `normalize::Normalizer::normalize` before constructing `semantic`.
    pub fn transform<'ast>(
        ast: &mut Ast<'ast>,
        semantic: &mut ScopeModel<'ast>,
        options: TransformerOptions,
    ) {
        let features = options.transform_features();
        let program = ast.root_program();

        let mut transformer = Self {
            x0_es2022: es2022::ES2022::new(features),
        };

        let state = TransformerState::new(semantic);
        let mut ctx = TraverseMutCtx::new(ast, state);
        let _ = program.traverse_mut_with(&mut transformer, &mut ctx);
        ctx.mutate().finish();
    }
}

impl<'s, 'a> TraverseMut<'a, TransformerState<'s, 'a>> for Transformer {
    fn enter_node(
        &mut self,
        node: NodeId,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'s, 'a>>,
    ) {
        ctx.state.path.push(node);
    }

    fn exit_node(
        &mut self,
        node: NodeId,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'s, 'a>>,
    ) {
        let popped = ctx.state.path.pop();
        debug_assert_eq!(popped, Some(node));
    }

    fn enter_class_body(
        &mut self,
        node: ClassBody,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'s, 'a>>,
    ) -> TraverseAction<ClassBody> {
        let ctx = &mut ctx.mutate();
        self.x0_es2022.enter_class_body(node, ctx);
        ctx.container_flush(ContainerOwner::ClassBody(node));
        TraverseAction::Continue
    }

    fn exit_class_body(
        &mut self,
        node: ClassBody,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'s, 'a>>,
    ) -> TraverseAction<ClassBody> {
        let ctx = &mut ctx.mutate();
        ctx.container_flush(ContainerOwner::ClassBody(node));
        TraverseAction::Continue
    }

    fn exit_decl(
        &mut self,
        node: Decl,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'s, 'a>>,
    ) -> TraverseAction<Decl> {
        if let Some(class) = node.as_class(ctx.ast)
            && class.class_type(ctx.ast) == ClassType::ClassDeclaration
        {
            let action = self.x0_es2022.exit_class(class, &mut ctx.mutate());
            debug_assert!(matches!(action, TraverseAction::Continue));
        }
        TraverseAction::Continue
    }

    fn exit_export_default_declaration_kind(
        &mut self,
        node: ExportDefaultDeclarationKind,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'s, 'a>>,
    ) -> TraverseAction<ExportDefaultDeclarationKind> {
        // Default exports dispatch Class directly rather than through Decl.
        if let Some(class) = node.as_class(ctx.ast)
            && class.class_type(ctx.ast) == ClassType::ClassDeclaration
        {
            let action = self.exit_decl(Decl::Class(class), ctx);
            debug_assert!(matches!(action, TraverseAction::Continue));
        }
        TraverseAction::Continue
    }

    fn exit_expr(
        &mut self,
        node: Expr,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'s, 'a>>,
    ) -> TraverseAction<Expr> {
        if let Some(class) = node.as_class(ctx.ast)
            && class.class_type(ctx.ast) == ClassType::ClassExpression
        {
            return self.x0_es2022.exit_class(class, &mut ctx.mutate());
        }
        TraverseAction::Continue
    }

    fn exit_program(
        &mut self,
        node: Program,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'s, 'a>>,
    ) -> TraverseAction<Program> {
        let ctx = &mut ctx.mutate();
        ctx.inject_helpers(node);
        ctx.inject_statements_to(node.node_id());
        TraverseAction::Continue
    }

    fn exit_function(
        &mut self,
        node: Function,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'s, 'a>>,
    ) -> TraverseAction<Function> {
        let ctx = &mut ctx.mutate();
        let body = node.body(ctx.ast).node_id();
        ctx.inject_statements_to(body);
        TraverseAction::Continue
    }

    fn exit_function_body(
        &mut self,
        node: FunctionBody,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'s, 'a>>,
    ) -> TraverseAction<FunctionBody> {
        let ctx = &mut ctx.mutate();
        ctx.inject_statements_to(node.node_id());
        TraverseAction::Continue
    }

    fn exit_block_statement(
        &mut self,
        node: BlockStatement,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'s, 'a>>,
    ) -> TraverseAction<BlockStatement> {
        let ctx = &mut ctx.mutate();
        ctx.inject_statements_to(node.node_id());
        TraverseAction::Continue
    }

    fn exit_static_block(
        &mut self,
        node: StaticBlock,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'s, 'a>>,
    ) -> TraverseAction<StaticBlock> {
        let ctx = &mut ctx.mutate();
        ctx.inject_statements_to(node.node_id());
        TraverseAction::Continue
    }

    fn exit_switch_case(
        &mut self,
        node: SwitchCase,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'s, 'a>>,
    ) -> TraverseAction<SwitchCase> {
        let ctx = &mut ctx.mutate();
        ctx.inject_statements_to(node.node_id());
        TraverseAction::Continue
    }

    fn exit_ts_module_block(
        &mut self,
        node: TsModuleBlock,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'s, 'a>>,
    ) -> TraverseAction<TsModuleBlock> {
        let ctx = &mut ctx.mutate();
        ctx.inject_statements_to(node.node_id());
        TraverseAction::Continue
    }

    fn exit_arrow_function_expression(
        &mut self,
        node: ArrowFunctionExpression,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'s, 'a>>,
    ) -> TraverseAction<ArrowFunctionExpression> {
        let ctx = &mut ctx.mutate();
        let container = node
            .body(ctx.ast)
            .as_function_body(ctx.ast)
            .map_or(node.node_id(), |body| body.node_id());
        ctx.inject_statements_to(container);
        TraverseAction::Continue
    }
}
