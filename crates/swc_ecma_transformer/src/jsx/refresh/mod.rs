use super::ReactRefreshOptions;
use crate::uid::UidGenerator;
use signatures::SignatureTransformer;
use swc_next_ecma_ast::VisitMutWith;
use swc_next_ecma_ast::{Ast, Program, Stmt};
use swc_next_ecma_semantic::Semantic;

mod build;
mod digest;
mod hooks;
mod registration;
mod signatures;

pub(crate) fn transform<'a>(
    program: Program,
    ast: &mut Ast<'a>,
    semantic: &Semantic<'a>,
    options: &ReactRefreshOptions,
) -> Option<Stmt> {
    let mut uid = UidGenerator::new(ast, semantic);
    for callback in [&options.refresh_reg, &options.refresh_sig] {
        let root = callback
            .split_once('.')
            .map_or(callback.as_str(), |(root, _)| root);
        uid.reserve(root);
    }
    registration::transform(program, ast, semantic, options, &mut uid);

    // Registration keeps source functions, references, and scopes intact.
    let mut signatures = SignatureTransformer::new(ast, semantic, options, uid);
    program.visit_mut_children_with(&mut signatures);
    signatures.program_signature_declaration()
}
