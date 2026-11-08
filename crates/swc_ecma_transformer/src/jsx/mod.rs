use swc_next_ecma_ast::{Diagnostic, VisitMutWith};

use crate::{MultiPassTransformer, uid::UidGenerator};

mod display_name;
mod lower;
mod options;
pub(crate) mod pragma;
mod refresh;
mod self_context;
mod source;
mod text;

use lower::RuntimeBindings;
pub use options::{JsxOptions, JsxRuntime, ReactRefreshOptions};
use pragma::{PragmaKind, pragmas};

pub(crate) use display_name::DisplayName;

impl MultiPassTransformer<'_, '_> {
    // The caller either reanalyzes immediately or returns the context and discards
    // this pipeline, so the pre-lowering model cannot be reused after mutation.
    pub(super) fn jsx(
        &mut self,
        options: &JsxOptions,
        source_path: Option<&str>,
    ) -> Vec<Diagnostic<'static>> {
        let bindings = RuntimeBindings::new(options, self.ctx.source_type());
        let runtime_import_anchor = options
            .refresh
            .as_ref()
            .and_then(|refresh| self.refresh(refresh, &bindings));
        let program = self.ctx.root_program();
        // SAFETY: The pipeline supplies this AST's current model. Lowering
        // obtains all handles from this AST and preserves their ownership and validity.
        let ast = unsafe { self.ctx.ast_mut() };
        let semantic = &self.semantic;
        let implicit_roots = bindings.implicit_roots();
        let mut uid = UidGenerator::new(ast.allocator());
        for entry in semantic.iter_symbols() {
            uid.observe(ast.get_wtf8(entry.symbol.name).as_bytes());
        }
        for entry in semantic.iter_references() {
            uid.observe(ast.get_utf8(entry.reference.name).as_bytes());
        }
        for name in implicit_roots.into_iter().flatten() {
            uid.observe(name.as_bytes());
        }
        let mut lowerer = lower::JsxLowerer::new(options, bindings, uid, source_path);
        lowerer.set_runtime_import_anchor(runtime_import_anchor);
        {
            let mut visitor = lower::JsxVisitor::new(ast, &mut lowerer);
            let _ = program.visit_mut_with(&mut visitor);
        }
        lowerer.finish_program(program, ast);
        lowerer.take_diagnostics()
    }

    /// Apply JSX pragma comments to the options for this program.
    pub(super) fn jsx_pragmas(&mut self, options: &mut JsxOptions) {
        let program = self.ctx.root_program();
        // SAFETY: Pragma inspection only reads handles obtained from this AST.
        let ast = unsafe { self.ctx.ast() };
        for (kind, value) in pragmas(program, ast) {
            match kind {
                PragmaKind::Jsx => options.pragma = Some(value.to_owned()),
                PragmaKind::Runtime => match value {
                    "classic" => options.runtime = JsxRuntime::Classic,
                    "automatic" => options.runtime = JsxRuntime::Automatic,
                    _ => {}
                },
                PragmaKind::ImportSource => options.import_source = Some(value.to_owned()),
                PragmaKind::Fragment => options.pragma_frag = Some(value.to_owned()),
            }
        }
    }
}

#[cfg(test)]
mod tests;
