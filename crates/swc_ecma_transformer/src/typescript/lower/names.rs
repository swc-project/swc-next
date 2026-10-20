use super::TypeScriptLowerer;
use crate::used_names::UsedNameState;
use rustc_hash::FxHashMap;
use swc_next_ecma_ast::SymbolId;

impl<'ast> TypeScriptLowerer<'_, 'ast> {
    pub(super) fn fresh_uid(&mut self, base: &'ast str) -> swc_next_ecma_ast::AstInternedUtf8 {
        let names = self.used_names.get_or_insert_with(|| {
            let mut names = UsedNameState::new(self.ast.allocator());
            for entry in self.semantic.iter_symbols() {
                names.observe(self.ast.get_wtf8(entry.symbol.name).as_bytes());
            }
            for entry in self.semantic.iter_references() {
                names.observe(self.ast.get_utf8(entry.reference.name).as_bytes());
            }
            names
        });
        self.ast.add_interned_utf8(names.next_name(base, 2))
    }

    fn can_reuse_source_name(
        &mut self,
        name: swc_next_ecma_ast::AstInternedUtf8,
        symbol: SymbolId,
    ) -> bool {
        let owners = self.source_name_owners.get_or_insert_with(|| {
            let mut owners = FxHashMap::default();
            for entry in self.semantic.iter_symbols() {
                let name = self.ast.get_wtf8(entry.symbol.name).as_bytes();
                record_source_name_owner(&mut owners, name, Some(entry.id));
            }
            for entry in self.semantic.iter_references() {
                let name = self.ast.get_utf8(entry.reference.name).as_bytes();
                record_source_name_owner(&mut owners, name, entry.reference.resolution().symbol());
            }
            owners
        });
        let name = self.ast.get_utf8(name).as_bytes();
        owners.get(name).copied() == Some(Some(symbol))
    }

    pub(super) fn container_parameter(
        &mut self,
        name: swc_next_ecma_ast::AstInternedUtf8,
        symbol: SymbolId,
    ) -> swc_next_ecma_ast::AstInternedUtf8 {
        if self.can_reuse_source_name(name, symbol) {
            return name;
        }
        let base = self.ast_ref().get_utf8(name);
        self.fresh_uid(base)
    }
}

fn record_source_name_owner<'a>(
    owners: &mut FxHashMap<&'a [u8], Option<SymbolId>>,
    name: &'a [u8],
    owner: Option<SymbolId>,
) {
    owners
        .entry(name)
        .and_modify(|existing| {
            if *existing != owner {
                *existing = None;
            }
        })
        .or_insert(owner);
}
