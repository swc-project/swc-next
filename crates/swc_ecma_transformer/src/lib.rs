use swc_next_ecma_ast::{
    ArrowFunctionExpression, AstCtx, BlockStatement, CallExpression, CatchClause, Class, ClassBody,
    ClassType, Decl, Diagnostic, ExportDefaultDeclarationKind, Expr, ForInStatement,
    ForOfStatement, ForStatement, FormalParameters, Function, FunctionBody, NodeId, Program,
    StaticBlock, SwitchCase, TraverseAction, TraverseMut, TraverseMutCtx, TraverseMutWith,
    TsCallSignatureDeclaration, TsConstructSignatureDeclaration, TsEnumBody, TsFunction,
    TsIndexSignature, TsInterfaceDeclaration, TsMethodSignature, TsModuleBlock, TsThisParameter,
    TsType, TsTypeAliasDeclaration, TsTypeAnnotation, TsTypeParameter, TsTypeParameterDeclaration,
};
use swc_next_ecma_semantic::{AnalyzeOptions, ScopeModel, Semantic, SemanticAst, analyze};

use crate::{
    assumptions::Assumptions,
    context::{ContainerOwner, TransformCtx, TransformerState},
};

mod context;
mod helpers;
mod uid;

pub use helpers::HelperMode;

pub mod assumptions;
pub mod es2019;
pub mod es2022;
pub mod jsx;
pub mod normalize;
pub mod proposal;
pub mod typescript;

#[derive(Default)]
pub struct TransformerOptions {
    /// How to emit runtime helpers. Modules default to imports, and scripts inline helpers.
    pub helpers: HelperMode,
    pub filename: Option<String>,
    pub assumptions: Assumptions,
    pub proposal: proposal::ProposalOptions,

    // Transform Options
    pub es2019: es2019::ES2019Options,
    pub es2022: es2022::ES2022Options,
    pub jsx: Option<jsx::JsxOptions>,
    pub typescript: Option<typescript::TypeScriptOptions>,
}

/// Semantic indexes for the transformed AST.
pub struct TransformerReturn<'a> {
    pub semantic: ScopeModel<'a>,
    pub diagnostics: Vec<Diagnostic>,
}

pub struct Transformer;

impl Transformer {
    /// Transforms a program using its bound semantic model.
    ///
    /// Consumes the analysis binding, releasing the AST for subsequent use.
    pub fn transform<'ast>(
        analyzed: SemanticAst<'ast, &mut AstCtx<'ast>>,
        mut options: TransformerOptions,
    ) -> TransformerReturn<'ast> {
        let mut transformer = MultiPassTransformer::new(analyzed);
        let filename = options.filename.as_deref();
        let mut diagnostics = Vec::new();

        // JSX Transformer
        let jsx = &mut options.jsx;
        if let Some(jsx) = jsx {
            transformer.jsx_pragmas(jsx);
            let jsx_diagnostics = transformer.jsx(jsx, filename);
            transformer.analyze();
            diagnostics.extend(jsx_diagnostics);
        }

        // TypeScript Transformer
        if let Some(typescript) = options.typescript.as_mut() {
            if let Some(jsx) = jsx {
                typescript.jsx.automatic = jsx.runtime == jsx::JsxRuntime::Automatic;
                typescript.jsx.pragma = jsx.pragma.clone();
                typescript.jsx.pragma_frag = jsx.pragma_frag.clone();
            }
            transformer.typescript(typescript);
            transformer.analyze();
        }

        let semantic = ECMAScriptTransformer::transform(transformer, &options);
        TransformerReturn {
            semantic,
            diagnostics,
        }
    }
}

/// Hosts the JSX and TypeScript transformers as separate passes.
///
/// When both are enabled, the pipeline runs JSX -> reanalyze -> TypeScript ->
/// reanalyze before handing off to [`ECMAScriptTransformer`]. These transformers
/// should eventually move into `ECMAScriptTransformer` and share its single pass.
///
/// Starts with an exclusive analysis binding. JSX and TypeScript lowering may
/// leave the model stale; callers must reanalyze before the next pass or discard
/// the pipeline. React Refresh reanalyzes before returning to JSX lowering.
/// This ordering is an internal safety invariant.
struct MultiPassTransformer<'ctx, 'a> {
    ctx: &'ctx mut AstCtx<'a>,
    semantic: Semantic<'a>,
}

