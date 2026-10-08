//! Low-level index operations composed by the transformer context.
//!
//! Callers must update AST edges in the same operation. These methods do not
//! expose the model's mutable tables.

use oxc_index::Idx;
use swc_next_allocator::CloneIn;
use swc_next_allocator::hash_map::HashMap as ArenaHashMap;
use swc_next_ecma_ast::{
    Ast, AstUtf8, BindingIdentifier, IdentifierReference, NodeId, NodeKindData, ReferenceId,
    ScopeId, SymbolId,
};

use crate::{
    Declaration, DeclarationId, Reference, ReferenceFlags, Semantic, Symbol, SymbolFlags,
    binder::Range,
    scope::{Scope, ScopeFlags, ScopeKind},
};

impl<'a> Semantic<'a> {
    /// Initializes semantic slots for newly allocated AST nodes.
    #[doc(hidden)]
    pub fn grow_node_indexes(&mut self, node_count: usize) {
        assert!(node_count >= self.node_symbols.len());
        self.node_symbols.resize(node_count, None);
        self.node_references.resize(node_count, None);
        self.node_scopes.resize(node_count, None);
    }

    /// Register freshly allocated nodes in their evaluation scope.
    #[doc(hidden)]
    pub fn register_nodes(&mut self, ast: &Ast<'a>, start: usize, scope: ScopeId) {
        self.grow_node_indexes(ast.node_count());
        self.node_scopes[start..].fill(Some(scope));
    }

    /// Registers a fresh declaration whose name is unique in its owning scope.
    #[doc(hidden)]
    pub fn register_binding(
        &mut self,
        ast: &Ast<'a>,
        node: BindingIdentifier,
        scope: ScopeId,
        flags: SymbolFlags,
    ) -> SymbolId {
        self.grow_node_indexes(ast.node_count());
        self.node_scopes[node.node_id().index()] = Some(scope);
        let name = node.name(ast);
        let bytes = ast.get_utf8(name).as_bytes();
        assert!(self.binding(scope, bytes).is_none());
        assert!(self.node_symbols[node.node_id().index()].is_none());
        let symbol = SymbolId::from_usize(self.symbols.len());
        let declaration = DeclarationId::from_usize(self.declarations.len());
        self.symbols.push(Symbol {
            name: name.into(),
            flags,
            declaration,
        });
        self.declarations.push(Declaration {
            node: node.node_id(),
            symbol,
            scope,
            flags,
        });
        self.use_ranges.push(Range::default());
        let bytes = bytes.clone_in(ast.allocator());
        self.scope_maps[scope.index()].insert(bytes, symbol);
        self.node_symbols[node.node_id().index()] = Some(symbol);
        self.refresh_named_references(ast, name);
        symbol
    }

    /// Adds a reference and updates both its node index and the symbol's uses.
    #[doc(hidden)]
    pub fn register_reference(
        &mut self,
        ast: &Ast<'a>,
        node: IdentifierReference,
        scope: ScopeId,
        symbol: Option<SymbolId>,
        mut flags: ReferenceFlags,
    ) -> ReferenceId {
        self.grow_node_indexes(ast.node_count());
        self.node_scopes[node.node_id().index()] = Some(scope);
        assert!(self.node_references[node.node_id().index()].is_none());
        if flags.space == crate::ReferenceSpace::Value {
            let target = symbol.map(|symbol| self.scope_of(symbol));
            let arguments = symbol.is_none() && ast.get_utf8(node.name(ast)) == "arguments";
            for ancestor in self.scopes.ancestors(Some(scope)) {
                let current = self.scope(ancestor);
                if Some(ancestor) == target || (arguments && current.flags.arguments_barrier) {
                    break;
                }
                if current.kind == ScopeKind::With {
                    flags = flags.with_dynamic();
                    break;
                }
            }
        }
        let id = ReferenceId::from_usize(self.references.len());
        self.references.push(Reference {
            name: node.name(ast),
            scope,
            node: node.node_id(),
            symbol,
            flags,
        });
        self.node_references[node.node_id().index()] = Some(id);
        if let Some(symbol) = symbol {
            self.append_use(symbol, id);
        }
        id
    }

    /// Retires a scope after its bindings, references and children have been moved.
    #[doc(hidden)]
    pub fn remove_scope(&mut self, scope: ScopeId) {
        debug_assert!(self.scope_maps[scope.index()].is_empty());
        debug_assert!(
            !self
                .iter_references()
                .any(|entry| entry.reference.scope == scope)
        );
        debug_assert!(
            !self
                .scopes
                .list
                .iter()
                .any(|child| child.active && child.parent == Some(scope))
        );
        self.scopes.list[scope.index()].active = false;
        self.scopes.list[scope.index()].flags.direct_eval = false;
        self.hoisting_variables[scope.index()] = None;
    }

