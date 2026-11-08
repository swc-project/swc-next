use swc_next_ecma_ast::{Lang, SubRange};

use crate::MultiPassTransformer;

mod analysis;
mod lower;

mod options;
pub use options::{JsxImportOptions, TypeScriptOptions};

impl MultiPassTransformer<'_, '_> {
    // Terminal public transforms discard the model; complete pipeline stages
    // reanalyze before any subsequent stage can read it.
    pub(super) fn typescript(&mut self, options: &TypeScriptOptions) {
        let program = self.ctx.root_program();
        // SAFETY: The pipeline supplies this AST's current model. Lowering
        // uses only handles from this AST and preserves its invariants.
        let ast = unsafe { self.ctx.ast_mut() };
        let semantic = &self.semantic;
        if ast.lang() == Lang::Dts {
            program.set_directives(ast, SubRange::empty());
            program.set_body(ast, SubRange::empty());
            program.set_hashbang(ast, None);
            ast.clear_attached_comments();
            return;
        }

        let dependencies = analysis::jsx_dependencies(program, ast, semantic, &options.jsx);
        let analysis =
            analysis::TypeScriptAnalysis::analyze(program, ast, semantic, options, dependencies);
        lower::TypeScriptLowerer::new(options, analysis, ast, semantic).lower_program(program);
        let lang = match ast.lang() {
            Lang::Ts => Lang::Js,
            Lang::Tsx => Lang::Jsx,
            lang => lang,
        };
        ast.set_source_metadata(ast.source_type(), lang);
    }
}

#[cfg(test)]
mod tests;
