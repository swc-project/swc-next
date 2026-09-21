use super::{JsxOptions, self_context::JsxSelfContext, source::SourceLocations};
use crate::{syntax::SyntaxResult, uid::LazyUidGenerator};
pub(crate) use runtime::RuntimeBindings;
use swc_next_ecma_ast::{Ast, AstUtf8, Diagnostic, Expr, ExprData, JsxElement, JsxFragment};

mod build;
mod elements;
mod runtime;
mod visit;
pub(super) use visit::JsxVisitor;

enum JsxRoot {
    Element(JsxElement),
    Fragment(JsxFragment),
}

impl JsxRoot {
    #[inline]
    fn from_expression(expression: Expr, ast: &Ast<'_>) -> Option<Self> {
        match ast.expr_data(expression) {
            ExprData::JsxElement(element) => Some(Self::Element(element)),
            ExprData::JsxFragment(fragment) => Some(Self::Fragment(fragment)),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
struct LoweringContext {
    opening_end: u32,
    automatic_fallback: bool,
    can_add_self: bool,
}

#[derive(Default)]
struct NamespaceProjection {
    factory: Option<AstUtf8>,
    fragment: Option<AstUtf8>,
}

/// Direct JSX children share the source namespace projection of this root.
struct JsxEmitter<'lowerer, 'options, 'ast, const TRACK_SELF: bool> {
    state: &'lowerer mut JsxLowerer<'options, 'ast, TRACK_SELF>,
    namespace: NamespaceProjection,
}

pub(crate) struct JsxLowerer<'options, 'ast, const TRACK_SELF: bool> {
    options: &'options JsxOptions,
    pure: bool,
    bindings: RuntimeBindings<'options>,
    uid_generator: LazyUidGenerator<'options, 'ast>,
    self_context: JsxSelfContext,
    source_locations: SourceLocations,
    source_path: &'options str,
    filename: Option<AstUtf8>,
    diagnostics: Vec<Diagnostic<'static>>,
}

impl<'options, 'ast, const TRACK_SELF: bool> JsxLowerer<'options, 'ast, TRACK_SELF> {
    pub(crate) fn new(
        options: &'options JsxOptions,
        bindings: RuntimeBindings<'options>,
        uid_generator: LazyUidGenerator<'options, 'ast>,
        source_path: Option<&'options str>,
    ) -> Self {
        let pure = options
            .pure
            .unwrap_or(options.import_source.is_none() && options.pragma.is_none());
        debug_assert_eq!(TRACK_SELF, options.development && options.source_self);
        let self_context = JsxSelfContext::new();
        Self {
            options,
            pure,
            bindings,
            uid_generator,
            self_context,
            source_locations: SourceLocations::default(),
            source_path: source_path.unwrap_or_default(),
            filename: None,
            diagnostics: Vec::new(),
        }
    }

    pub(crate) fn take_result(&mut self) -> SyntaxResult {
        SyntaxResult {
            diagnostics: std::mem::take(&mut self.diagnostics),
        }
    }
}
