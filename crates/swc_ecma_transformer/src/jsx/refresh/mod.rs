use super::{ReactRefreshOptions, RuntimeBindings};
use crate::{MultiPassTransformer, uid::UidGenerator};
use signatures::SignatureTransformer;
use swc_next_ecma_ast::{Stmt, VisitMutWith};

mod build;
mod digest;
mod hooks;
mod registration;
mod signatures;

impl MultiPassTransformer<'_, '_> {
    pub(super) fn refresh(
        &mut self,
        options: &ReactRefreshOptions,
        bindings: &RuntimeBindings<'_>,
    ) -> Option<Stmt> {
        let program = self.ctx.root_program();
        // SAFETY: The pipeline supplies this AST's current model. Registration
        // and signature injection retain source handles and allocate new handles in this AST.
        let ast = unsafe { self.ctx.ast_mut() };
        let semantic = &self.semantic;
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
        let runtime_import_anchor = signatures.program_signature_declaration();
        self.analyze();
        runtime_import_anchor
    }
}
