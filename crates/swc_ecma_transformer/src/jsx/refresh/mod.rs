use super::ReactRefreshOptions;
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
) {
    let mut uid = UidGenerator::new(ast, semantic);
    let root = options
        .refresh_reg
        .split_once('.')
        .map_or(options.refresh_reg.as_str(), |(root, _)| root);
    uid.reserve(root, ast);
    registration::transform(program, ast, semantic, options, &mut uid);
}
