//! SWC's JSX transform on the Next AST.

use std::collections::{HashMap, HashSet};
use swc_next_ecma_ast::*;

use crate::common::resolver::context::Resolver;

mod display_name;
mod jsx_self;
mod jsx_src;
mod parser;
mod pure_annotations;
mod refresh;
mod static_check;

pub use self::{Options as JsxOptions, Runtime as JsxRuntime};
pub use refresh::options::Options as ReactRefreshOptions;

use jsx_self::JsxSelfCtx;
use jsx_src::JsxSrc;
use parser::jsx::decode_jsx_entity;
use static_check::should_use_create_element;
use swc_next_allocator::wtf8::{Wtf8, Wtf8Buf};

/// SWC's React JSX runtime selection. The default runtime is classic.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Runtime {
    Automatic,
    #[default]
    Classic,
    /// Keep JSX syntax, matching SWC's compiler-level `preserve` mode.
    Preserve,
}

/// SWC's `swc_ecma_transforms_react::Options`.
///
/// Optional values preserve SWC's distinction between unset and explicitly set
/// configuration. An unset runtime selects [`JsxRuntime::Classic`]; import source
/// defaults to `react`, pragmas to `React.createElement` / `React.Fragment`,
/// `throw_if_namespace` to `true`, and `development` to `false`.
///
/// The React preset includes display names and purity annotations. Development
/// mode also enables source/self metadata; these are not independent SWC options.
/// Text options use owned strings instead of legacy SWC's atom/byte-string types.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Options {
    /// SWC compatibility flag, currently unused by the reference React lowering.
    pub next: Option<bool>,
    /// `None` uses the classic runtime.
    pub runtime: Option<JsxRuntime>,
    /// Module supplying the automatic JSX runtime.
    pub import_source: Option<String>,
    /// Dotted JSX factory path for the classic runtime.
    pub pragma: Option<String>,
    /// Dotted fragment factory path for the classic runtime.
    pub pragma_frag: Option<String>,
    /// Report XML namespace syntax in JSX. Defaults to `true` when unset.
    pub throw_if_namespace: Option<bool>,
    /// Select development JSX output and source/self metadata.
    pub development: Option<bool>,
    /// Legacy SWC compatibility option; ignored by its current JSX lowering.
    pub use_builtins: Option<bool>,
    /// Legacy SWC compatibility option; ignored by its current JSX lowering.
    pub use_spread: Option<bool>,
    /// Configure React Refresh; `None` disables it. SWC applies it in development
    /// mode only.
    pub refresh: Option<ReactRefreshOptions>,
}

/// SWC JSX hook construction: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L212>.
/// SWC React hook composition: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/lib.rs#L79>.
pub(crate) fn jsx(
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
        if !is_valid_for_pragma(value) {
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
        source: JsxSrc::new(ast.source()),
        self_ctx: JsxSelfCtx::default(),
        method_functions: HashSet::default(),
        comments,
        diagnostics,
        top_level_node: true,
        pure_positions: HashSet::new(),
    };
    let program = ast.root_program();
    let _ = program.traverse_mut_with(&mut jsx, &mut TraverseMutCtx::new(ast, ()));
    display_name::hook(ast, jsx.resolver);
    pure_annotations::hook(ast, jsx.resolver, &mut jsx.pure_positions);
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
    source: JsxSrc,
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
            let source = self
                .source
                .jsx_dev_source_arg(ast, node.span(ast), &self.filename);
            attributes.push(self.add_attribute(ast, "__source", source));
        }
        if self.self_ctx.can_add_self() {
            let value = ast.expr_this_expression(DUMMY_SP);
            attributes.push(self.add_attribute(ast, "__self", value));
        }
        let attributes = ast.add_sub_range(attributes);
        node.set_attributes(ast, attributes);
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
            TraverseAction::ReplaceSkip(replacement)
        } else {
            TraverseAction::Continue
        }
    }
}

/// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1472>.
pub(crate) fn string(ast: &mut Ast<'_>, span: Span, value: &str) -> Expr {
    let value = ast.add_utf8(value);
    ast.expr_string_literal(span, value.into(), AstUtf8::EMPTY)
}

/// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1506>.
pub(super) fn member(ast: &mut Ast<'_>, object: Expr, property: &str, span: Span) -> Expr {
    let name = ast.add_utf8(property);
    let property = ast.identifier_name(span, name);
    ast.expr_member_expression(
        span,
        object,
        PropertyKey::IdentifierName(property),
        false,
        false,
    )
}

/// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L499>.
pub(crate) fn property(ast: &mut Ast<'_>, name: &str, value: Expr) -> ObjectPropertyKind {
    let name = ast.add_utf8(name);
    let key = PropertyKey::IdentifierName(ast.identifier_name(DUMMY_SP, name));
    key_value(ast, key, value)
}

/// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L840>.
pub(super) fn key_value(ast: &mut Ast<'_>, key: PropertyKey, value: Expr) -> ObjectPropertyKind {
    ObjectPropertyKind::ObjectProperty(ast.object_property(
        DUMMY_SP,
        key,
        value,
        PropertyKind::Init,
        false,
        false,
        false,
    ))
}

/// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L628>.
pub(crate) fn object(ast: &mut Ast<'_>, properties: Vec<ObjectPropertyKind>) -> Expr {
    let properties = ast.add_sub_range(properties);
    ast.expr_object_expression(DUMMY_SP, properties)
}

/// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L657>.
pub(super) fn array(ast: &mut Ast<'_>, arguments: Vec<Argument>) -> Expr {
    let elements = arguments.into_iter().map(Some);
    let elements = ast.add_sub_range(elements);
    ast.expr_array_expression(DUMMY_SP, elements)
}

/// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L692>.
pub(super) fn call(ast: &mut Ast<'_>, span: Span, callee: Expr, arguments: Vec<Argument>) -> Expr {
    let arguments = ast.add_sub_range(arguments);
    ast.expr_call_expression(span, callee, None, arguments, false, true)
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_ast/src/expr.rs#L179>.
pub(crate) fn undefined(ast: &mut Ast<'_>) -> Expr {
    let zero = ast.expr_numeric_literal(DUMMY_SP, 0.0, None);
    ast.expr_unary_expression(DUMMY_SP, zero, UnaryOperator::Void)
}

impl Jsx<'_> {
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L610>.
    pub(super) fn jsx_frag_to_expr(&mut self, fragment: JsxFragment, ast: &mut Ast<'_>) -> Expr {
        let span = fragment.span(ast);
        self.add_pure_comment(span);
        match self.runtime {
            JsxRuntime::Automatic => {
                let binding = *self
                    .import_fragment
                    .get_or_insert_with(|| self.resolver.private_binding(ast, "_Fragment"));
                let name = self.resolver.reference(ast, binding);
                let mut properties = Vec::new();
                let mut children = self.children(fragment.children(ast), ast);
                let use_jsxs = match children.len() {
                    0 => false,
                    1 if children[0].is_expr(ast) => {
                        let child = children.pop().unwrap().as_expr(ast).unwrap();
                        properties.push(property(ast, "children", child));
                        false
                    }
                    _ => {
                        let children = array(ast, children);
                        properties.push(property(ast, "children", children));
                        true
                    }
                };
                let jsx = self.jsx_binding(use_jsxs, false, ast);
                let callee = self.resolver.reference(ast, jsx);
                let props = object(ast, properties);
                let mut args = vec![Argument::Expr(name), Argument::Expr(props)];
                if self.development {
                    args.push(Argument::Expr(undefined(ast)));
                    args.push(Argument::Expr(ast.expr_boolean_literal(DUMMY_SP, use_jsxs)));
                    args.push(Argument::Expr(self.source.jsx_dev_source_arg(
                        ast,
                        fragment.opening_fragment(ast).span(ast),
                        &self.filename,
                    )));
                    let this = if self.self_ctx.can_add_self() {
                        ast.expr_this_expression(DUMMY_SP)
                    } else {
                        undefined(ast)
                    };
                    args.push(Argument::Expr(this));
                }
                call(ast, span, callee, args)
            }
            JsxRuntime::Classic => {
                let name = parse_expr_for_jsx(ast, self.resolver, &self.pragma_frag);
                let props = ast.expr_null_literal(DUMMY_SP);
                let mut args = vec![Argument::Expr(name), Argument::Expr(props)];
                args.extend(self.children(fragment.children(ast), ast));
                let callee = parse_expr_for_jsx(ast, self.resolver, &self.pragma);
                call(ast, span, callee, args)
            }
            JsxRuntime::Preserve => unreachable!(),
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L734>.
    pub(super) fn jsx_elem_to_expr(&mut self, element: JsxElement, ast: &mut Ast<'_>) -> Expr {
        let top_level_node = self.top_level_node;
        let span = element.span(ast);
        let opening = element.opening_element(ast);
        let attributes = opening
            .attributes(ast)
            .iter()
            .map(|id| ast.get_node_in_sub_range(id))
            .collect::<Vec<_>>();
        let use_create_element = should_use_create_element(&attributes, ast);
        self.top_level_node = false;
        let name = self.jsx_name(opening.name(ast), ast);
        self.add_pure_comment(span);
        match self.runtime {
            JsxRuntime::Automatic => {
                let mut properties = Vec::new();
                let mut key = None;
                let mut source = None;
                let mut this = None;
                for attribute in attributes {
                    match ast.jsx_attribute_item_data(attribute) {
                        JsxAttributeItemData::JsxAttribute(attribute) => {
                            match ast.jsx_attribute_name_data(attribute.name(ast)) {
                                JsxAttributeNameData::JsxIdentifier(identifier) => {
                                    let attr_name = ast.get_utf8(identifier.name(ast));
                                    if !use_create_element && attr_name == "key" {
                                        key = attribute.value(ast).and_then(|value| {
                                            self.jsx_attr_value_to_expr(value, ast)
                                        });
                                        if key.is_none() {
                                            self.diagnostics.push(Diagnostic::error(
                                                "The value of property 'key' should not be empty",
                                                identifier.span(ast),
                                            ));
                                        }
                                        continue;
                                    }
                                    if !use_create_element
                                        && attr_name == "__source"
                                        && self.development
                                    {
                                        assert!(source.is_none(), "Duplicate __source is found");
                                        source = attribute.value(ast).and_then(|value| {
                                            self.jsx_attr_value_to_expr(value, ast)
                                        });
                                        assert!(
                                            source.is_some(),
                                            "value of property '__source' should not be empty"
                                        );
                                        continue;
                                    }
                                    if !use_create_element
                                        && attr_name == "__self"
                                        && self.development
                                    {
                                        assert!(this.is_none(), "Duplicate __self is found");
                                        this = attribute.value(ast).and_then(|value| {
                                            self.jsx_attr_value_to_expr(value, ast)
                                        });
                                        assert!(
                                            this.is_some(),
                                            "value of property '__self' should not be empty"
                                        );
                                        continue;
                                    }
                                    let value = self.process_attr_value(attribute.value(ast), ast);
                                    let key = self.to_prop_name(attribute.name(ast), ast);
                                    properties.push(key_value(ast, key, value));
                                }
                                JsxAttributeNameData::JsxNamespacedName(name) => {
                                    self.namespace_diagnostic(span);
                                    let value = self.process_attr_value(attribute.value(ast), ast);
                                    let name = format!(
                                        "{}:{}",
                                        ast.get_utf8(name.namespace(ast).name(ast)),
                                        ast.get_utf8(name.name(ast).name(ast))
                                    );
                                    let key = PropertyKey::Expr(string(ast, span, &name));
                                    properties.push(key_value(ast, key, value));
                                }
                            }
                        }
                        JsxAttributeItemData::JsxSpreadAttribute(spread) => {
                            self.spread_properties(spread, &mut properties, ast)
                        }
                    }
                }
                let mut children = self.children(element.children(ast), ast);
                let use_jsxs = match children.len() {
                    0 => false,
                    1 if children[0].is_expr(ast) => {
                        if !use_create_element {
                            let child = children.pop().unwrap().as_expr(ast).unwrap();
                            properties.push(property(ast, "children", child));
                        }
                        false
                    }
                    _ => {
                        let children = array(ast, std::mem::take(&mut children));
                        properties.push(property(ast, "children", children));
                        true
                    }
                };
                let binding = self.jsx_binding(use_jsxs, use_create_element, ast);
                self.top_level_node = top_level_node;
                let callee = self.resolver.reference(ast, binding);
                let props = object(ast, properties);
                let mut args = vec![Argument::Expr(name), Argument::Expr(props)];
                if use_create_element {
                    args.extend(children);
                } else if self.development {
                    args.push(Argument::Expr(key.unwrap_or_else(|| undefined(ast))));
                    args.push(Argument::Expr(ast.expr_boolean_literal(DUMMY_SP, use_jsxs)));
                    args.push(Argument::Expr(source.unwrap_or_else(|| undefined(ast))));
                    args.push(Argument::Expr(this.unwrap_or_else(|| undefined(ast))));
                } else if let Some(key) = key {
                    args.push(Argument::Expr(key));
                }
                call(ast, span, callee, args)
            }
            JsxRuntime::Classic => {
                let props = self.fold_attrs_for_classic(attributes, ast);
                let mut args = vec![Argument::Expr(name), Argument::Expr(props)];
                args.extend(self.children(element.children(ast), ast));
                let callee = parse_expr_for_jsx(ast, self.resolver, &self.pragma);
                call(ast, span, callee, args)
            }
            JsxRuntime::Preserve => unreachable!(),
        }
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1026>.
    fn children(&mut self, children: SubRange<JsxChild>, ast: &mut Ast<'_>) -> Vec<Argument> {
        let children = children
            .iter()
            .map(|id| ast.get_node_in_sub_range(id))
            .collect::<Vec<_>>();
        children
            .into_iter()
            .filter_map(|child| self.jsx_elem_child_to_expr(child, ast))
            .collect()
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1026>.
    fn jsx_elem_child_to_expr(&mut self, child: JsxChild, ast: &mut Ast<'_>) -> Option<Argument> {
        self.top_level_node = false;
        Some(match ast.jsx_child_data(child) {
            JsxChildData::JsxText(node) => {
                let raw = ast.get_utf8(node.value(ast));
                let decoded = decode_entities(raw);
                let value = jsx_text_to_str_with_raw(&decoded, raw);
                if value.is_empty() {
                    return None;
                }
                let value = ast.add_wtf8(&value);
                Argument::Expr(ast.expr_string_literal(node.span(ast), value, AstUtf8::EMPTY))
            }
            JsxChildData::JsxExpressionContainer(container) => {
                match ast.jsx_expression_data(container.expression(ast)) {
                    JsxExpressionData::Expr(expression) => Argument::Expr(expression),
                    JsxExpressionData::JsxEmptyExpression(_) => return None,
                }
            }
            JsxChildData::JsxElement(element) => {
                Argument::Expr(self.jsx_elem_to_expr(element, ast))
            }
            JsxChildData::JsxFragment(fragment) => {
                Argument::Expr(self.jsx_frag_to_expr(fragment, ast))
            }
            JsxChildData::JsxSpreadChild(spread) => Argument::SpreadElement(
                ast.spread_element(spread.span(ast), spread.expression(ast)),
            ),
        })
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1064>.
    fn fold_attrs_for_classic(
        &mut self,
        attributes: Vec<JsxAttributeItem>,
        ast: &mut Ast<'_>,
    ) -> Expr {
        if attributes.is_empty() {
            return ast.expr_null_literal(DUMMY_SP);
        }
        let count = attributes.len();
        let mut properties = Vec::new();
        for attribute in attributes {
            match ast.jsx_attribute_item_data(attribute) {
                JsxAttributeItemData::JsxAttribute(attribute) => {
                    properties.push(self.attr_to_prop(attribute, ast));
                }
                JsxAttributeItemData::JsxSpreadAttribute(spread) => {
                    if count == 1 {
                        return spread.argument(ast);
                    }
                    self.spread_properties(spread, &mut properties, ast);
                }
            }
        }
        object(ast, properties)
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1099>.
    fn attr_to_prop(&mut self, attribute: JsxAttribute, ast: &mut Ast<'_>) -> ObjectPropertyKind {
        let key = self.to_prop_name(attribute.name(ast), ast);
        let value = if let Some(value) = attribute.value(ast) {
            self.process_attr_value(Some(value), ast)
        } else {
            ast.expr_boolean_literal(key.span(ast), true)
        };
        key_value(ast, key, value)
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L881>.
    fn spread_properties(
        &mut self,
        spread: JsxSpreadAttribute,
        properties: &mut Vec<ObjectPropertyKind>,
        ast: &mut Ast<'_>,
    ) {
        let expression = spread.argument(ast);
        if let Some(object) = expression.as_object_expression(ast) {
            properties.extend(
                object
                    .properties(ast)
                    .iter()
                    .map(|id| ast.get_node_in_sub_range(id)),
            );
        } else {
            properties.push(ObjectPropertyKind::SpreadElement(
                ast.spread_element(spread.span(ast), expression),
            ));
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L539>.
    fn process_attr_value(&mut self, value: Option<JsxAttributeValue>, ast: &mut Ast<'_>) -> Expr {
        match value.map(|value| ast.jsx_attribute_value_data(value)) {
            Some(JsxAttributeValueData::JsxElement(element)) => self.jsx_elem_to_expr(element, ast),
            Some(JsxAttributeValueData::JsxFragment(fragment)) => {
                self.jsx_frag_to_expr(fragment, ast)
            }
            Some(_) => self
                .jsx_attr_value_to_expr(value.unwrap(), ast)
                .expect("empty expression container"),
            None => ast.expr_boolean_literal(DUMMY_SP, true),
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1967>.
    fn jsx_attr_value_to_expr(
        &mut self,
        value: JsxAttributeValue,
        ast: &mut Ast<'_>,
    ) -> Option<Expr> {
        match ast.jsx_attribute_value_data(value) {
            JsxAttributeValueData::StringLiteral(literal) => {
                // Next stores quoted JSX text without decoding its entities.
                let raw = ast.get_wtf8(literal.value(ast)).as_str().unwrap();
                let value = decode_attribute(raw);
                let value = ast.add_wtf8(&value);
                Some(ast.expr_string_literal(literal.span(ast), value, AstUtf8::EMPTY))
            }
            JsxAttributeValueData::JsxExpressionContainer(container) => {
                match ast.jsx_expression_data(container.expression(ast)) {
                    JsxExpressionData::Expr(expression) => Some(expression),
                    JsxExpressionData::JsxEmptyExpression(_) => None,
                }
            }
            JsxAttributeValueData::JsxElement(element) => Some(Expr::JsxElement(element)),
            JsxAttributeValueData::JsxFragment(fragment) => Some(Expr::JsxFragment(fragment)),
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1541>.
    fn to_prop_name(&mut self, name: JsxAttributeName, ast: &mut Ast<'_>) -> PropertyKey {
        match ast.jsx_attribute_name_data(name) {
            JsxAttributeNameData::JsxIdentifier(identifier) => {
                let name = identifier.name(ast);
                if ast.get_utf8(name).contains('-') {
                    PropertyKey::Expr(ast.expr_string_literal(
                        identifier.span(ast),
                        name.into(),
                        AstUtf8::EMPTY,
                    ))
                } else {
                    PropertyKey::IdentifierName(ast.identifier_name(identifier.span(ast), name))
                }
            }
            JsxAttributeNameData::JsxNamespacedName(name) => {
                let value = format!(
                    "{}:{}",
                    ast.get_utf8(name.namespace(ast).name(ast)),
                    ast.get_utf8(name.name(ast).name(ast))
                );
                PropertyKey::Expr(string(ast, name.span(ast), &value))
            }
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1459>.
    fn jsx_name(&mut self, name: JsxElementName, ast: &mut Ast<'_>) -> Expr {
        match ast.jsx_element_name_data(name) {
            JsxElementNameData::JsxIdentifier(identifier) => {
                let name = ast.get_utf8(identifier.name(ast));
                if name == "this" {
                    ast.expr_this_expression(identifier.span(ast))
                } else if name.starts_with(|character: char| character.is_ascii_lowercase()) {
                    string(ast, identifier.span(ast), name)
                } else {
                    self.resolver.source_reference(ast, identifier)
                }
            }
            JsxElementNameData::JsxNamespacedName(name) => {
                self.namespace_diagnostic(name.span(ast));
                let value = format!(
                    "{}:{}",
                    ast.get_utf8(name.namespace(ast).name(ast)),
                    ast.get_utf8(name.name(ast).name(ast))
                );
                string(ast, name.span(ast), &value)
            }
            JsxElementNameData::JsxMemberExpression(member) => self.jsx_member(member, ast),
        }
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1506>.
    fn jsx_member(&mut self, node: JsxMemberExpression, ast: &mut Ast<'_>) -> Expr {
        let object = match ast.jsx_member_expression_object_data(node.object(ast)) {
            JsxMemberExpressionObjectData::JsxIdentifier(identifier) => {
                if ast.get_utf8(identifier.name(ast)) == "this" {
                    ast.expr_this_expression(identifier.span(ast))
                } else {
                    self.resolver.source_reference(ast, identifier)
                }
            }
            JsxMemberExpressionObjectData::JsxMemberExpression(member) => {
                self.jsx_member(member, ast)
            }
        };
        let name = node.property(ast).name(ast);
        let property = ast.identifier_name(node.property(ast).span(ast), name);
        ast.expr_member_expression(
            node.span(ast),
            object,
            PropertyKey::IdentifierName(property),
            false,
            false,
        )
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1483>.
    fn namespace_diagnostic(&mut self, span: Span) {
        if self.throw_if_namespace {
            self.diagnostics.push(Diagnostic::error("JSX Namespace is disabled by default because react does not support it yet. You can specify jsc.transform.react.throwIfNamespace to false to override default behavior", span));
        }
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L936>.
    fn jsx_binding(
        &mut self,
        use_jsxs: bool,
        use_create_element: bool,
        ast: &mut Ast<'_>,
    ) -> BindingIdentifier {
        let (binding, name) = if use_create_element {
            (&mut self.import_create_element, "_createElement")
        } else if use_jsxs && !self.development {
            (&mut self.import_jsxs, "_jsxs")
        } else {
            (
                &mut self.import_jsx,
                if self.development { "_jsxDEV" } else { "_jsx" },
            )
        };
        *binding.get_or_insert_with(|| self.resolver.private_binding(ast, name))
    }
}

/// SWC comment validation; dotted paths are the agreed option restriction: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L475>.
pub(super) fn is_valid_for_pragma(value: &str) -> bool {
    value.split('.').all(|part| is_identifier(part.as_bytes()))
}

/// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L143>.
pub(super) fn parse_expr_for_jsx(ast: &mut Ast<'_>, resolver: &mut Resolver, value: &str) -> Expr {
    let mut parts = value.split('.');
    let root = parts.next().unwrap();
    let mut result = if root == "this" {
        ast.expr_this_expression(DUMMY_SP)
    } else {
        resolver.marked_reference(ast, root, false)
    };
    for part in parts {
        result = member(ast, result, part, DUMMY_SP);
    }
    result
}

#[derive(Default)]
struct JsxDirectives {
    runtime: Option<Runtime>,
    import_source: Option<String>,
    pragma: Option<(String, Span)>,
    pragma_frag: Option<(String, Span)>,
}

impl JsxDirectives {
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L320>.
    fn from_comments(
        ast: &Ast<'_>,
        comments: &[Comment],
        diagnostics: &mut Vec<Diagnostic>,
    ) -> Self {
        let mut directives = Self::default();
        for comment in comments {
            if comment.kind != CommentKind::Block || comment.position != CommentPosition::Leading {
                continue;
            }
            let text = ast.get_utf8(comment.value(ast.source().as_bytes()));
            for line in text.lines() {
                let line = line.trim();
                let line = line.strip_prefix('*').map_or(line, str::trim);
                if !line.starts_with("@jsx") {
                    continue;
                }
                let mut words = line.split_whitespace();
                while let Some(directive) = words.next() {
                    let value = words.next();
                    match (directive, value) {
                        ("@jsxRuntime", Some("classic")) => {
                            directives.runtime = Some(JsxRuntime::Classic)
                        }
                        ("@jsxRuntime", Some("automatic")) => {
                            directives.runtime = Some(JsxRuntime::Automatic)
                        }
                        ("@jsxRuntime", Some(_)) => diagnostics.push(Diagnostic::error(
                            "Runtime must be either `classic` or `automatic`.",
                            comment.span,
                        )),
                        ("@jsxImportSource", Some(value)) => {
                            directives.runtime = Some(JsxRuntime::Automatic);
                            directives.import_source = Some(value.to_owned());
                        }
                        ("@jsx", Some(value)) if is_valid_for_pragma(value) => {
                            directives.pragma = Some((value.to_owned(), comment.span))
                        }
                        ("@jsxFrag", Some(value)) if is_valid_for_pragma(value) => {
                            directives.pragma_frag = Some((value.to_owned(), comment.span))
                        }
                        _ => {}
                    }
                }
            }
        }
        directives
    }
}

impl Jsx<'_> {
    /// SWC directive application: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1143>.
    pub(super) fn parse_directives(&mut self, ast: &Ast<'_>, offset: u32) -> bool {
        let Some(comments) = self.comments.get(&offset) else {
            return false;
        };
        let JsxDirectives {
            runtime,
            import_source,
            pragma,
            pragma_frag,
        } = JsxDirectives::from_comments(ast, comments, &mut self.diagnostics);
        let found = runtime.is_some()
            || import_source.is_some()
            || pragma.is_some()
            || pragma_frag.is_some();
        if let Some(runtime) = runtime {
            self.runtime = runtime;
        }
        if let Some(import_source) = import_source {
            self.import_source = import_source;
        }
        if let Some((pragma, span)) = pragma {
            if self.runtime == JsxRuntime::Automatic {
                self.diagnostics.push(Diagnostic::error(
                    "pragma cannot be set when runtime is automatic",
                    span,
                ));
            }
            self.pragma = pragma;
        }
        if let Some((pragma, span)) = pragma_frag {
            if self.runtime == JsxRuntime::Automatic {
                self.diagnostics.push(Diagnostic::error(
                    "pragmaFrag cannot be set when runtime is automatic",
                    span,
                ));
            }
            self.pragma_frag = pragma;
        }
        found
    }
}

impl Jsx<'_> {
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L554>.
    pub(super) fn inject_runtime(&mut self, program: Program, ast: &mut Ast<'_>) {
        if self.runtime != JsxRuntime::Automatic {
            return;
        }
        // Each SWC injection prepends, so the runtime precedes createElement.
        if let Some(local) = self.import_create_element.take() {
            let source = self.import_source.clone();
            self.inject(program, vec![(local, "createElement")], &source, ast);
        }
        let mut imports = Vec::new();
        if let Some(local) = self.import_jsx.take() {
            imports.push((local, if self.development { "jsxDEV" } else { "jsx" }));
        }
        if !self.development
            && let Some(local) = self.import_jsxs.take()
        {
            imports.push((local, "jsxs"));
        }
        if let Some(local) = self.import_fragment.take() {
            imports.push((local, "Fragment"));
        }
        if !imports.is_empty() {
            let runtime = if self.development {
                "jsx-dev-runtime"
            } else {
                "jsx-runtime"
            };
            let source = format!("{}/{runtime}", self.import_source);
            self.inject(program, imports, &source, ast);
        }
    }

    /// SWC module injection: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1341>.
    /// SWC script injection: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1400>.
    /// SWC directive placement: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_utils/src/lib.rs#L346>.
    fn inject(
        &mut self,
        program: Program,
        imports: Vec<(BindingIdentifier, &str)>,
        source: &str,
        ast: &mut Ast<'_>,
    ) {
        let statement = if ast.is_module() {
            let specifiers = imports
                .into_iter()
                .map(|(local, imported)| {
                    let imported = ast.add_utf8(imported);
                    let imported =
                        ModuleExportName::IdentifierName(ast.identifier_name(DUMMY_SP, imported));
                    ImportDeclarationSpecifier::ImportSpecifier(ast.import_specifier(
                        DUMMY_SP,
                        imported,
                        local,
                        ImportOrExportKind::Value,
                    ))
                })
                .collect::<Vec<_>>();
            let specifiers = ast.add_sub_range(specifiers);
            let source = ast.add_utf8(source);
            let source = ast.string_literal(DUMMY_SP, source.into(), AstUtf8::EMPTY);
            ast.stmt_import_declaration(
                DUMMY_SP,
                specifiers,
                source,
                SubRange::empty(),
                None,
                ImportOrExportKind::Value,
            )
        } else {
            let properties = imports
                .into_iter()
                .map(|(local, imported)| {
                    let imported = ast.add_utf8(imported);
                    let key = PropertyKey::IdentifierName(ast.identifier_name(DUMMY_SP, imported));
                    ast.binding_property(
                        DUMMY_SP,
                        key,
                        BindingPattern::BindingIdentifier(local),
                        false,
                        false,
                    )
                })
                .collect::<Vec<_>>();
            let properties = ast.add_sub_range(properties);
            let pattern = ast.binding_pattern_object_pattern(DUMMY_SP, properties, None);
            let callee = self.resolver.marked_reference(ast, "require", true);
            let source = string(ast, DUMMY_SP, source);
            let arguments = ast.add_sub_range([Argument::Expr(source)]);
            let init = ast.expr_call_expression(DUMMY_SP, callee, None, arguments, false, false);
            let declarator = ast.variable_declarator(DUMMY_SP, pattern, None, Some(init), false);
            let declarations = ast.add_sub_range([declarator]);
            Stmt::Declaration(Decl::VariableDeclaration(ast.variable_declaration(
                DUMMY_SP,
                VariableKind::Const,
                declarations,
                false,
            )))
        };
        let mut body = vec![statement];
        body.extend(
            program
                .body(ast)
                .iter()
                .map(|id| ast.get_node_in_sub_range(id)),
        );
        let body = ast.add_sub_range(body);
        program.set_body(ast, body);
    }
}

// JSX whitespace handling ported from SWC's React transform.

/// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_parser/src/lexer/state.rs#L607>.
pub(super) fn decode_entities(mut raw: &str) -> Wtf8Buf {
    let mut value = Wtf8Buf::with_capacity(raw.len());
    while let Some(character) = raw.chars().next() {
        if let Some((code_point, len)) = decode_jsx_entity(raw) {
            value.push(code_point);
            raw = &raw[len..];
        } else {
            value.push_char(character);
            raw = &raw[character.len_utf8()..];
        }
    }
    value
}

/// Decode quoted attribute contents with SWC's `read_jsx_str` chunk boundaries.
/// In particular, preserve its CRLF chunk-start behavior instead of applying
/// JSX child whitespace normalization to attribute strings.
///
/// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_parser/src/lexer/mod.rs#L1427>.
pub(super) fn decode_attribute(raw: &str) -> Wtf8Buf {
    let mut value = Wtf8Buf::new();
    let mut chunk_start = 0;
    let mut position = 0;
    while let Some(character) = raw[position..].chars().next() {
        if character == '\\' {
            value.push_str(&raw[chunk_start..position]);
            value.push_char(character);
            position += character.len_utf8();
            chunk_start = position;
        } else if character == '&' {
            value.push_str(&raw[chunk_start..position]);
            if let Some((code_point, len)) = decode_jsx_entity(&raw[position..]) {
                value.push(code_point);
                position += len;
            } else {
                value.push_char('&');
                position += 1;
            }
            chunk_start = position;
        } else if is_line_terminator(character) {
            value.push_str(&raw[chunk_start..position]);
            chunk_start = position + character.len_utf8();
            if raw[position..].starts_with("\r\n") {
                value.push_str("\r\n");
                position += 2;
            } else {
                value.push_char(character);
                position += character.len_utf8();
            }
        } else {
            position += character.len_utf8();
        }
    }
    value.push_str(&raw[chunk_start..]);
    value
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_utils/src/str.rs#L101>.
fn is_line_terminator(character: char) -> bool {
    matches!(character, '\n' | '\r' | '\u{2028}' | '\u{2029}')
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1590>.
pub(super) fn jsx_text_to_str_with_raw(value: &Wtf8, raw: &str) -> Wtf8Buf {
    // Fast path: if no HTML entities (raw == value), use the simple algorithm
    if let Some(value) = value.as_str()
        && value == raw
    {
        return Wtf8Buf::from_string(jsx_text_to_str_impl(value));
    }
    // Build a mask of which code point positions in value came from HTML
    // entities.
    let entity_mask = build_entity_mask(value, raw);

    jsx_text_to_str_with_entity_mask(value, &entity_mask)
}

/// Build a mask indicating which character positions in `value` came from HTML
/// entities in `raw`.
///
/// Returns a Vec<bool> where true means the character at that index was from an
/// entity.
///
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1609>.
fn build_entity_mask(value: &Wtf8, mut raw: &str) -> Vec<bool> {
    let mut mask = Vec::with_capacity(value.code_points().count());
    let mut previous_is_lead_surrogate = false;

    while let Some(raw_char) = raw.chars().next() {
        let entity = if raw_char == '&' {
            decode_jsx_entity(raw)
        } else {
            None
        };
        let Some((code_point, len)) = entity else {
            raw = &raw[raw_char.len_utf8()..];
            mask.push(false);
            previous_is_lead_surrogate = false;
            continue;
        };
        raw = &raw[len..];
        let code_point_value = code_point.to_u32();

        // Wtf8Buf::push folds an adjacent surrogate pair into one code point.
        // Both halves come from entities because raw text is valid UTF-8.
        if !previous_is_lead_surrogate || !matches!(code_point_value, 0xdc00..=0xdfff) {
            mask.push(true);
        }
        previous_is_lead_surrogate = matches!(code_point_value, 0xd800..=0xdbff);
    }

    mask
}

/// JSX text processing with entity mask - preserves whitespace from HTML
/// entities.
///
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1641>.
fn jsx_text_to_str_with_entity_mask(t: &Wtf8, entity_mask: &[bool]) -> Wtf8Buf {
    // Fast path: if no line terminators and no trimmable whitespace
    // (whitespace that's not from entities at the leading edge)
    let chars: Vec<_> = wtf8_chars(t).collect();
    let has_line_terminator = chars.iter().any(|(_, _, c)| is_line_terminator(*c));

    // For single-line text, we keep all whitespace (matching original behavior)
    // The original jsx_text_to_str_impl preserves leading/trailing whitespace on
    // single-line text
    if !t.is_empty() && !has_line_terminator {
        return t.to_owned();
    }

    let mut acc: Option<Wtf8Buf> = None;
    let mut only_line: Option<(usize, usize)> = None;
    let mut line_start: Option<usize> = Some(0);
    let mut line_end: Option<usize> = None;
    // The first line preserves leading whitespace; subsequent lines trim it.
    let mut is_first_line = true;

    for (char_idx, (_, _, c)) in chars.iter().enumerate() {
        let is_from_entity = *entity_mask.get(char_idx).unwrap_or(&false);

        if is_line_terminator(*c) {
            // Process current line - trim both leading AND trailing (intermediate
            // line)
            if let (Some(start), Some(end)) = (line_start, line_end) {
                let line_range =
                    extract_line_content(&chars, start, end, entity_mask, !is_first_line, true);
                if let Some((line_start, line_end)) = line_range {
                    add_line_of_jsx_text_wtf8(line_start, line_end, t, &mut acc, &mut only_line);
                }
            }
            is_first_line = false;
            line_start = None;
            line_end = None;
        } else if !is_white_space_single_line(*c) || is_from_entity {
            // Non-whitespace or entity-derived whitespace - counts as content
            line_end = Some(char_idx + 1);
            if line_start.is_none() {
                line_start = Some(char_idx);
            }
        }
    }

    // Handle final line. Leading whitespace is preserved only if this is still
    // the first line (single-line input).
    if let Some(start) = line_start {
        let line_range = extract_line_content(
            &chars,
            start,
            chars.len(),
            entity_mask,
            !is_first_line,
            false,
        );
        if let Some((line_start, line_end)) = line_range {
            add_line_of_jsx_text_wtf8(line_start, line_end, t, &mut acc, &mut only_line);
        }
    }

    if let Some(acc) = acc {
        acc
    } else if let Some((start, end)) = only_line {
        t.slice(start, end).to_owned()
    } else {
        Wtf8Buf::default()
    }
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1711>.
fn wtf8_chars(t: &Wtf8) -> impl Iterator<Item = (usize, usize, char)> + '_ {
    let mut byte_pos = 0;

    t.code_points().map(move |cp| {
        let start = byte_pos;
        let cp_value = cp.to_u32();
        let cp_byte_len = if cp_value < 0x80 {
            1
        } else if cp_value < 0x800 {
            2
        } else if cp_value < 0x10000 {
            3
        } else {
            4
        };
        byte_pos += cp_byte_len;

        (start, byte_pos, cp.to_char_lossy())
    })
}

/// Extract line content, optionally trimming non-entity whitespace from edges
///
/// - `trim_leading`: if true, trim leading non-entity whitespace
/// - `trim_trailing`: if true, trim trailing non-entity whitespace
///
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1736>.
fn extract_line_content(
    chars: &[(usize, usize, char)],
    start: usize,
    end: usize,
    entity_mask: &[bool],
    trim_leading: bool,
    trim_trailing: bool,
) -> Option<(usize, usize)> {
    // Find first non-trimmable position (if trim_leading is true)
    let mut actual_start = start;
    if trim_leading {
        while actual_start < end {
            let c = chars[actual_start].2;
            let is_from_entity = *entity_mask.get(actual_start).unwrap_or(&false);
            if !is_white_space_single_line(c) || is_from_entity {
                break;
            }
            actual_start += 1;
        }
    }

    // Find last non-trimmable position (if trim_trailing is true)
    let mut actual_end = end;
    if trim_trailing {
        while actual_end > actual_start {
            let c = chars[actual_end - 1].2;
            let is_from_entity = *entity_mask.get(actual_end - 1).unwrap_or(&false);
            if !is_white_space_single_line(c) || is_from_entity {
                break;
            }
            actual_end -= 1;
        }
    }

    if actual_start == actual_end {
        None
    } else {
        Some((chars[actual_start].0, chars[actual_end - 1].1))
    }
}

/// Helper for adding lines of JSX text when handling Wtf8 with surrogates
///
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1847>.
fn add_line_of_jsx_text_wtf8(
    line_start: usize,
    line_end: usize,
    source: &Wtf8,
    acc: &mut Option<Wtf8Buf>,
    only_line: &mut Option<(usize, usize)>,
) {
    if let Some((only_start, only_end)) = only_line.take() {
        // Second line - create accumulator
        let mut buffer = Wtf8Buf::with_capacity(source.len());
        buffer.push_wtf8(source.slice(only_start, only_end));
        buffer.push_str(" ");
        buffer.push_wtf8(source.slice(line_start, line_end));
        *acc = Some(buffer);
    } else if let Some(buffer) = acc {
        // Subsequent lines
        buffer.push_str(" ");
        buffer.push_wtf8(source.slice(line_start, line_end));
    } else {
        // First line
        *only_line = Some((line_start, line_end));
    }
}

/// Internal implementation that works with &str
#[inline]
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1873>.
fn jsx_text_to_str_impl(t: &str) -> String {
    // Fast path: if no line terminators and no leading/trailing whitespace
    if !t.is_empty()
        && !t.chars().any(is_line_terminator)
        && !t.starts_with(is_white_space_single_line)
        && !t.ends_with(is_white_space_single_line)
    {
        return t.to_owned();
    }

    let mut acc: Option<String> = None;
    let mut only_line: Option<&str> = None;
    let mut first_non_whitespace: Option<usize> = Some(0);
    let mut last_non_whitespace: Option<usize> = None;

    for (index, c) in t.char_indices() {
        if is_line_terminator(c) {
            if let (Some(first), Some(last)) = (first_non_whitespace, last_non_whitespace) {
                let line_text = &t[first..last];
                add_line_of_jsx_text(line_text, &mut acc, &mut only_line);
            }
            first_non_whitespace = None;
        } else if !is_white_space_single_line(c) {
            last_non_whitespace = Some(index + c.len_utf8());
            if first_non_whitespace.is_none() {
                first_non_whitespace.replace(index);
            }
        }
    }

    if let Some(first) = first_non_whitespace {
        let line_text = &t[first..];
        add_line_of_jsx_text(line_text, &mut acc, &mut only_line);
    }

    if let Some(acc) = acc {
        acc
    } else if let Some(only_line) = only_line {
        only_line.into()
    } else {
        "".into()
    }
}

/// [TODO]: Re-validate this whitespace handling logic.
///
/// We cannot use [swc_ecma_utils::str::is_white_space_single_line] because
/// HTML entities (like `&nbsp;` → `\u{00a0}`) are pre-processed by the parser,
/// making it impossible to distinguish them from literal Unicode characters. We
/// should never trim HTML entities.
///
/// As a reference, Babel only trims regular spaces and tabs, so this is a
/// simplified implementation already in use.
/// https://github.com/babel/babel/blob/e5c8dc7330cb2f66c37637677609df90b31ff0de/packages/babel-types/src/utils/react/cleanJSXElementLiteralChild.ts#L28-L39
///
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1927>.
fn is_white_space_single_line(c: char) -> bool {
    matches!(c, ' ' | '\t')
}

// less allocations trick from OXC
// https://github.com/oxc-project/oxc/blob/4c35f4abb6874bd741b84b34df7889637425e9ea/crates/oxc_transformer/src/jsx/jsx_impl.rs#L1061-L1091
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1933>.
fn add_line_of_jsx_text<'a>(
    trimmed_line: &'a str,
    acc: &mut Option<String>,
    only_line: &mut Option<&'a str>,
) {
    if let Some(buffer) = acc.as_mut() {
        // Already some text in accumulator. Push a space before this line is added to
        // `acc`.
        buffer.push(' ');
    } else if let Some(only_line_content) = only_line.take() {
        // This is the 2nd line containing text. Previous line did not contain any HTML
        // entities. Generate an accumulator containing previous line and a
        // trailing space. Current line will be added to the accumulator after
        // it.
        let mut buffer = String::with_capacity(trimmed_line.len() * 2); // rough estimate
        buffer.push_str(only_line_content);
        buffer.push(' ');
        *acc = Some(buffer);
    }

    // [TODO]: Decode any HTML entities in this line

    // For now, just use the trimmed line directly
    if let Some(buffer) = acc.as_mut() {
        buffer.push_str(trimmed_line);
    } else {
        // This is the first line containing text, and there are no HTML entities in
        // this line. Record this line in `only_line`.
        // If this turns out to be the only line, we won't need to construct a String,
        // so avoid all copying.
        *only_line = Some(trimmed_line);
    }
}
