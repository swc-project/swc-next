//! SWC's JSX transform on the Next AST.

use std::collections::{HashMap, HashSet};
use swc_next_ecma_ast::*;

use crate::resolver::Resolver;

mod build;
mod elements;
mod entities;
mod options;
mod pragma;
mod runtime;
mod self_context;
mod source;
mod text;

pub use options::{JsxOptions, JsxRuntime, ReactRefreshOptions};

use self_context::JsxSelfCtx;
use source::SourceLocations;

/// SWC JSX hook construction: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L212>.
/// SWC React hook composition: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/lib.rs#L79>.
pub(crate) fn transform(
    ast: &mut Ast<'_>,
    options: &JsxOptions,
    filename: Option<&str>,
    resolver: &mut Resolver,
) -> Vec<Diagnostic> {
    if options.runtime == Some(JsxRuntime::Preserve) {
        return Vec::new();
    }
    let mut diagnostics = Vec::new();
    let pragma = options.pragma.as_deref().unwrap_or("React.createElement");
    let pragma_frag = options.pragma_frag.as_deref().unwrap_or("React.Fragment");
    for (name, value) in [("pragma", pragma), ("pragmaFrag", pragma_frag)] {
        if !pragma::valid_path(value) {
            let mut diagnostic =
                Diagnostic::error("JSX pragma must be a dotted identifier path", DUMMY_SP);
            diagnostic.help = Some(name.into());
            diagnostics.push(diagnostic);
        }
    }
    if !diagnostics.is_empty() {
        return diagnostics;
    }
    let mut comments: HashMap<u32, Vec<Comment>> = HashMap::default();
    for &comment in ast.comments() {
        comments
            .entry(comment.attached_to)
            .or_default()
            .push(comment);
    }
    let mut jsx = Jsx {
        resolver,
        runtime: options.runtime.unwrap_or_default(),
        import_source: options.import_source.as_deref().unwrap_or("react").into(),
        import_jsx: None,
        import_jsxs: None,
        import_fragment: None,
        import_create_element: None,
        pragma: pragma.into(),
        pragma_frag: pragma_frag.into(),
        development: options.development.unwrap_or_default(),
        throw_if_namespace: options.throw_if_namespace.unwrap_or(true),
        filename: filename.unwrap_or_default().into(),
        source: SourceLocations::new(ast.source()),
        self_ctx: JsxSelfCtx::default(),
        method_functions: HashSet::default(),
        comments,
        diagnostics,
        top_level_node: true,
        pure_positions: HashSet::new(),
    };
    let program = ast.root_program();
    let _ = program.traverse_mut_with(&mut jsx, &mut TraverseMutCtx::new(ast, ()));
    let diagnostics = jsx.diagnostics;
    // CallExpression::pure represents SWC's synthetic annotation. Remove only
    // identical existing annotations which SWC's add_pure_comment deduplicates.
    let comments = ast
        .comments()
        .iter()
        .copied()
        .filter(|comment| {
            !(comment.position == CommentPosition::Leading
                && jsx.pure_positions.contains(&comment.attached_to)
                && ast.get_utf8(comment.value(ast.source().as_bytes())) == "#__PURE__")
        })
        .collect::<Vec<_>>();
    ast.set_attached_comments(swc_next_allocator::vec::ArenaVec::from_iter_in(
        comments,
        ast.allocator(),
    ));
    let lang = match ast.lang() {
        Lang::Jsx => Lang::Js,
        Lang::Tsx => Lang::Ts,
        lang => lang,
    };
    ast.set_source_metadata(ast.source_type(), lang);
    diagnostics
}

struct Jsx<'r> {
    resolver: &'r mut Resolver,
    runtime: JsxRuntime,
    import_source: String,
    import_jsx: Option<BindingIdentifier>,
    import_jsxs: Option<BindingIdentifier>,
    import_fragment: Option<BindingIdentifier>,
    import_create_element: Option<BindingIdentifier>,
    pragma: String,
    pragma_frag: String,
    development: bool,
    throw_if_namespace: bool,
    filename: String,
    source: SourceLocations,
    self_ctx: JsxSelfCtx,
    method_functions: HashSet<NodeId>,
    comments: HashMap<u32, Vec<Comment>>,
    diagnostics: Vec<Diagnostic>,
    top_level_node: bool,
    pure_positions: HashSet<u32>,
}

