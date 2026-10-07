use super::Mutation;
use swc_next_ecma_ast::{AstUtf8, NodeKindData};
use swc_next_ecma_semantic::UsedNameState;

impl<'a> Mutation<'_, '_, 'a> {
    /// Reserves and interns a name in the shared transform collision domain.
    /// Collect existing names once, when a transform first needs a fresh name.
    pub fn generate_name(&mut self, base: &str, start_suffix: u64) -> AstUtf8 {
        let names = self.state.used_names.get_or_insert_with(|| {
            let mut names = UsedNameState::new(self.ast.allocator());
            for entry in self.state.semantic.iter_symbols() {
                names.observe(self.ast.get_wtf8(entry.symbol.name).as_bytes());
            }
            // Merged TypeScript declarations can have different source names
            // while sharing a canonical symbol, notably default exports.
            for entry in self.state.semantic.iter_declarations() {
                if let NodeKindData::BindingIdentifier(identifier) =
                    self.ast.kind_data(entry.declaration.node)
                {
                    names.observe(self.ast.get_utf8(identifier.name(self.ast)).as_bytes());
                }
            }
            for entry in self.state.semantic.iter_references() {
                names.observe(self.ast.get_utf8(entry.reference.name).as_bytes());
            }
            names
        });
        self.ast.add_utf8(names.next_name(base, start_suffix))
    }

    /// Keep names introduced after initialization reserved for later transforms.
    pub(crate) fn observe_name(&mut self, name: AstUtf8) {
        if let Some(names) = &mut self.state.used_names {
            names.observe(self.ast.get_utf8(name).as_bytes());
        }
    }
}