    fn append_use(&mut self, symbol: SymbolId, reference: ReferenceId) {
        let range = self.use_ranges[symbol.index()];
        let start = self.use_ids.len();
        for offset in 0..range.len as usize {
            self.use_ids
                .push(self.use_ids[range.start as usize + offset]);
        }
        self.use_ids.push(reference);
        self.use_ranges[symbol.index()] = Range {
            start: start.try_into().expect("reference storage overflow"),
            len: range.len + 1,
        };
    }

    /// Creates a child scope and all associated lookup tables.
    #[doc(hidden)]
    pub fn create_child_scope(
        &mut self,
        ast: &Ast<'a>,
        owner: NodeId,
        parent: ScopeId,
        kind: ScopeKind,
    ) -> ScopeId {
        assert!(self.scope(parent).active, "parent scope must be active");
        self.grow_node_indexes(ast.node_count());
        let id = ScopeId::from_usize(self.scopes.list.len());
        let parent_scope = self.scope(parent);
        let hoist_target = if kind.is_hoist_target() {
            id
        } else {
            parent_scope.hoist_target
        };
        self.scopes.list.push(Scope {
            active: true,
            node: owner,
            parent: Some(parent),
            hoist_target,
            function_body: None,
            kind,
            flags: ScopeFlags {
                strict: parent_scope.flags.strict,
                direct_eval: false,
                arguments_barrier: kind == ScopeKind::StaticBlock
                    || (kind == ScopeKind::Function
                        && matches!(ast.kind_data(owner), NodeKindData::Function(_))),
            },
        });
        self.node_scopes[owner.index()] = Some(id);
        self.scope_maps.push(ArenaHashMap::new_in(ast.allocator()));
        self.hoisting_variables.push(None);
        id
    }

    /// Reparent a function/static-block scope; descendants keep their own IDs
    /// and hoist targets, and only the outer evaluation boundary changes.
    #[doc(hidden)]
    pub fn reparent_scope(&mut self, scope: ScopeId, parent: ScopeId) {
        assert!(self.scope(scope).active && self.scope(parent).active);
        assert!(self.scope(scope).kind.is_hoist_target());
        assert!(!self.scopes.ancestors(Some(parent)).any(|id| id == scope));
        if self.scope(scope).parent == Some(parent) {
            return;
        }
        self.scopes.list[scope.index()].parent = Some(parent);
        if self.scope(scope).flags.direct_eval {
            // Current lowering retains the old ancestors and may add a closure scope.
            self.scopes.mark_direct_eval(parent);
        }
    }

    /// Transfers scope ownership while preserving all contained identities.
    #[doc(hidden)]
    pub fn transfer_scope_owner(
        &mut self,
        ast: &Ast<'_>,
        old: NodeId,
        new: NodeId,
        kind: ScopeKind,
    ) {
        let previous = self.scope_outside(old);
        let current = self.scope_at(old);
        if let Some(current) = current {
            let mut next = Some(current);
            while let Some(id) = next {
                let scope = &mut self.scopes.list[id.index()];
                if scope.node != old {
                    break;
                }
                next = scope.parent;
                scope.node = new;
                if scope.kind != ScopeKind::ExpressionName {
                    scope.kind = kind;
                    scope.flags.arguments_barrier = kind == ScopeKind::StaticBlock
                        || (kind == ScopeKind::Function
                            && matches!(ast.kind_data(new), NodeKindData::Function(_)));
                }
            }
        } else {
            for scope in &mut self.scopes.list {
                if scope.active && scope.node == old {
                    scope.node = new;
                    if scope.kind != ScopeKind::ExpressionName {
                        scope.kind = kind;
                        scope.flags.arguments_barrier = kind == ScopeKind::StaticBlock
                            || (kind == ScopeKind::Function
                                && matches!(ast.kind_data(new), NodeKindData::Function(_)));
                    }
                }
            }
        }
        self.node_scopes[new.index()] = current;
        self.node_scopes[old.index()] = previous;
    }
}

