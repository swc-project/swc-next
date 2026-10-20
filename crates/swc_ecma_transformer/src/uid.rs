use swc_next_ecma_ast::{Ast, AstUtf8};
use swc_next_ecma_semantic::Semantic;

use crate::used_names::UsedNameState;

/// Generates deterministic identifier names which cannot collide with source bindings or uses.
pub(crate) struct UidGenerator<'ast> {
    used_names: UsedNameState<'ast>,
}

impl<'ast> UidGenerator<'ast> {
    pub(crate) fn new(ast: &Ast<'ast>, semantic: &Semantic<'_>) -> Self {
        let mut used_names = UsedNameState::new(ast.allocator());
        for entry in semantic.iter_symbols() {
            used_names.observe(ast.get_wtf8(entry.symbol.name).as_bytes());
        }
        for entry in semantic.iter_references() {
            used_names.observe(ast.get_utf8(entry.reference.name).as_bytes());
        }
        Self { used_names }
    }

    pub(crate) fn reserve(&mut self, name: &str) {
        self.used_names.observe(name.as_bytes());
    }

    pub(crate) fn fresh(&mut self, base: &'ast str, ast: &mut Ast<'ast>) -> AstUtf8 {
        ast.add_utf8(self.used_names.next_name(base, 2))
    }
}
