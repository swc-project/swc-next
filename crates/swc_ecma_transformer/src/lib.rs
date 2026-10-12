use swc_next_ecma_ast::{
    ArrowFunctionExpression, Ast, BlockStatement, ClassBody, ClassType, Decl,
    ExportDefaultDeclarationKind, Expr, Function, FunctionBody, NodeId, Program, StaticBlock,
    SwitchCase, TraverseAction, TraverseMut, TraverseMutCtx, TraverseMutWith, TsModuleBlock,
};
use swc_next_ecma_semantic::{AnalyzeOptions, ScopeModel, Semantic, analyze};

use crate::{
    assumptions::Assumptions,
    context::{ContainerOwner, Mutate},
    state::TransformerState,
};

mod context;
mod helpers;
mod jsx_pragma;
mod state;
mod used_names;

pub mod assumptions;
pub mod es2022;
pub mod normalize;
pub mod proposal;
pub mod typescript;

#[derive(Default)]
pub struct TransformerOptions {
    pub assumptions: Assumptions,
    pub es2022: es2022::ES2022Options,
    pub proposal: proposal::ProposalOptions,
    pub typescript: Option<typescript::TypeScriptOptions>,
}

pub struct Transformer {
    x0_es2022: es2022::ES2022,
}

impl Transformer {
    /// Run the Semantic-based TypeScript phase, then synchronized scope transforms.
    /// Normalize runtime syntax before analysis when TypeScript is enabled, otherwise normalize all syntax.
    pub fn transform<'ast>(
        ast: &mut Ast<'ast>,
        mut semantic: Semantic<'ast>,
        mut options: TransformerOptions,
    ) -> ScopeModel<'ast> {
        if let Some(typescript) = options.typescript.take() {
            typescript::transform(ast.root_program(), ast, &semantic, typescript);
            semantic = analyze(ast, AnalyzeOptions::default()).semantic;
        }

        let mut scopes = ScopeModel::new(semantic);
        Self::transform_with_scopes(ast, &mut scopes, options);
        scopes
    }

    /// Run synchronized transforms on an already normalized runtime AST and its ScopeModel.
    /// TypeScript must be disabled; use `transform` to run TypeScript lowering first.
    pub fn transform_with_scopes<'ast>(
        ast: &mut Ast<'ast>,
        semantic: &mut ScopeModel<'ast>,
        options: TransformerOptions,
    ) {
        debug_assert!(options.typescript.is_none());
        let program = ast.root_program();

        let mut transformer = Self {
            x0_es2022: es2022::ES2022::new(options.es2022, options.proposal),
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
