use swc_next_ecma_ast::{
    ArrowFunctionExpression, Ast, BlockStatement, CallExpression, CatchClause, Class, ClassBody,
    ClassType, Decl, Diagnostic, ExportDefaultDeclarationKind, Expr, ForInStatement,
    ForOfStatement, ForStatement, FormalParameters, Function, FunctionBody, NodeId, Program,
    StaticBlock, SwitchCase, TraverseAction, TraverseMut, TraverseMutCtx, TraverseMutWith,
    TsCallSignatureDeclaration, TsConstructSignatureDeclaration, TsEnumBody, TsFunction,
    TsIndexSignature, TsInterfaceDeclaration, TsMethodSignature, TsModuleBlock, TsThisParameter,
    TsType, TsTypeAliasDeclaration, TsTypeAnnotation, TsTypeParameter, TsTypeParameterDeclaration,
};
use swc_next_ecma_semantic::{AnalyzeOptions, ScopeModel, Semantic, analyze};

use crate::{
    assumptions::Assumptions,
    context::{ContainerOwner, TransformCtx, TransformerState},
};

mod context;
mod helpers;
mod jsx_pragma;
mod syntax;
mod uid;
mod used_names;

pub mod assumptions;
pub mod es2022;
pub mod jsx;
pub mod normalize;
pub mod proposal;
pub mod typescript;

#[derive(Default)]
pub struct TransformerOptions {
    pub source_path: Option<String>,
    pub react_display_name: bool,
    pub assumptions: Assumptions,
    pub es2022: es2022::ES2022Options,
    pub proposal: proposal::ProposalOptions,
    pub jsx: Option<jsx::JsxOptions>,
    pub typescript: Option<typescript::TypeScriptOptions>,
}

/// Semantic indexes for the transformed AST.
pub struct TransformerReturn<'a> {
    pub semantic: ScopeModel<'a>,
    pub diagnostics: Vec<Diagnostic<'static>>,
}

pub struct Transformer {
    x0_es2022: es2022::ES2022,
    display_name: Option<jsx::DisplayName>,
}

impl Transformer {
    /// Run TypeScript and JSX syntax lowering, then synchronized scope transforms.
    /// Normalize runtime syntax before analysis when TypeScript is enabled, otherwise normalize all syntax.
    pub fn transform<'ast>(
        ast: &mut Ast<'ast>,
        mut semantic: Semantic<'ast>,
        mut options: TransformerOptions,
    ) -> TransformerReturn<'ast> {
        let program = ast.root_program();
        let mut runtime_import_anchor = None;
        if let Some(refresh) = options.jsx.as_ref().and_then(|jsx| jsx.refresh.as_ref()) {
            runtime_import_anchor = jsx::transform_refresh(program, ast, &semantic, refresh);
            semantic = analyze(ast, AnalyzeOptions::default()).semantic;
        }
        let jsx = options
            .jsx
            .take()
            .map(|jsx| jsx::resolve_options(program, ast, jsx));
        let syntax_transformed = options.typescript.is_some() || jsx.is_some();
        let syntax_result = match (options.typescript.take(), jsx) {
            (Some(typescript), Some(jsx)) => {
                let mut jsx =
                    jsx::JsxLowerer::new(&jsx, ast, &semantic, options.source_path.as_deref());
                jsx.set_runtime_import_anchor(runtime_import_anchor);
                typescript::transform_with_following(program, ast, &semantic, typescript, &mut jsx);
                jsx.take_result()
            }
            (Some(typescript), None) => {
                typescript::transform(program, ast, &semantic, typescript);
                syntax::SyntaxResult::default()
            }
            (None, Some(jsx)) => jsx::transform_resolved(
                program,
                ast,
                &semantic,
                jsx,
                options.source_path.as_deref(),
                runtime_import_anchor,
            ),
            (None, None) => syntax::SyntaxResult::default(),
        };
        if syntax_transformed {
            semantic = analyze(ast, AnalyzeOptions::default()).semantic;
        }

        let mut result = Self::transform_with_scopes(ast, ScopeModel::new(semantic), options);
        result.diagnostics = syntax_result.diagnostics;
        result
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
            display_name: options
                .react_display_name
                .then(|| jsx::DisplayName::new(options.source_path)),
        };

        let state = TransformerState::new(semantic);
        let mut ctx = TraverseMutCtx::new(ast, state);
        let _ = program.traverse_mut_with(&mut transformer, &mut ctx);
        TransformCtx::new(&mut ctx).finish();
        TransformerReturn {
            semantic: ctx.state.semantic,
            diagnostics: Vec::new(),
        }
    }
}

impl<'a> TraverseMut<'a, TransformerState<'a>> for Transformer {
    fn exit_call_expression(
        &mut self,
        node: CallExpression,
        ctx: &mut TraverseMutCtx<'_, 'a, TransformerState<'a>>,
    ) -> TraverseAction<CallExpression> {
        if let Some(display_name) = &self.display_name {
            display_name.exit_call_expression(node, &mut TransformCtx::new(ctx));
        }
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
}
