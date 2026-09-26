use rustc_hash::{FxHashMap, FxHashSet};

use swc_next_ecma_ast::{Ast, AstUtf8};
use swc_next_ecma_semantic::Semantic;

/// Generates deterministic identifier names which cannot collide with source bindings or uses.
pub(crate) struct UidGenerator<'ast> {
    used_names: FxHashSet<&'ast [u8]>,
    next_suffixes: FxHashMap<&'ast [u8], u32>,
}

impl<'ast> UidGenerator<'ast> {
    pub(crate) fn new(ast: &Ast<'ast>, semantic: &Semantic<'_>) -> Self {
        let mut used_names = FxHashSet::default();
        for entry in semantic.iter_symbols() {
            let name = ast.get_wtf8(entry.symbol.name).as_bytes();
            used_names.insert(name);
        }
        for entry in semantic.iter_references() {
            let name = ast.get_utf8(entry.reference.name).as_bytes();
            used_names.insert(name);
        }
        Self {
            used_names,
            next_suffixes: FxHashMap::default(),
        }
    }

    pub(crate) fn fresh(&mut self, base: &'ast str, ast: &mut Ast<'ast>) -> AstUtf8 {
        if self.used_names.insert(base.as_bytes()) {
            return ast.add_utf8(base);
        }

        let next_suffix = self.next_suffixes.entry(base.as_bytes()).or_insert(2);
        loop {
            let candidate = format!("{base}{next_suffix}");
            *next_suffix += 1;
            if !self.used_names.contains(candidate.as_bytes()) {
                let name = ast.add_utf8(&candidate);
                self.used_names.insert(ast.get_utf8(name).as_bytes());
                return name;
            }
        }
    }
}
