//! Protect free names in inline helpers from bindings in the surrounding program.

use swc_next_ecma_semantic::ReferenceSpace;

use super::TransformCtx;

impl TransformCtx<'_, '_> {
    pub(crate) fn protect_helper_globals(&mut self) {
        while !self.state.helpers.globals.is_empty() {
            let scope = self.program_scope();
            let globals = std::mem::take(&mut self.state.helpers.globals);
            let collisions: Vec<_> = globals
                .into_iter()
                .filter_map(|(name, references)| {
                    let symbol = self.semantic().binding(scope, name.as_bytes())?;
                    self.semantic()
                        .symbol(symbol)
                        .flags
                        .visible_in(ReferenceSpace::Value)
                        .then_some((name, symbol, references))
                })
                .collect();
            let renames: Vec<_> = collisions
                .iter()
                .map(|(name, symbol, _)| (*symbol, self.generate_name(&format!("_{name}"), 2)))
                .collect();
            let rejected = self.rename_bindings(renames);
            for (_, symbol, references) in collisions {
                if rejected.contains(&symbol) {
                    // Keep the AST's lexical resolution consistent on error.
                    for reference in references {
                        self.state
                            .semantic
                            .bind_unresolved_reference(reference, symbol);
                    }
                }
            }
            // Name preservation can prepare helpers with additional free names.
        }
    }
}
