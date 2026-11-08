use swc_next_ecma_ast::{AstCtx, Diagnostic, VisitMutWith};
use swc_next_ecma_semantic::{AnalyzeOptions, SemanticAst, analyze};

use crate::uid::UidGenerator;

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

/// Lowers JSX using options updated with the file's pragmas.
///
/// Consumes the input binding and returns the transformed context without a bound
/// semantic model. Re-run analysis before subsequent semantic work.
pub fn transform<'ctx, 'a>(
    mut analyzed: SemanticAst<'a, &'ctx mut AstCtx<'a>>,
    options: &JsxOptions,
    source_path: Option<&str>,
) -> (&'ctx mut AstCtx<'a>, Vec<Diagnostic<'static>>) {
    let bindings = RuntimeBindings::new(options, analyzed.ctx().source_type());
    let mut runtime_import_anchor = None;
    if let Some(refresh) = &options.refresh {
        let (ctx, anchor) = refresh::transform(analyzed, refresh, &bindings);
        runtime_import_anchor = anchor;
        analyzed = analyze(ctx, AnalyzeOptions::default()).semantic;
    }
    let (ctx, semantic) = analyzed.into_parts();
    let program = ctx.root_program();
    // SAFETY: The consumed binding supplies this AST's current model. Lowering
    // obtains all handles from this AST and preserves their ownership and validity.
    let ast = unsafe { ctx.ast_mut() };
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
    let diagnostics = lowerer.take_diagnostics();
    (ctx, diagnostics)
}

/// Apply JSX pragma comments to the options for this program.
pub fn inject_pragmas(ctx: &AstCtx<'_>, options: &mut JsxOptions) {
    let program = ctx.root_program();
    // SAFETY: Pragma inspection only reads handles obtained from this AST.
    let ast = unsafe { ctx.ast() };
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
