use swc_next_ecma_ast::{
    AstUtf8, BindingIdentifier, DUMMY_SP, IdentifierReference, NodeKindData, ScopeId, Span,
    SymbolId,
};
use swc_next_ecma_semantic::{ReferenceFlags, SymbolFlags};

use crate::{context::Mutation, used_names::UsedNameState};

/// A generated binding keeps its declaration and symbol together
#[derive(Clone, Copy)]
pub struct BoundIdentifier {
    pub identifier: BindingIdentifier,
    pub symbol: SymbolId,
}

impl BoundIdentifier {
    pub fn named(
        ctx: &mut Mutation<'_, '_, '_>,
        scope: ScopeId,
        base: &str,
        flags: SymbolFlags,
    ) -> BoundIdentifier {
        let name = ctx.generate_name(base, 2);
        let identifier = ctx.ast.binding_identifier(DUMMY_SP, name);
        let symbol = ctx
            .state
            .semantic
            .register_binding(ctx.ast, scope, identifier, flags);
        let binding = BoundIdentifier { identifier, symbol };
        binding
    }

    /// Create a fresh read-only reference to this binding.
    pub fn reference_read(
        &self,
        ctx: &mut Mutation<'_, '_, '_>,
        scope: ScopeId,
        span: Span,
    ) -> IdentifierReference {
        let node = ctx
            .ast
            .identifier_reference(span, self.identifier.name(ctx.ast));
        ctx.state.semantic.register_reference(
            ctx.ast,
            scope,
            node,
            Some(self.symbol),
            ReferenceFlags::default(),
        );
        node
    }

    /// Create a fresh write-only reference for an assignment target.
    pub fn reference_write(
        &self,
        ctx: &mut Mutation<'_, '_, '_>,
        scope: ScopeId,
        span: Span,
    ) -> IdentifierReference {
        let node = ctx
            .ast
            .identifier_reference(span, self.identifier.name(ctx.ast));
        ctx.state.semantic.register_reference(
            ctx.ast,
            scope,
            node,
            Some(self.symbol),
            ReferenceFlags::default().write_only(),
        );
        node
    }
}

impl Mutation<'_, '_, '_> {
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
}