impl Jsx<'_> {
    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_common/src/comments.rs#L502>.
    fn add_pure_comment(&mut self, span: Span) {
        self.pure_positions.insert(span.start);
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx_self.rs#L176>.
    fn add_attribute(&mut self, ast: &mut Ast<'_>, name: &str, value: Expr) -> JsxAttributeItem {
        let name = ast.add_utf8(name);
        let name = JsxAttributeName::JsxIdentifier(ast.jsx_identifier(DUMMY_SP, name));
        let value = ast.jsx_expression_container(DUMMY_SP, JsxExpression::Expr(value));
        JsxAttributeItem::JsxAttribute(ast.jsx_attribute(
            DUMMY_SP,
            name,
            Some(JsxAttributeValue::JsxExpressionContainer(value)),
        ))
    }
}

impl<'a> TraverseMut<'a> for Jsx<'_> {
    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1330>.
    fn enter_program(
        &mut self,
        program: Program,
        ctx: &mut TraverseMutCtx<'_, 'a>,
    ) -> TraverseAction<Program> {
        self.parse_directives(ctx.ast, program.span(ctx.ast).start);
        // Directives are statements in SWC and a separate list in Next.
        let offsets = program
            .directives(ctx.ast)
            .iter()
            .map(|id| ctx.ast.get_node_in_sub_range(id).span(ctx.ast).start)
            .chain(
                program
                    .body(ctx.ast)
                    .iter()
                    .map(|id| ctx.ast.get_node_in_sub_range(id).span(ctx.ast).start),
            )
            .collect::<Vec<_>>();
        for offset in offsets {
            if self.parse_directives(ctx.ast, offset) {
                break;
            }
        }
        TraverseAction::Continue
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1341>.
    fn exit_program(
        &mut self,
        program: Program,
        ctx: &mut TraverseMutCtx<'_, 'a>,
    ) -> TraverseAction<Program> {
        self.inject_runtime(program, ctx.ast);
        self.resolver.register(ctx.ast);
        TraverseAction::Continue
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1211>.
    fn enter_class(
        &mut self,
        node: Class,
        ctx: &mut TraverseMutCtx<'_, 'a>,
    ) -> TraverseAction<Class> {
        self.self_ctx
            .enter_class(node.super_class(ctx.ast).is_some());
        TraverseAction::Continue
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1215>.
    fn exit_class(
        &mut self,
        _node: Class,
        _ctx: &mut TraverseMutCtx<'_, 'a>,
    ) -> TraverseAction<Class> {
        self.self_ctx.pop();
        TraverseAction::Continue
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1259>.
    fn enter_method_definition(
        &mut self,
        node: MethodDefinition,
        ctx: &mut TraverseMutCtx<'_, 'a>,
    ) -> TraverseAction<MethodDefinition> {
        self.method_functions.insert(node.value(ctx.ast).node_id());
        if node.kind(ctx.ast) == MethodDefinitionKind::Constructor {
            self.self_ctx.enter_constructor();
        } else {
            self.self_ctx.enter_non_constructor_scope();
        }
        TraverseAction::Continue
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1263>.
    fn exit_method_definition(
        &mut self,
        _node: MethodDefinition,
        _ctx: &mut TraverseMutCtx<'_, 'a>,
    ) -> TraverseAction<MethodDefinition> {
        self.self_ctx.pop();
        TraverseAction::Continue
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1251>.
    fn enter_object_property(
        &mut self,
        node: ObjectProperty,
        ctx: &mut TraverseMutCtx<'_, 'a>,
    ) -> TraverseAction<ObjectProperty> {
        if node.method(ctx.ast) || node.kind(ctx.ast) != PropertyKind::Init {
            self.method_functions.insert(node.value(ctx.ast).node_id());
            self.self_ctx.enter_non_constructor_scope();
        }
        TraverseAction::Continue
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1255>.
    fn exit_object_property(
        &mut self,
        node: ObjectProperty,
        ctx: &mut TraverseMutCtx<'_, 'a>,
    ) -> TraverseAction<ObjectProperty> {
        if node.method(ctx.ast) || node.kind(ctx.ast) != PropertyKind::Init {
            self.self_ctx.pop();
        }
        TraverseAction::Continue
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1227>.
    fn enter_function(
        &mut self,
        node: Function,
        _ctx: &mut TraverseMutCtx<'_, 'a>,
    ) -> TraverseAction<Function> {
        // Next wraps methods in Function nodes; SWC hooks the method wrapper.
        if !self.method_functions.contains(&node.node_id()) {
            self.self_ctx.enter_non_constructor_scope();
        }
        TraverseAction::Continue
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1231>.
    fn exit_function(
        &mut self,
        node: Function,
        _ctx: &mut TraverseMutCtx<'_, 'a>,
    ) -> TraverseAction<Function> {
        if !self.method_functions.remove(&node.node_id()) {
            self.self_ctx.pop();
        }
        TraverseAction::Continue
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1283>.
    fn enter_static_block(
        &mut self,
        _node: StaticBlock,
        _ctx: &mut TraverseMutCtx<'_, 'a>,
    ) -> TraverseAction<StaticBlock> {
        self.self_ctx.enter_non_constructor_scope();
        TraverseAction::Continue
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1287>.
    fn exit_static_block(
        &mut self,
        _node: StaticBlock,
        _ctx: &mut TraverseMutCtx<'_, 'a>,
    ) -> TraverseAction<StaticBlock> {
        self.self_ctx.pop();
        TraverseAction::Continue
    }

    /// SWC source hook: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx_src.rs#L18>.
    /// SWC self hook: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx_self.rs#L176>.
    fn enter_jsx_opening_element(
        &mut self,
        node: JsxOpeningElement,
        ctx: &mut TraverseMutCtx<'_, 'a>,
    ) -> TraverseAction<JsxOpeningElement> {
        if !self.development {
            return TraverseAction::Continue;
        }
        let ast = &mut *ctx.ast;
        let mut attributes = node
            .attributes(ast)
            .iter()
            .map(|id| ast.get_node_in_sub_range(id))
            .collect::<Vec<_>>();
        if node.span(ast) != DUMMY_SP {
            let source = self.source.expression(ast, node.span(ast), &self.filename);
            attributes.push(self.add_attribute(ast, "__source", source));
        }
        if self.self_ctx.can_add_self() {
            let value = ast.expr_this_expression(DUMMY_SP);
            attributes.push(self.add_attribute(ast, "__self", value));
        }
        let attributes = ast.add_sub_range(attributes);
        node.set_attributes(ast, attributes);
        self.resolver.register(ast);
        TraverseAction::Continue
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1297>.
    fn exit_expr(
        &mut self,
        expression: Expr,
        ctx: &mut TraverseMutCtx<'_, 'a>,
    ) -> TraverseAction<Expr> {
        let top_level_node = self.top_level_node;
        let replacement = match ctx.ast.expr_data(expression) {
            ExprData::JsxElement(element) => Some(self.jsx_elem_to_expr(element, ctx.ast)),
            ExprData::JsxFragment(fragment) => Some(self.jsx_frag_to_expr(fragment, ctx.ast)),
            ExprData::ParenthesizedExpression(paren) => match ctx
                .ast
                .expr_data(paren.expression(ctx.ast))
            {
                ExprData::JsxElement(element) => Some(self.jsx_elem_to_expr(element, ctx.ast)),
                ExprData::JsxFragment(fragment) => Some(self.jsx_frag_to_expr(fragment, ctx.ast)),
                _ => None,
            },
            _ => None,
        };
        if replacement.is_some() {
            self.top_level_node = false;
        }
        self.top_level_node = top_level_node;
        if let Some(replacement) = replacement {
            self.resolver.register(ctx.ast);
            TraverseAction::ReplaceSkip(replacement)
        } else {
            TraverseAction::Continue
        }
    }
}
