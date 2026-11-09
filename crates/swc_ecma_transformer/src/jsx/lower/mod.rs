use super::{JsxOptions, self_context::JsxSelfContext, source::SourceLocations};
use crate::uid::UidGenerator;
pub(crate) use runtime::RuntimeBindings;
use swc_next_ecma_ast::{AstUtf8, Diagnostic, Stmt};

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

struct JsxEmitter<'lowerer, 'options, 'ast> {
    state: &'lowerer mut JsxLowerer<'options, 'ast>,
}

pub(crate) struct JsxLowerer<'options, 'ast> {
    options: &'options JsxOptions,
    pure: bool,
    bindings: RuntimeBindings<'options>,
    uid_generator: UidGenerator<'ast>,
    self_context: JsxSelfContext,
    source_locations: SourceLocations,
    source_path: &'options str,
    filename: Option<AstUtf8>,
    diagnostics: Vec<Diagnostic>,
    runtime_import_anchor: Option<Stmt>,
}

impl<'options, 'ast> JsxLowerer<'options, 'ast> {
    pub(crate) fn new(
        options: &'options JsxOptions,
        bindings: RuntimeBindings<'options>,
        uid_generator: UidGenerator<'ast>,
        source_path: Option<&'options str>,
    ) -> Self {
        let pure = options
            .pure
            .unwrap_or(options.import_source.is_none() && options.pragma.is_none());
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
            runtime_import_anchor: None,
        }
    }

    pub(crate) fn set_runtime_import_anchor(&mut self, anchor: Option<Stmt>) {
        self.runtime_import_anchor = anchor;
    }

    pub(crate) fn take_diagnostics(&mut self) -> Vec<Diagnostic> {
        std::mem::take(&mut self.diagnostics)
    }
}
