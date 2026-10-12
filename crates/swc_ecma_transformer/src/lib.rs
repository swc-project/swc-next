use swc_next_ecma_ast::{
    ArrowFunctionExpression, Ast, BlockStatement, ClassBody, ClassType, Decl,
    ExportDefaultDeclarationKind, Expr, Function, FunctionBody, NodeId, Program, StaticBlock,
    SwitchCase, TraverseAction, TraverseMut, TraverseMutCtx, TraverseMutWith, TsModuleBlock,
};
use swc_next_ecma_semantic::{AnalyzeOptions, ScopeModel, Semantic, analyze};

use crate::{
    assumptions::Assumptions,
    context::{ContainerOwner, TransformCtx, TransformerState},
};

mod context;
mod helpers;
mod jsx_pragma;
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

/// Semantic indexes for the transformed AST.
pub struct TransformerReturn<'a> {
    pub semantic: ScopeModel<'a>,
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
    ) -> TransformerReturn<'ast> {
        if let Some(typescript) = options.typescript.take() {
            typescript::transform(ast.root_program(), ast, &semantic, typescript);
            semantic = analyze(ast, AnalyzeOptions::default()).semantic;
        }

        Self::transform_with_scopes(ast, ScopeModel::new(semantic), options)
    }

    /// Run synchronized transforms on an already normalized runtime AST and its ScopeModel.
    /// TypeScript must be disabled; use `transform` to run TypeScript lowering first.
    fn transform_with_scopes<'ast>(
        ast: &mut Ast<'ast>,
        semantic: ScopeModel<'ast>,
        options: TransformerOptions,
    ) -> TransformerReturn<'ast> {
        debug_assert!(options.typescript.is_none());
        let program = ast.root_program();

        let mut transformer = Self {
            x0_es2022: es2022::ES2022::new(options.es2022, options.proposal),
        };

        let state = TransformerState::new(semantic);
        let mut ctx = TraverseMutCtx::new(ast, state);
        let _ = program.traverse_mut_with(&mut transformer, &mut ctx);
        TransformCtx::new(&mut ctx).finish();
        TransformerReturn {
            semantic: ctx.state.semantic,
        }
    }
}

impl<'a> TraverseMut<'a, TransformerState<'a>> for Transformer {
    fn enter_node(&mut self, node: NodeId, ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>) {
        ctx.state.path.push(node);
    }

    fn exit_node(&mut self, node: NodeId, ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>) {
        let popped = ctx.state.path.pop();
        debug_assert_eq!(popped, Some(node));
    }

    fn enter_class_body(
        &mut self,
        node: ClassBody,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<ClassBody> {
        let mut ctx = TransformCtx::new(ctx);
        self.x0_es2022.enter_class_body(node, &mut ctx);
        ctx.container_flush(ContainerOwner::ClassBody(node));
        TraverseAction::Continue
    }

    fn exit_class_body(
        &mut self,
        node: ClassBody,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<ClassBody> {
        let mut ctx = TransformCtx::new(ctx);
        ctx.container_flush(ContainerOwner::ClassBody(node));
        TraverseAction::Continue
    }

    fn exit_decl(
        &mut self,
        node: Decl,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<Decl> {
        if let Some(class) = node.as_class(ctx.ast)
            && class.class_type(ctx.ast) == ClassType::ClassDeclaration
        {
            let _ = self
                .x0_es2022
                .exit_class(class, &mut TransformCtx::new(ctx));
        }
        TraverseAction::Continue
    }

    fn exit_export_default_declaration_kind(
        &mut self,
        node: ExportDefaultDeclarationKind,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
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
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<Expr> {
        if let Some(class) = node.as_class(ctx.ast)
            && class.class_type(ctx.ast) == ClassType::ClassExpression
        {
            return self
                .x0_es2022
                .exit_class(class, &mut TransformCtx::new(ctx));
        }
        TraverseAction::Continue
    }

    fn exit_program(
        &mut self,
        node: Program,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<Program> {
        let mut ctx = TransformCtx::new(ctx);
        ctx.inject_helpers(node);
        ctx.inject_statements_to(node.node_id());
        TraverseAction::Continue
    }

    fn exit_function(
        &mut self,
        node: Function,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<Function> {
        let mut ctx = TransformCtx::new(ctx);
        let body = node.body(ctx.ast).node_id();
        ctx.inject_statements_to(body);
        TraverseAction::Continue
    }

    fn exit_function_body(
        &mut self,
        node: FunctionBody,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<FunctionBody> {
        let mut ctx = TransformCtx::new(ctx);
        ctx.inject_statements_to(node.node_id());
        TraverseAction::Continue
    }

    fn exit_block_statement(
        &mut self,
        node: BlockStatement,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<BlockStatement> {
        let mut ctx = TransformCtx::new(ctx);
        ctx.inject_statements_to(node.node_id());
        TraverseAction::Continue
    }

    fn exit_static_block(
        &mut self,
        node: StaticBlock,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<StaticBlock> {
        let mut ctx = TransformCtx::new(ctx);
        ctx.inject_statements_to(node.node_id());
        TraverseAction::Continue
    }

    fn exit_switch_case(
        &mut self,
        node: SwitchCase,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<SwitchCase> {
        let mut ctx = TransformCtx::new(ctx);
        ctx.inject_statements_to(node.node_id());
        TraverseAction::Continue
    }

    fn exit_ts_module_block(
        &mut self,
        node: TsModuleBlock,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<TsModuleBlock> {
        let mut ctx = TransformCtx::new(ctx);
        ctx.inject_statements_to(node.node_id());
        TraverseAction::Continue
    }

    fn exit_arrow_function_expression(
        &mut self,
        node: ArrowFunctionExpression,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<ArrowFunctionExpression> {
        let mut ctx = TransformCtx::new(ctx);
        let container = node
            .body(ctx.ast)
            .as_function_body(ctx.ast)
            .map_or(node.node_id(), |body| body.node_id());
        ctx.inject_statements_to(container);
        TraverseAction::Continue
    }
}