impl<'a> Semantic<'a> {
    /// Merge a binding-free scope using the existing semantic indexes.
    /// Nodes, references and nested scopes retain their identities.
    #[doc(hidden)]
    pub fn merge_scope(&mut self, source: ScopeId, parent: ScopeId) {
        let scope = self.scope(source);
        let target = self.scope(parent);
        assert!(
            scope.active && target.active,
            "source and target scopes must be active"
        );
        assert!(
            scope.parent.is_some(),
            "cannot merge the root scope into another scope"
        );
        assert!(
            self.scope_maps[source.index()].is_empty(),
            "source scope must have no bindings"
        );
        assert!(
            !self.scopes.ancestors(Some(parent)).any(|id| id == source),
            "merge target cannot be inside the source scope"
        );
        assert!(
            scope.kind.is_hoist_target() || scope.hoist_target == target.hoist_target,
            "merging a block must preserve its hoist target"
        );

        for child in &mut self.scopes.list {
            if !child.active {
                continue;
            }
            if child.parent == Some(source) {
                child.parent = Some(parent);
            }
            if child.hoist_target == source {
                child.hoist_target = target.hoist_target;
            }
        }
        for node_scope in &mut self.node_scopes {
            if *node_scope == Some(source) {
                *node_scope = Some(parent);
            }
        }
        for reference in &mut self.references {
            if reference.scope == source {
                reference.scope = parent;
            }
        }
        self.remove_scope(source);
        if scope.flags.direct_eval {
            self.scopes.mark_direct_eval(parent);
        }
    }
}

impl<'a> Semantic<'a> {
    /// Updates only the export target affected by an expression-edge replacement.
    #[doc(hidden)]
    pub fn replace_export_target(&mut self, ast: &Ast<'a>, statement: NodeId, replacement: NodeId) {
        use swc_next_ecma_ast::NodeKindData;
        let reference = self.reference_of(replacement);
        let symbol = reference.and_then(|id| self.reference(id).symbol);
        let (declaration, binding) = match ast.kind_data(replacement) {
            NodeKindData::Class(node) => (true, node.id(ast)),
            NodeKindData::Function(node) => (true, node.id(ast)),
            _ => (false, None),
        };
        let declaration_symbol = binding.and_then(|node| self.symbol_of(node.node_id()));
        self.module_record.replace_target(
            statement,
            replacement,
            declaration,
            reference,
            symbol,
            declaration_symbol,
        );
    }
}

impl Semantic<'_> {
    fn resolve_mutated_reference(
        &self,
        ast: &Ast<'_>,
        reference: Reference,
    ) -> (Option<SymbolId>, ReferenceFlags) {
        let name = ast.get_utf8(reference.name).as_bytes();
        let arguments =
            name == b"arguments" && reference.flags.space == crate::ReferenceSpace::Value;
        let mut symbol = None;
        let mut flags = reference.flags.without_dynamic();
        for scope in self.scopes.ancestors(Some(reference.scope)) {
            if let Some(binding) = self.binding(scope, name)
                && self.symbol(binding).flags.visible_in(reference.flags.space)
            {
                symbol = Some(binding);
                break;
            }
            let current = self.scope(scope);
            if arguments && current.flags.arguments_barrier {
                break;
            }
            if reference.flags.space == crate::ReferenceSpace::Value
                && current.kind == ScopeKind::With
            {
                flags = flags.with_dynamic();
            }
        }
        (symbol, flags)
    }

    /// A declaration introduced into an existing scope can capture references
    /// that were previously unresolved or resolved to an outer declaration.
    fn refresh_named_references(&mut self, ast: &Ast<'_>, name: AstUtf8) {
        let mut affected = rustc_hash::FxHashMap::<SymbolId, Vec<ReferenceId>>::default();
        for index in 0..self.references.len() {
            let reference = self.references[index];
            if ast.get_utf8(reference.name) != ast.get_utf8(name)
                || self.node_references[reference.node.index()]
                    != Some(ReferenceId::from_usize(index))
            {
                continue;
            }
            let (symbol, flags) = self.resolve_mutated_reference(ast, reference);
            self.references[index].flags = flags;
            if reference.symbol != symbol {
                if let Some(old) = reference.symbol {
                    affected.entry(old).or_default();
                }
                if let Some(new) = symbol {
                    affected
                        .entry(new)
                        .or_default()
                        .push(ReferenceId::from_usize(index));
                }
                self.references[index].symbol = symbol;
            }
        }
        self.rebuild_changed_uses(affected);
    }

    fn rebuild_changed_uses(
        &mut self,
        affected: rustc_hash::FxHashMap<SymbolId, Vec<ReferenceId>>,
    ) {
        if affected.is_empty() {
            return;
        }
        for (symbol, added) in affected {
            let range = self.use_ranges[symbol.index()];
            let start = self.use_ids.len();
            for offset in 0..range.len as usize {
                let id = self.use_ids[range.start as usize + offset];
                let reference = self.references[id.index()];
                if reference.symbol == Some(symbol)
                    && self.node_references[reference.node.index()] == Some(id)
                {
                    self.use_ids.push(id);
                }
            }
            self.use_ids.extend(added);
            self.use_ids[start..].sort_unstable();
            self.use_ranges[symbol.index()] = Range {
                start: start.try_into().expect("reference storage overflow"),
                len: (self.use_ids.len() - start)
                    .try_into()
                    .expect("reference storage overflow"),
            };
        }
        self.module_record.refresh_references(&self.references);
    }
}
