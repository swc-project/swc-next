use swc_next_ecma_ast::{Ast, Diagnostic, Program, VisitMutWith};
use swc_next_ecma_semantic::Semantic;

use crate::{
    syntax::{FollowingTransform, SyntaxResult},
    uid::LazyUidGenerator,
};

mod comments;
mod display_name;
mod lower;
mod options;
mod refresh;
mod self_context;
mod source;
mod text;

pub(crate) use lower::{JsxLowerer, RuntimeBindings};
pub use options::{JsxOptions, JsxRuntime, ReactRefreshOptions};

pub(crate) use display_name::DisplayName;
pub(crate) use refresh::transform as transform_refresh;

pub fn transform<'a>(
    program: Program,
    ast: &mut Ast<'a>,
    semantic: &Semantic<'a>,
    options: JsxOptions,
) -> Vec<Diagnostic<'static>> {
    let options = comments::resolve_options(program, ast, options);
    let bindings = RuntimeBindings::new(&options, ast.source_type());
    transform_resolved(program, ast, semantic, &options, bindings, None).diagnostics
}

pub(crate) fn resolve_options(program: Program, ast: &Ast<'_>, options: JsxOptions) -> JsxOptions {
    comments::resolve_options(program, ast, options)
}

pub(crate) fn transform_resolved<'a, 'options>(
    program: Program,
    ast: &mut Ast<'a>,
    semantic: &Semantic<'a>,
    options: &'options JsxOptions,
    bindings: RuntimeBindings<'options>,
    source_path: Option<&str>,
) -> SyntaxResult {
    if options.development && options.source_self {
        lower_resolved::<true>(program, ast, semantic, options, bindings, source_path)
    } else {
        lower_resolved::<false>(program, ast, semantic, options, bindings, source_path)
    }
}

fn lower_resolved<'a, 'options, const TRACK_SELF: bool>(
    program: Program,
    ast: &mut Ast<'a>,
    semantic: &Semantic<'a>,
    options: &'options JsxOptions,
    bindings: RuntimeBindings<'options>,
    source_path: Option<&str>,
) -> SyntaxResult {
    let implicit_roots = bindings.implicit_roots();
    let uid = LazyUidGenerator::new(semantic, &implicit_roots);
    let mut lowerer = lower::JsxLowerer::<TRACK_SELF>::new(options, bindings, uid, source_path);
    {
        let mut visitor = lower::JsxVisitor::new(ast, &mut lowerer);
        let _ = program.visit_mut_with(&mut visitor);
    }
    lowerer.finish_program(program, ast);
    lowerer.take_result()
}

#[cfg(test)]
mod tests;
