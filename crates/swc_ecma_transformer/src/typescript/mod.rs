use swc_next_ecma_ast::{AstCtx, Lang, SubRange};
use swc_next_ecma_semantic::SemanticAst;

mod analysis;
mod lower;

mod options;
pub use options::{JsxImportOptions, TypeScriptOptions};

/// Applies only the TypeScript syntax transform.
///
/// Run any required normalization before analysis. Consumes the input binding
/// and returns the transformed context without analyzing it again. Re-run analysis
/// before subsequent semantic work.
pub fn transform<'ctx, 'a>(
    analyzed: SemanticAst<'a, &'ctx mut AstCtx<'a>>,
    options: &TypeScriptOptions,
) -> &'ctx mut AstCtx<'a> {
    let (ctx, semantic) = analyzed.into_parts();
    let semantic = &semantic;
    let program = ctx.root_program();
    // SAFETY: The consumed binding supplies this AST's current model. Lowering
    // uses only handles from this AST and preserves its invariants.
    let ast = unsafe { ctx.ast_mut() };
    if ast.lang() == Lang::Dts {
        program.set_directives(ast, SubRange::empty());
        program.set_body(ast, SubRange::empty());
        program.set_hashbang(ast, None);
        ast.clear_attached_comments();
        return ctx;
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
    ctx
}

#[cfg(test)]
mod tests;
