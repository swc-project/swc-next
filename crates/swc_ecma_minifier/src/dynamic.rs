//! Scopes that dynamic name lookup can reach.
//!
//! A direct `eval` can read, write, and redeclare every binding of the scopes
//! that enclose it, and outside strict code its `var` declarations bind names
//! in the enclosing function. A `with` statement looks up the names in its
//! body on an object before the enclosing scopes. Bindings of these scopes
//! keep their names and declarations, and unresolved names that a binding
//! created at run time can shadow are not replaced. Other scopes are optimized
//! as usual.

use swc_next_allocator::Idx;
use swc_next_ecma_ast::{Ast, ScopeId, SymbolId};
use swc_next_ecma_semantic::{Semantic, scope::ScopeKind};

/// Scopes whose names dynamic lookup can observe. Both lists are empty when
/// the program has no dynamic lookup.
#[derive(Default)]
pub(crate) struct DynamicScopes {
    /// Whether dynamic lookup can see the bindings of each scope.
    visible: Vec<bool>,
    /// Whether an unresolved name in each scope can resolve to a binding that
    /// dynamic lookup creates, or to an object property.
    shadowed: Vec<bool>,
}

impl DynamicScopes {
    pub(crate) fn build(ast: &Ast<'_>, semantic: &Semantic<'_>) -> Self {
        let mut scopes = Self::default();
        for entry in semantic.iter_scopes() {
            if entry.scope.kind == ScopeKind::With {
                scopes.mark(semantic, entry.id, Some(entry.id));
            }
        }
        // Any reference named `eval` can be a direct call of the global
        // `eval`, even one that resolves to a local binding that holds it.
        for entry in semantic.iter_references() {
            let reference = entry.reference;
            if ast.get_utf8(reference.name).as_bytes() != b"eval" {
                continue;
            }
            let scope = semantic.scope(reference.scope);
            // Strict code evaluates declarations in a separate environment.
            let shadowed = (!scope.flags.strict).then_some(scope.hoist_target);
            scopes.mark(semantic, reference.scope, shadowed);
        }
        if scopes.shadowed.is_empty() {
            return scopes;
        }
        // Scopes are created before their descendants, in id order.
        for entry in semantic.iter_scopes() {
            if let Some(parent) = entry.scope.parent
                && scopes.shadowed[parent.index()]
            {
                scopes.shadowed[entry.id.index()] = true;
            }
        }
        scopes
    }

    /// Marks `scope` and its ancestors as visible and `shadowed` as the root
    /// of shadowed scopes.
    fn mark(&mut self, semantic: &Semantic<'_>, scope: ScopeId, shadowed: Option<ScopeId>) {
        if self.visible.is_empty() {
            let count = semantic
                .iter_scopes()
                .last()
                .map_or(0, |entry| entry.id.index() + 1);
            self.visible = vec![false; count];
            self.shadowed = vec![false; count];
        }
        if let Some(shadowed) = shadowed {
            self.shadowed[shadowed.index()] = true;
        }
        let mut ancestor = Some(scope);
        // The ancestors of a visible scope are visible already.
        while let Some(id) = ancestor
            && !self.visible[id.index()]
        {
            self.visible[id.index()] = true;
            ancestor = semantic.scope(id).parent;
        }
    }

    /// Whether dynamic lookup can see a binding of `symbol`.
    pub(crate) fn contains(&self, semantic: &Semantic<'_>, symbol: SymbolId) -> bool {
        !self.visible.is_empty()
            && semantic
                .declarations(symbol)
                .any(|declaration| self.visible[declaration.scope.index()])
    }

    /// Whether an unresolved name in `scope` can resolve to a binding or an
    /// object property that only exists at run time.
    pub(crate) fn shadows(&self, scope: ScopeId) -> bool {
        self.shadowed.get(scope.index()).copied().unwrap_or(false)
    }
}