impl<'ctx, 'a> MultiPassTransformer<'ctx, 'a> {
    fn new(analyzed: SemanticAst<'a, &'ctx mut AstCtx<'a>>) -> Self {
        let (ctx, semantic) = analyzed.into_parts();
        Self { ctx, semantic }
    }

    fn analyze(&mut self) {
        let (_, semantic) = analyze(&*self.ctx, AnalyzeOptions::default())
            .semantic
            .into_parts();
        self.semantic = semantic;
    }
}

struct ECMAScriptTransformer {
    es2019: es2019::ES2019,
    es2022: es2022::ES2022,
    jsx_display_name: jsx::DisplayName,
}

impl ECMAScriptTransformer {
    fn transform<'a>(
        multi_pass: MultiPassTransformer<'_, 'a>,
        options: &TransformerOptions,
    ) -> ScopeModel<'a> {
        let MultiPassTransformer { ctx: ast, semantic } = multi_pass;
        let program = ast.root_program();
        let state = TransformerState::new(ScopeModel::new(semantic), options.helpers);
        let mut transformer = Self {
            es2019: es2019::ES2019::new(options.es2019),
            es2022: es2022::ES2022::new(options.es2022, options.proposal),
            jsx_display_name: jsx::DisplayName::new(options, options.filename.clone()),
        };
        // SAFETY: The pipeline reanalyzes after each JSX and TypeScript pass,
        // so its model is current. Traversal and helper injection use only
        // handles from this AST and preserve its invariants.
        let mut ctx = TraverseMutCtx::new(unsafe { ast.ast_mut() }, state);
        let _ = program.traverse_mut_with(&mut transformer, &mut ctx);
        TransformCtx::new(&mut ctx).finish();
        ctx.state.semantic
    }
}

