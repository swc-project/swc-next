use super::{ReactRefreshOptions, RuntimeBindings};
use crate::uid::UidGenerator;
use swc_next_ecma_ast::{Ast, Program};
use swc_next_ecma_semantic::Semantic;

mod build;
mod registration;

pub(crate) fn transform<'a>(
    program: Program,
    ast: &mut Ast<'a>,
    semantic: &Semantic<'a>,
    options: &ReactRefreshOptions,
    bindings: &RuntimeBindings<'_>,
) {
    let mut uid = UidGenerator::new(ast, semantic);
    for name in bindings.implicit_roots().into_iter().flatten() {
        uid.reserve(name);
    }
    let root = options
        .refresh_reg
        .split_once('.')
        .map_or(options.refresh_reg.as_str(), |(root, _)| root);
    uid.reserve(root);
    registration::transform(program, ast, semantic, options, &mut uid);
}
