use swc_next_ecma_ast::{Ast, Diagnostic, Program, VisitMutWith};
use swc_next_ecma_semantic::Semantic;

use crate::syntax::{FollowingTransform, SyntaxResult};

mod comments;
mod development;
mod display_name;
mod lower;
mod options;
mod self_context;
mod source;
mod text;

pub(crate) use lower::JsxLowerer;
pub use options::{JsxOptions, JsxRuntime};

pub(crate) use development::transform as transform_development_attributes;
pub(crate) use display_name::DisplayName;

pub fn transform<'a>(
    program: Program,
    ast: &mut Ast<'a>,
    semantic: &Semantic<'a>,
    options: JsxOptions,
) -> Vec<Diagnostic<'static>> {
    let options = comments::resolve_options(program, ast, options);
    transform_resolved(program, ast, semantic, options, None).diagnostics
}

pub(crate) fn resolve_options(program: Program, ast: &Ast<'_>, options: JsxOptions) -> JsxOptions {
    comments::resolve_options(program, ast, options)
}

pub(crate) fn transform_resolved<'a>(
    program: Program,
    ast: &mut Ast<'a>,
    semantic: &Semantic<'a>,
    options: JsxOptions,
    source_path: Option<&str>,
) -> SyntaxResult {
    let mut lowerer = lower::JsxLowerer::new(&options, ast, semantic, source_path);
    {
        let mut visitor = lower::JsxVisitor::new(ast, &mut lowerer);
        let _ = program.visit_mut_with(&mut visitor);
    }
    lowerer.finish_program(program, ast);
    lowerer.take_result()
}

#[cfg(test)]
mod tests;