impl<'a> TraverseMut<'a, TransformerState<'a>> for ECMAScriptTransformer {
    fn exit_call_expression(
        &mut self,
        node: CallExpression,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<CallExpression> {
        self.jsx_display_name
            .exit_call_expression(node, &mut TransformCtx::new(ctx));
        TraverseAction::Continue
    }

    fn enter_program(
        &mut self,
        node: Program,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<Program> {
        let scope = ctx.state.semantic.scope_of(node.node_id()).unwrap();
        ctx.state.scope_stack.push(scope);
        TraverseAction::Continue
    }

    fn exit_program(
        &mut self,
        node: Program,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<Program> {
        let mut transform = TransformCtx::new(ctx);
        transform.protect_helper_globals();
        transform.inject_helpers(node);
        transform.inject_statements_to(node.node_id());
        ctx.state.scope_stack.pop().unwrap();
        TraverseAction::Continue
    }

    fn enter_function(
        &mut self,
        node: Function,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<Function> {
        let scope = ctx.state.semantic.scope_of(node.node_id()).unwrap();
        ctx.state.scope_stack.push(scope);
        TraverseAction::Continue
    }

    fn exit_function(
        &mut self,
        node: Function,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<Function> {
        let mut transform = TransformCtx::new(ctx);
        let body = node.body(transform.ast).node_id();
        transform.inject_statements_to(body);
        ctx.state.scope_stack.pop().unwrap();
        TraverseAction::Continue
    }

    fn enter_arrow_function_expression(
        &mut self,
        node: ArrowFunctionExpression,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<ArrowFunctionExpression> {
        let scope = ctx.state.semantic.scope_of(node.node_id()).unwrap();
        ctx.state.scope_stack.push(scope);
        TraverseAction::Continue
    }

    fn exit_arrow_function_expression(
        &mut self,
        node: ArrowFunctionExpression,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<ArrowFunctionExpression> {
        let mut transform = TransformCtx::new(ctx);
        let container = node
            .body(transform.ast)
            .as_function_body(transform.ast)
            .map_or(node.node_id(), |body| body.node_id());
        transform.inject_statements_to(container);
        ctx.state.scope_stack.pop().unwrap();
        TraverseAction::Continue
    }

    fn enter_function_body(
        &mut self,
        node: FunctionBody,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<FunctionBody> {
        if let Some(scope) = ctx.state.semantic.scope_of(node.node_id()) {
            ctx.state.scope_stack.push(scope);
        }
        TraverseAction::Continue
    }

    fn exit_function_body(
        &mut self,
        node: FunctionBody,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<FunctionBody> {
        let mut transform = TransformCtx::new(ctx);
        transform.inject_statements_to(node.node_id());
        if ctx.state.semantic.scope_of(node.node_id()).is_some() {
            ctx.state.scope_stack.pop().unwrap();
        }
        TraverseAction::Continue
    }

    fn enter_block_statement(
        &mut self,
        node: BlockStatement,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<BlockStatement> {
        if let Some(scope) = ctx.state.semantic.scope_of(node.node_id()) {
            ctx.state.scope_stack.push(scope);
        }
        TraverseAction::Continue
    }

    fn exit_block_statement(
        &mut self,
        node: BlockStatement,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<BlockStatement> {
        let mut transform = TransformCtx::new(ctx);
        transform.inject_statements_to(node.node_id());
        if ctx.state.semantic.scope_of(node.node_id()).is_some() {
            ctx.state.scope_stack.pop().unwrap();
        }
        TraverseAction::Continue
    }

    fn enter_static_block(
        &mut self,
        node: StaticBlock,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<StaticBlock> {
        let scope = ctx.state.semantic.scope_of(node.node_id()).unwrap();
        ctx.state.scope_stack.push(scope);
        TraverseAction::Continue
    }

    fn exit_static_block(
        &mut self,
        node: StaticBlock,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<StaticBlock> {
        let mut transform = TransformCtx::new(ctx);
        transform.inject_statements_to(node.node_id());
        ctx.state.scope_stack.pop().unwrap();
        TraverseAction::Continue
    }

    fn enter_ts_module_block(
        &mut self,
        node: TsModuleBlock,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<TsModuleBlock> {
        let scope = ctx.state.semantic.scope_of(node.node_id()).unwrap();
        ctx.state.scope_stack.push(scope);
        TraverseAction::Continue
    }

    fn exit_ts_module_block(
        &mut self,
        node: TsModuleBlock,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<TsModuleBlock> {
        let mut transform = TransformCtx::new(ctx);
        transform.inject_statements_to(node.node_id());
        ctx.state.scope_stack.pop().unwrap();
        TraverseAction::Continue
    }

    fn enter_class(
        &mut self,
        node: Class,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<Class> {
        let scope = ctx.state.semantic.scope_of(node.node_id()).unwrap();
        ctx.state.scope_stack.push(scope);
        TraverseAction::Continue
    }

    fn exit_class(
        &mut self,
        _node: Class,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<Class> {
        ctx.state.scope_stack.pop().unwrap();
        TraverseAction::Continue
    }

    fn enter_catch_clause(
        &mut self,
        node: CatchClause,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<CatchClause> {
        let scope = ctx.state.semantic.scope_of(node.node_id()).unwrap();
        ctx.state.scope_stack.push(scope);
        self.es2019
            .enter_catch_clause(node, &mut TransformCtx::new(ctx));
        TraverseAction::Continue
    }
    fn exit_catch_clause(
        &mut self,
        _node: CatchClause,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<CatchClause> {
        ctx.state.scope_stack.pop().unwrap();
        TraverseAction::Continue
    }

    fn enter_for_statement(
        &mut self,
        node: ForStatement,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<ForStatement> {
        let scope = ctx.state.semantic.scope_of(node.node_id()).unwrap();
        ctx.state.scope_stack.push(scope);
        TraverseAction::Continue
    }

    fn exit_for_statement(
        &mut self,
        _node: ForStatement,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<ForStatement> {
        ctx.state.scope_stack.pop().unwrap();
        TraverseAction::Continue
    }

    fn enter_for_in_statement(
        &mut self,
        node: ForInStatement,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<ForInStatement> {
        let scope = ctx.state.semantic.scope_of(node.node_id()).unwrap();
        ctx.state.scope_stack.push(scope);
        TraverseAction::Continue
    }

    fn exit_for_in_statement(
        &mut self,
        _node: ForInStatement,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<ForInStatement> {
        ctx.state.scope_stack.pop().unwrap();
        TraverseAction::Continue
    }

    fn enter_for_of_statement(
        &mut self,
        node: ForOfStatement,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<ForOfStatement> {
        let scope = ctx.state.semantic.scope_of(node.node_id()).unwrap();
        ctx.state.scope_stack.push(scope);
        TraverseAction::Continue
    }

    fn exit_for_of_statement(
        &mut self,
        _node: ForOfStatement,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<ForOfStatement> {
        ctx.state.scope_stack.pop().unwrap();
        TraverseAction::Continue
    }

    fn enter_ts_function(
        &mut self,
        node: TsFunction,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<TsFunction> {
        let scope = ctx.state.semantic.scope_of(node.node_id()).unwrap();
        ctx.state.scope_stack.push(scope);
        TraverseAction::Continue
    }

    fn exit_ts_function(
        &mut self,
        _node: TsFunction,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<TsFunction> {
        ctx.state.scope_stack.pop().unwrap();
        TraverseAction::Continue
    }

    fn enter_ts_enum_body(
        &mut self,
        node: TsEnumBody,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<TsEnumBody> {
        let scope = ctx.state.semantic.scope_of(node.node_id()).unwrap();
        ctx.state.scope_stack.push(scope);
        TraverseAction::Continue
    }

    fn exit_ts_enum_body(
        &mut self,
        _node: TsEnumBody,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<TsEnumBody> {
        ctx.state.scope_stack.pop().unwrap();
        TraverseAction::Continue
    }

    fn enter_ts_interface_declaration(
        &mut self,
        node: TsInterfaceDeclaration,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<TsInterfaceDeclaration> {
        let scope = ctx.state.semantic.scope_of(node.node_id()).unwrap();
        ctx.state.scope_stack.push(scope);
        TraverseAction::Continue
    }

    fn exit_ts_interface_declaration(
        &mut self,
        _node: TsInterfaceDeclaration,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<TsInterfaceDeclaration> {
        ctx.state.scope_stack.pop().unwrap();
        TraverseAction::Continue
    }

    fn enter_ts_type_alias_declaration(
        &mut self,
        node: TsTypeAliasDeclaration,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<TsTypeAliasDeclaration> {
        let scope = ctx.state.semantic.scope_of(node.node_id()).unwrap();
        ctx.state.scope_stack.push(scope);
        TraverseAction::Continue
    }

    fn exit_ts_type_alias_declaration(
        &mut self,
        _node: TsTypeAliasDeclaration,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<TsTypeAliasDeclaration> {
        ctx.state.scope_stack.pop().unwrap();
        TraverseAction::Continue
    }

    fn enter_formal_parameters(
        &mut self,
        node: FormalParameters,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<FormalParameters> {
        if let Some(scope) = ctx.state.semantic.scope_of(node.node_id()) {
            ctx.state.scope_stack.push(scope);
        }
        TraverseAction::Continue
    }

    fn exit_formal_parameters(
        &mut self,
        node: FormalParameters,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<FormalParameters> {
        if ctx.state.semantic.scope_of(node.node_id()).is_some() {
            ctx.state.scope_stack.pop().unwrap();
        }
        TraverseAction::Continue
    }

    fn enter_ts_this_parameter(
        &mut self,
        node: TsThisParameter,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<TsThisParameter> {
        let scope = ctx.state.semantic.scope_of(node.node_id()).unwrap();
        ctx.state.scope_stack.push(scope);
        TraverseAction::Continue
    }

    fn exit_ts_this_parameter(
        &mut self,
        _node: TsThisParameter,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<TsThisParameter> {
        ctx.state.scope_stack.pop().unwrap();
        TraverseAction::Continue
    }

    fn enter_ts_type_parameter_declaration(
        &mut self,
        node: TsTypeParameterDeclaration,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<TsTypeParameterDeclaration> {
        let scope = ctx.state.semantic.scope_of(node.node_id()).unwrap();
        ctx.state.scope_stack.push(scope);
        TraverseAction::Continue
    }

    fn exit_ts_type_parameter_declaration(
        &mut self,
        _node: TsTypeParameterDeclaration,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<TsTypeParameterDeclaration> {
        ctx.state.scope_stack.pop().unwrap();
        TraverseAction::Continue
    }

    fn enter_ts_type_annotation(
        &mut self,
        node: TsTypeAnnotation,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<TsTypeAnnotation> {
        if let Some(scope) = ctx.state.semantic.scope_of(node.node_id()) {
            ctx.state.scope_stack.push(scope);
        }
        TraverseAction::Continue
    }

    fn exit_ts_type_annotation(
        &mut self,
        node: TsTypeAnnotation,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<TsTypeAnnotation> {
        if ctx.state.semantic.scope_of(node.node_id()).is_some() {
            ctx.state.scope_stack.pop().unwrap();
        }
        TraverseAction::Continue
    }

    fn enter_ts_type_parameter(
        &mut self,
        node: TsTypeParameter,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<TsTypeParameter> {
        if let Some(scope) = ctx.state.semantic.scope_of(node.node_id()) {
            ctx.state.scope_stack.push(scope);
        }
        TraverseAction::Continue
    }

    fn exit_ts_type_parameter(
        &mut self,
        node: TsTypeParameter,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<TsTypeParameter> {
        if ctx.state.semantic.scope_of(node.node_id()).is_some() {
            ctx.state.scope_stack.pop().unwrap();
        }
        TraverseAction::Continue
    }

    fn enter_ts_type(
        &mut self,
        node: TsType,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<TsType> {
        if let Some(scope) = ctx.state.semantic.scope_of(node.node_id()) {
            ctx.state.scope_stack.push(scope);
        }
        TraverseAction::Continue
    }

    fn exit_ts_type(
        &mut self,
        node: TsType,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<TsType> {
        if ctx.state.semantic.scope_of(node.node_id()).is_some() {
            ctx.state.scope_stack.pop().unwrap();
        }
        TraverseAction::Continue
    }

    fn enter_ts_method_signature(
        &mut self,
        node: TsMethodSignature,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<TsMethodSignature> {
        let scope = ctx.state.semantic.scope_of(node.node_id()).unwrap();
        ctx.state.scope_stack.push(scope);
        TraverseAction::Continue
    }

    fn exit_ts_method_signature(
        &mut self,
        _node: TsMethodSignature,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<TsMethodSignature> {
        ctx.state.scope_stack.pop().unwrap();
        TraverseAction::Continue
    }

    fn enter_ts_call_signature_declaration(
        &mut self,
        node: TsCallSignatureDeclaration,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<TsCallSignatureDeclaration> {
        let scope = ctx.state.semantic.scope_of(node.node_id()).unwrap();
        ctx.state.scope_stack.push(scope);
        TraverseAction::Continue
    }

    fn exit_ts_call_signature_declaration(
        &mut self,
        _node: TsCallSignatureDeclaration,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<TsCallSignatureDeclaration> {
        ctx.state.scope_stack.pop().unwrap();
        TraverseAction::Continue
    }

    fn enter_ts_construct_signature_declaration(
        &mut self,
        node: TsConstructSignatureDeclaration,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<TsConstructSignatureDeclaration> {
        let scope = ctx.state.semantic.scope_of(node.node_id()).unwrap();
        ctx.state.scope_stack.push(scope);
        TraverseAction::Continue
    }

    fn exit_ts_construct_signature_declaration(
        &mut self,
        _node: TsConstructSignatureDeclaration,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<TsConstructSignatureDeclaration> {
        ctx.state.scope_stack.pop().unwrap();
        TraverseAction::Continue
    }

    fn enter_ts_index_signature(
        &mut self,
        node: TsIndexSignature,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<TsIndexSignature> {
        let scope = ctx.state.semantic.scope_of(node.node_id()).unwrap();
        ctx.state.scope_stack.push(scope);
        TraverseAction::Continue
    }

    fn exit_ts_index_signature(
        &mut self,
        _node: TsIndexSignature,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<TsIndexSignature> {
        ctx.state.scope_stack.pop().unwrap();
        TraverseAction::Continue
    }

    fn enter_node(&mut self, node: NodeId, ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>) {
        ctx.state.path.push(node);
    }

    fn exit_node(&mut self, node: NodeId, ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>) {
        let popped = ctx.state.path.pop();
        debug_assert_eq!(popped, Some(node));
    }

    fn enter_switch_case(
        &mut self,
        _node: SwitchCase,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<SwitchCase> {
        let parent = TransformCtx::new(ctx).parent().expect("switch case parent");
        let scope = ctx.state.semantic.scope_of(parent).expect("switch scope");
        ctx.state.scope_stack.push(scope);
        TraverseAction::Continue
    }

    fn exit_switch_case(
        &mut self,
        node: SwitchCase,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<SwitchCase> {
        let mut transform = TransformCtx::new(ctx);
        transform.inject_statements_to(node.node_id());
        ctx.state.scope_stack.pop().unwrap();
        TraverseAction::Continue
    }

    fn enter_class_body(
        &mut self,
        node: ClassBody,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<ClassBody> {
        let mut ctx = TransformCtx::new(ctx);
        self.es2022.enter_class_body(node, &mut ctx);
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
            let _ = self.es2022.exit_class(class, &mut TransformCtx::new(ctx));
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
            return self.es2022.exit_class(class, &mut TransformCtx::new(ctx));
        }
        TraverseAction::Continue
    }
}
