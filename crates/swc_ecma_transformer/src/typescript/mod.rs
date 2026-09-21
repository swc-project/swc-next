use swc_next_ecma_ast::{Ast, Lang, Program, SubRange};
use swc_next_ecma_semantic::Semantic;

use crate::syntax::{FollowingTransform, Noop};

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
    options: TypeScriptOptions,
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
        analysis::TypeScriptAnalysis::analyze(program, ast, semantic, &options, dependencies);
    lower::TypeScriptLowerer::new(&options, analysis, ast, semantic, &mut Noop)
        .lower_program(program);
}

pub(crate) fn transform_with_following<'a>(
    program: Program,
    ast: &mut Ast<'a>,
    semantic: &Semantic<'a>,
    options: TypeScriptOptions,
    following: &mut impl FollowingTransform<'a>,
) {
    if ast.lang() == Lang::Dts {
        transform(program, ast, semantic, options);
        return;
    }

    let dependencies = following.runtime_dependencies(semantic);
    let analysis =
        analysis::TypeScriptAnalysis::analyze(program, ast, semantic, &options, dependencies);
    lower::TypeScriptLowerer::new(&options, analysis, ast, semantic, following)
        .lower_program(program);
}

#[cfg(test)]
mod tests;
