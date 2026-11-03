use swc_next_ecma_ast::{Ast, Diagnostic, Program, VisitMutWith};
use swc_next_ecma_semantic::{AnalyzeOptions, Semantic, analyze};

use crate::uid::LazyUidGenerator;

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

/// Lower JSX using options updated with the file's pragmas.
pub fn transform<'a>(
    program: Program,
    ast: &mut Ast<'a>,
    semantic: &Semantic<'a>,
    options: &JsxOptions,
    source_path: Option<&str>,
) -> Vec<Diagnostic<'static>> {
    let bindings = RuntimeBindings::new(options, ast.source_type());
    let mut runtime_import_anchor = None;
    let refreshed;
    let semantic = if let Some(refresh) = &options.refresh {
        runtime_import_anchor = refresh::transform(program, ast, semantic, refresh, &bindings);
        refreshed = analyze(ast, AnalyzeOptions::default()).semantic;
        &refreshed
    } else {
        semantic
    };
    let implicit_roots = bindings.implicit_roots();
    let uid = LazyUidGenerator::new(semantic, &implicit_roots);
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
pub fn inject_pragmas(program: Program, ast: &Ast<'_>, options: &mut JsxOptions) {
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

#[cfg(test)]
mod tests;
