use swc_next_ecma_ast::{Ast, Lang, Program, SubRange};
use swc_next_ecma_semantic::Semantic;

mod analysis;
mod lower;

mod options;
pub use options::{JsxImportOptions, TypeScriptOptions};

/// Applies only the TypeScript syntax transform.
///
/// The caller is responsible for running any required normalization first.
pub fn transform<'a>(
    program: Program,
    ast: &mut Ast<'a>,
    semantic: &Semantic<'a>,
    options: &TypeScriptOptions,
) {
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

#[cfg(test)]
mod tests;
