//! Low-level index operations composed by the transformer context.
//!
//! Callers must update AST edges in the same operation. These methods do not
//! expose the model's mutable tables.

use oxc_index::Idx;
use swc_next_allocator::CloneIn;
use swc_next_allocator::hash_map::HashMap as ArenaHashMap;
use swc_next_ecma_ast::{
    Ast, AstUtf8, BindingIdentifier, ClassElement, IdentifierReference, NodeId, NodeKindData,
    ReferenceId, ScopeId, SymbolId,
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
        if self
            .features
            .contains(crate::SemanticFeatureFlags::NODE_SCOPES)
        {
            self.node_scopes.resize(node_count, None);
        }
    }

    /// Register freshly allocated nodes in their evaluation scope.
    #[doc(hidden)]
    pub fn register_nodes(&mut self, ast: &Ast<'a>, start: usize, scope: ScopeId) {
        self.grow_node_indexes(ast.node_count());
        let mut eval_changed = false;
        for index in start..ast.node_count() {
            let node = NodeId::from_usize(index);
            if let Some(slot) = self.node_scopes.get_mut(index) {
                *slot = Some(scope);
            }
            eval_changed |= self.update_eval_call(ast, node, scope);
        }
        if eval_changed {
            self.refresh_direct_eval();
        }
    }

    /// Refresh metadata determined by a call's immediate children after an edge edit.
    #[doc(hidden)]
    pub fn refresh_node(&mut self, ast: &Ast<'_>, node: NodeId) {
        if matches!(ast.kind_data(node), NodeKindData::ClassBody(_)) {
            self.invalidate_static_block_layout(node);
            return;
        }
        if !matches!(
            ast.kind_data(node),
            swc_next_ecma_ast::NodeKindData::CallExpression(_)
        ) {
            return;
        }
        if let Some(scope) = self
            .scope_at(node)
            .or_else(|| self.direct_eval_calls.get(&node).copied())
            && self.update_eval_call(ast, node, scope)
        {
            self.refresh_direct_eval();
        }
    }

    /// Returns whether the indexed call or its evaluation scope changed.
    fn update_eval_call(&mut self, ast: &Ast<'_>, node: NodeId, scope: ScopeId) -> bool {
        let swc_next_ecma_ast::NodeKindData::CallExpression(call) = ast.kind_data(node) else {
            return false;
        };
        if crate::builder::is_direct_eval_call(ast, call) {
            if self.direct_eval_calls.get(&node) == Some(&scope) {
                return false;
            }
            self.direct_eval_calls.insert(node, scope);
            true
        } else {
            self.direct_eval_calls.remove(&node).is_some()
        }
    }

    fn refresh_direct_eval(&mut self) {
        for scope in &mut self.scopes.list {
            scope.flags.direct_eval = false;
        }
        for &scope in self.direct_eval_calls.values() {
            let mut ancestor = Some(scope);
            while let Some(id) = ancestor {
                let scope = &mut self.scopes.list[id.index()];
                if scope.flags.direct_eval {
                    break;
                }
                scope.flags.direct_eval = true;
                ancestor = scope.parent;
            }
        }
    }

    /// Reserves and interns a name as one operation.
    #[doc(hidden)]
    pub fn generate_name(&mut self, ast: &mut Ast<'a>, base: &str, start_suffix: u64) -> AstUtf8 {
        ast.add_utf8(self.used_names.next_name(base, start_suffix))
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
        if let Some(slot) = self.node_scopes.get_mut(node.node_id().index()) {
            *slot = Some(scope);
        }
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
        self.used_names.observe(bytes);
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
        if let Some(slot) = self.node_scopes.get_mut(node.node_id().index()) {
            *slot = Some(scope);
        }
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

    /// Changes an existing reference without changing its ReferenceId.
    #[doc(hidden)]
    pub fn retarget_reference(&mut self, node: NodeId, name: AstUtf8, symbol: SymbolId) {
        let id = self
            .reference_of(node)
            .expect("retargeted node has a reference");
        self.unlink_reference_use(id);
        self.references[id.index()].name = name;
        self.references[id.index()].symbol = Some(symbol);
        self.append_use(symbol, id);
    }

    fn unlink_reference_use(&mut self, id: ReferenceId) {
        if let Some(old_symbol) = self.references[id.index()].symbol {
            let range = self.use_ranges[old_symbol.index()];
            let start = self.use_ids.len();
            for offset in 0..range.len as usize {
                let previous = self.use_ids[range.start as usize + offset];
                if previous != id {
                    self.use_ids.push(previous);
                }
            }
            self.use_ranges[old_symbol.index()] = Range {
                start: start.try_into().expect("reference storage overflow"),
                len: (self.use_ids.len() - start)
                    .try_into()
                    .expect("reference storage overflow"),
            };
        }
    }

    /// Update only the semantic records of a replaced leaf expression.
    /// The old AST storage and structural scope entry remain allocated.
    #[doc(hidden)]
    pub fn replace_leaf_expression(&mut self, old: swc_next_ecma_ast::NodeKindData) {
        let node = old.node_id();
        if let Some(reference) = self.node_references[node.index()].take() {
            self.unlink_reference_use(reference);
        }
        if matches!(old, swc_next_ecma_ast::NodeKindData::ThisExpression(_)) {
            let scope = self.scope_outside(node);
            self.class.replace_this(
                node,
                self.scopes
                    .ancestors(scope)
                    .map(|id| self.scopes.list[id.index()].node),
            );
        }
    }

    /// Update indexes when replacing a class element shell. Property and method
    /// replacements must preserve their children and evaluation environment.
    /// Static block contents must already have been moved; only empty blocks
    /// may be discarded without transferring their scope.
    #[doc(hidden)]
    pub fn replace_class_element(
        &mut self,
        ast: &Ast<'a>,
        old: ClassElement,
        replacement: Option<ClassElement>,
    ) {
        if replacement.map(|node| node.node_id()) == Some(old.node_id()) {
            return;
        }
        match ast.kind_data(old.node_id()) {
            NodeKindData::StaticBlock(block) => self.replace_static_block(ast, block),
            NodeKindData::PropertyDefinition(property) => {
                let Some(new) = replacement.and_then(|node| node.as_property_definition(ast))
                else {
                    debug_assert!(false, "property replacement must preserve its children");
                    return;
                };
                debug_assert_eq!(
                    self.scope_at(property.node_id()),
                    self.scope_at(new.node_id())
                );
                debug_assert_eq!(property.key(ast), new.key(ast));
                debug_assert_eq!(property.value(ast), new.value(ast));
                debug_assert_eq!(property.type_annotation(ast), new.type_annotation(ast));
                debug_assert!(
                    property
                        .decorators(ast)
                        .iter()
                        .map(|id| ast.get_node_in_sub_range(id))
                        .eq(new
                            .decorators(ast)
                            .iter()
                            .map(|id| ast.get_node_in_sub_range(id)))
                );
                debug_assert_eq!(property.computed(ast), new.computed(ast));
                debug_assert_eq!(property.r#static(ast), new.r#static(ast));
                debug_assert_eq!(property.declare(ast), new.declare(ast));
                debug_assert_eq!(property.r#abstract(ast), new.r#abstract(ast));
                self.class
                    .replace_property(property.node_id(), new.node_id());
            }
            NodeKindData::MethodDefinition(method) => {
                let Some(new) = replacement.and_then(|node| node.as_method_definition(ast)) else {
                    debug_assert!(false, "method replacement must preserve its children");
                    return;
                };
                debug_assert_eq!(
                    self.scope_at(method.node_id()),
                    self.scope_at(new.node_id())
                );
                debug_assert_eq!(method.key(ast), new.key(ast));
                debug_assert_eq!(method.value(ast), new.value(ast));
                debug_assert!(
                    method
                        .decorators(ast)
                        .iter()
                        .map(|id| ast.get_node_in_sub_range(id))
                        .eq(new
                            .decorators(ast)
                            .iter()
                            .map(|id| ast.get_node_in_sub_range(id)))
                );
                debug_assert_eq!(method.kind(ast), new.kind(ast));
                debug_assert_eq!(method.computed(ast), new.computed(ast));
                debug_assert_eq!(method.r#static(ast), new.r#static(ast));
            }
            _ => debug_assert!(false, "unsupported class element replacement"),
        }
        self.invalidate_static_block_layout(old.node_id());
    }

    fn invalidate_static_block_layout(&mut self, node: NodeId) {
        if !self
            .features
            .contains(crate::SemanticFeatureFlags::STATIC_BLOCK_LAYOUT)
        {
            return;
        }
        let class = self
            .scopes
            .ancestors(self.scope_at(node))
            .find_map(|scope| {
                let scope = self.scope(scope);
                (scope.kind == ScopeKind::Class).then_some(scope.node)
            });
        if let Some(class) = class {
            self.class.invalidate_static_block_layout(class);
        }
    }

    /// Finish replacing a static block whose contents have already been moved.
    /// Only an empty block can still own its original scope at this point.
    fn replace_static_block(&mut self, ast: &Ast<'a>, block: swc_next_ecma_ast::StaticBlock) {
        let node = block.node_id();
        if let Some(scope) = self.scope_at(node)
            && self.scope(scope).active
            && self.scope(scope).node == node
        {
            debug_assert!(
                block.body(ast).is_empty(),
                "static block contents must be moved first"
            );
            self.scopes.list[scope.index()].active = false;
        }
        self.class.replace_static_block(node);
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
        if let Some(slot) = self.node_scopes.get_mut(owner.index()) {
            *slot = Some(id);
        }
        self.scope_maps.push(ArenaHashMap::new_in(ast.allocator()));
        self.hoisting_variables.push(None);
        id
    }

    /// Replaces a scope owner, clearing metadata attached to the previous node.
    #[doc(hidden)]
    pub fn replace_scope_owner(
        &mut self,
        ast: &Ast<'_>,
        old: NodeId,
        new: NodeId,
        kind: ScopeKind,
    ) {
        let scope = self.scope_outside(old);
        self.class.remove_arrow(
            old,
            self.scopes
                .ancestors(scope)
                .map(|id| self.scopes.list[id.index()].node),
        );
        self.transfer_scope_owner(ast, old, new, kind);
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
            self.refresh_direct_eval();
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
        if let Some(slot) = self.node_scopes.get_mut(new.index()) {
            *slot = current;
        }
        if let Some(slot) = self.node_scopes.get_mut(old.index()) {
            *slot = previous;
        }
    }
}

impl<'a> Semantic<'a> {
    #[doc(hidden)]
    pub fn reserve_private_key(
        &mut self,
        ast: &mut Ast<'a>,
        class: NodeId,
        scope: ScopeId,
    ) -> swc_next_ecma_ast::PropertyKey {
        let name = self.class.next_private_name(ast, class);
        let start = ast.node_count();
        let key = ast.property_key_expr_private_identifier(swc_next_ecma_ast::DUMMY_SP, name);
        self.register_nodes(ast, start, scope);
        key
    }

    #[doc(hidden)]
    pub fn record_arrow_environment(&mut self, arrow: NodeId, environment: crate::ClassArrowInfo) {
        self.class.record_arrow_environment(arrow, environment);
    }

    /// Merge a binding-free scope using the existing semantic indexes.
    /// Nodes, references and nested scopes retain their identities.
    #[doc(hidden)]
    pub fn merge_scope(
        &mut self,
        source: ScopeId,
        parent: ScopeId,
        environment: crate::ClassArrowInfo,
    ) {
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

        self.class
            .inherit_scope_environment(scope.node, environment);
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
        for eval_scope in self.direct_eval_calls.values_mut() {
            if *eval_scope == source {
                *eval_scope = parent;
            }
        }
        self.scopes.list[source.index()].active = false;
        self.scopes.list[source.index()].flags.direct_eval = false;
        self.hoisting_variables[source.index()] = None;
        if scope.flags.direct_eval && scope.parent != Some(parent) {
            self.refresh_direct_eval();
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
    /// Reattach lookup views whose type wrappers were removed by normalization.
    #[doc(hidden)]
    pub fn normalize_scope_owners(&mut self, ast: &Ast<'_>) {
        for scope in &mut self.scopes.list {
            loop {
                let replacement = match ast.kind_data(scope.node) {
                    NodeKindData::TsParenthesizedType(node) => {
                        Some(node.type_annotation(ast).node_id())
                    }
                    NodeKindData::TsUnionType(node) if node.types(ast).len() == 1 => {
                        node.types(ast).get_node(ast, 0).map(|node| node.node_id())
                    }
                    NodeKindData::TsIntersectionType(node) if node.types(ast).len() == 1 => {
                        node.types(ast).get_node(ast, 0).map(|node| node.node_id())
                    }
                    _ => None,
                };
                let Some(replacement) = replacement else {
                    break;
                };
                scope.node = replacement;
            }
        }
    }

    /// Refreshes export expression roots after shape-only normalization.
    #[doc(hidden)]
    pub fn normalize_exports(&mut self, ast: &Ast<'_>) {
        self.module_record
            .normalize_targets(ast, &self.node_references, &self.references);
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

#[cfg(test)]
mod tests;
