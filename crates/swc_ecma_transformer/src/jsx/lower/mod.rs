use super::{JsxOptions, self_context::JsxSelfContext, source::SourceLocations};
use crate::{syntax::SyntaxResult, uid::UidGenerator};
use build::QualifiedName;
use swc_next_ecma_ast::{Ast, AstUtf8, Diagnostic, VariableDeclaration};
use swc_next_ecma_semantic::Semantic;

mod build;
mod elements;
mod runtime;
mod visit;
pub(super) use visit::JsxVisitor;

#[derive(Clone, Copy)]
struct LoweringContext {
    opening_end: u32,
    automatic_fallback: bool,
    can_add_self: bool,
}

pub(crate) struct JsxLowerer<'options, 'ast> {
    options: &'options JsxOptions,
    pure: bool,
    pragma: QualifiedName<'options>,
    pragma_frag: QualifiedName<'options>,
    import_source: &'options str,
    uid_generator: UidGenerator<'ast>,
    self_context: Option<JsxSelfContext>,
    source_locations: SourceLocations,
    source_path: &'options str,
    jsx: Option<AstUtf8>,
    jsxs: Option<AstUtf8>,
    jsx_dev: Option<AstUtf8>,
    fragment: Option<AstUtf8>,
    create_element: Option<AstUtf8>,
    runtime_object: Option<AstUtf8>,
    react_object: Option<AstUtf8>,
    filename: Option<AstUtf8>,
    helper_imports: Vec<(&'static str, AstUtf8)>,
    diagnostics: Vec<Diagnostic<'static>>,
    generated_prelude: Option<VariableDeclaration>,
}

impl<'options, 'ast> JsxLowerer<'options, 'ast> {
    pub(crate) fn new(
        options: &'options JsxOptions,
        ast: &Ast<'ast>,
        semantic: &Semantic<'_>,
        source_path: Option<&'options str>,
    ) -> Self {
        let pure = options
            .pure
            .unwrap_or(options.import_source.is_none() && options.pragma.is_none());
        let pragma = QualifiedName::parse(options.pragma.as_deref(), "React", "createElement");
        let pragma_frag = QualifiedName::parse(options.pragma_frag.as_deref(), "React", "Fragment");
        let import_source = options
            .import_source
            .as_deref()
            .filter(|source| !source.is_empty())
            .unwrap_or("react");
        let self_context = (options.jsx_self || (options.development && options.source_self))
            .then(|| JsxSelfContext::collect(ast.root_program(), ast));
        Self {
            options,
            pure,
            pragma,
            pragma_frag,
            import_source,
            uid_generator: UidGenerator::new(ast, semantic),
            self_context,
            source_locations: SourceLocations::default(),
            source_path: source_path.unwrap_or_default(),
            jsx: None,
            jsxs: None,
            jsx_dev: None,
            fragment: None,
            create_element: None,
            runtime_object: None,
            react_object: None,
            filename: None,
            helper_imports: Vec::new(),
            diagnostics: Vec::new(),
            generated_prelude: None,
        }
    }

    pub(crate) fn take_result(&mut self) -> SyntaxResult {
        SyntaxResult {
            diagnostics: std::mem::take(&mut self.diagnostics),
            generated_prelude: self.generated_prelude,
        }
    }
}
