use super::{ReactRefreshOptions, RuntimeBindings};
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
    bindings: &RuntimeBindings<'_>,
) -> Option<Stmt> {
    let mut uid = UidGenerator::new(ast.allocator());
    for entry in semantic.iter_symbols() {
        uid.observe(ast.get_wtf8(entry.symbol.name).as_bytes());
    }
    for entry in semantic.iter_references() {
        uid.observe(ast.get_utf8(entry.reference.name).as_bytes());
    }
    for name in bindings.implicit_roots().into_iter().flatten() {
        uid.observe(name.as_bytes());
    }
    for callback in [&options.refresh_reg, &options.refresh_sig] {
        let root = callback
            .split_once('.')
            .map_or(callback.as_str(), |(root, _)| root);
        uid.observe(root.as_bytes());
    }
    registration::transform(program, ast, semantic, options, &mut uid);

    // Registration keeps source functions, references, and scopes intact.
    let mut signatures = SignatureTransformer::new(ast, semantic, options, uid);
    program.visit_mut_children_with(&mut signatures);
    signatures.program_signature_declaration()
}
