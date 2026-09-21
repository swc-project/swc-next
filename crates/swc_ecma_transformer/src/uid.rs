use swc_next_ecma_ast::{Ast, AstUtf8};
use swc_next_ecma_semantic::Semantic;

use crate::used_names::UsedNameState;

/// Seeds source and implicit runtime names when a binding is first emitted.
pub(crate) struct LazyUidGenerator<'source, 'ast> {
    semantic: &'source Semantic<'ast>,
    reserved_names: &'source [Option<&'source str>],
    generator: Option<UidGenerator<'ast>>,
}

impl<'source, 'ast> LazyUidGenerator<'source, 'ast> {
    pub(crate) fn new(
        semantic: &'source Semantic<'ast>,
        reserved_names: &'source [Option<&'source str>],
    ) -> Self {
        Self {
            semantic,
            reserved_names,
            generator: None,
        }
    }

    pub(crate) fn fresh(&mut self, base: &'ast str, ast: &mut Ast<'ast>) -> AstUtf8 {
        self.generator
            .get_or_insert_with(|| {
                let mut generator = UidGenerator::new(ast, self.semantic);
                for name in self.reserved_names.iter().flatten() {
                    generator.reserve(name);
                }
                generator
            })
            .fresh(base, ast)
    }
}

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
