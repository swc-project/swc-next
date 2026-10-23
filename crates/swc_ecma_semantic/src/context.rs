use oxc_index::Idx;
use swc_next_ecma_ast::{
    ArrowFunctionExpression, Ast, Class, Directive, Function, FunctionBody, NodeId, NodeKindData,
    ScopeId,
};

use crate::{
    Semantic,
    binder::{ReferenceFlags, ReferenceSpace, SymbolTracker},
    ecmascript,
    scope::{Scope, ScopeFlags, ScopeKind, ScopeTree},
};

pub(crate) enum DecoratorSite {
    ClassDeclaration,
    ClassExpression,
    Member,
}

/// Walk context combining the scope tree, current scope, symbol tracker,
/// and reference state. Visitor hooks use it to inspect surrounding source
/// structure or to query and contribute symbols.
pub(crate) struct SemanticContext<'a> {
    pub scope: ScopeTree<'a>,
    pub symbols: SymbolTracker<'a>,
    pub current_scope: ScopeId,
    // One saved scope for each nested decorator that retargets the cursor.
    decorator_saved: Vec<ScopeId>,
    // Type syntax and erased value expressions save and restore this default.
    default_reference_flags: ReferenceFlags,
    ts_namespace_depth: u32,
    reference_flags: ReferenceFlags,
}

impl<'a> SemanticContext<'a> {
    pub fn init(tree: &Ast<'a>) -> Self {
        let scope = ScopeTree::init(tree);
        let current_scope = ScopeId::from_usize(scope.list.len() - 1);
        Self {
            scope,
            current_scope,
            decorator_saved: Vec::new(),
            symbols: SymbolTracker::init(tree),
            default_reference_flags: ReferenceFlags::default(),
            ts_namespace_depth: 0,
            reference_flags: ReferenceFlags::default(),
        }
    }

    /// Type syntax uses type defaults; erased computed keys use expression defaults.
    pub fn in_type_position(&self) -> bool {
        self.default_reference_flags.space == ReferenceSpace::Type
    }

    /// True when the walker is currently inside a TS namespace body.
    pub fn in_ts_namespace(&self) -> bool {
        self.ts_namespace_depth > 0
    }

    #[inline]
    pub fn enter_node(&mut self) -> ReferenceFlags {
        let flags = self.reference_flags;
        self.reference_flags = self.default_reference_flags();
        flags
    }

    #[inline]
    pub fn default_reference_flags(&self) -> ReferenceFlags {
        self.default_reference_flags
    }

    #[inline]
    pub fn replace_reference_flags(&mut self, flags: ReferenceFlags) -> ReferenceFlags {
        std::mem::replace(&mut self.reference_flags, flags)
    }

    #[inline]
    pub fn replace_default_reference_flags(&mut self, flags: ReferenceFlags) -> ReferenceFlags {
        std::mem::replace(&mut self.default_reference_flags, flags)
    }

    #[inline]
    pub fn enter_type_position(&mut self) -> ReferenceFlags {
        let flags = ReferenceFlags::default()
            .with_space(ReferenceSpace::Type)
            .no_access();
        std::mem::replace(&mut self.default_reference_flags, flags)
    }

    #[inline]
    pub fn exit_type_position(&mut self, previous: ReferenceFlags) {
        debug_assert!(self.in_type_position());
        self.default_reference_flags = previous;
        self.reference_flags = previous;
    }

    fn push_scope(&mut self, kind: ScopeKind, node: NodeId, flags: ScopeFlags) {
        self.current_scope = self.scope.push_scope(self.current_scope, kind, node, flags);
    }

    fn pop_scope(&mut self) {
        // Root and its source-type wrapper are never popped directly by the ordinary walk.
        self.current_scope = self
            .current_scope()
            .parent
            .expect("non-root scopes have a parent");
    }

    pub fn enter_directive(&mut self, tree: &Ast, directive: Directive) {
        if tree.get_utf8(directive.value(tree)) == "use strict" {
            self.current_scope_mut().flags.strict = true;
        }
    }

    pub fn enter_function(&mut self, tree: &Ast, function: Function) {
        let flags = if self.has_retroactive_use_strict(tree, function.body(tree)) {
            ScopeFlags {
                strict: true,
                ..ScopeFlags::default()
            }
        } else {
            self.inherit_strict_flag()
        };
        self.push_function_scope(function.node_id(), flags);
    }

    pub fn enter_ts_function(&mut self, node: NodeId) {
        self.push_function_scope(node, self.inherit_strict_flag());
    }

    // Ordinary functions own arguments whether or not they have a body.
    // Arrows, function types and interface signatures use other scope entries.
    fn push_function_scope(&mut self, node: NodeId, flags: ScopeFlags) {
        self.push_scope(
            ScopeKind::Function,
            node,
            ScopeFlags {
                arguments_barrier: true,
                ..flags
            },
        );
    }

    pub fn enter_arrow_function_expression(&mut self, tree: &Ast, arrow: ArrowFunctionExpression) {
        let body = arrow.body(tree).node_id();
        let flags = match tree.kind_data(body) {
            NodeKindData::FunctionBody(body) if self.has_retroactive_use_strict(tree, body) => {
                ScopeFlags {
                    strict: true,
                    ..ScopeFlags::default()
                }
            }
            _ => self.inherit_strict_flag(),
        };
        self.push_scope(ScopeKind::Function, arrow.node_id(), flags);
    }

    pub fn enter_function_body(&mut self, tree: &Ast, body: FunctionBody, parent: Option<NodeId>) {
        // Pushed only when the parent function's parameter list
        // contains expressions; see ScopeKind::FunctionBody.
        let Some(parent) = parent else { return };
        let params = match tree.kind_data(parent) {
            NodeKindData::Function(function) => function.params(tree),
            NodeKindData::ArrowFunctionExpression(arrow) => arrow.params(tree),
            _ => return,
        };
        if ecmascript::find_parameter_expression(tree, params).is_some() {
            let function = self.current_scope;
            self.push_scope(
                ScopeKind::FunctionBody,
                body.node_id(),
                self.inherit_strict_flag(),
            );
            self.scope.list[function.index()].function_body = Some(self.current_scope);
        }
    }

    pub fn enter_block_statement(&mut self, tree: &Ast, node: NodeId) {
        // 14.15.2 gives catch parameters and the block separate
        // environments. Folding them is unobservable and makes the
        // 14.15.1 conflict a single-scope lookup, except when a
        // parameter expression runs before the block environment:
        //
        //   catch (e)       { let e }   // shared, conflict found
        //   catch ([e = 1]) { let e }   // split; default cannot see let
        let catch = match tree.kind_data(self.current_scope().node) {
            NodeKindData::CatchClause(catch) => Some(catch),
            _ => None,
        };
        let shares_catch_scope = self.current_scope().kind == ScopeKind::Block
            && catch.is_some_and(|catch| {
                catch.body(tree).node_id() == node
                    && catch.param(tree).is_none_or(|param| {
                        ecmascript::find_pattern_expression(tree, Some(param.node_id())).is_none()
                    })
            });
        if !shares_catch_scope {
            self.enter_block_scope(node);
        }
    }

    pub fn enter_switch_case(&mut self, parent: Option<NodeId>) {
        // One case-block scope per switch, shared by every case and
        // created at the first case. Per 14.12.4, the discriminant
        // evaluates before this environment exists:
        //
        //   switch (x) { case 1: let x }
        //   //      ^ outer scope    ^ case-block scope
        let parent = parent.expect("a switch case has a parent");
        if self.current_scope().node != parent {
            self.enter_block_scope(parent);
        }
    }

    pub fn enter_ts_enum_body(&mut self, node: NodeId) {
        // TSC resolves unqualified member references inside the body.
        self.enter_block_scope(node);
    }

    fn enter_block_scope(&mut self, node: NodeId) {
        self.push_scope(ScopeKind::Block, node, self.inherit_strict_flag());
    }

    pub fn enter_with_statement(&mut self, node: NodeId) {
        self.push_scope(ScopeKind::With, node, self.inherit_strict_flag());
    }

    pub fn exit_with_statement(&mut self, node: NodeId) {
        let scope = self.current_scope();
        debug_assert_eq!(scope.node, node);
        debug_assert_eq!(scope.kind, ScopeKind::With);
        self.pop_scope();
    }

    pub fn enter_ts_lookup_view(&mut self, node: NodeId, kind: ScopeKind) {
        debug_assert!(kind.is_ts_lookup_view());
        self.push_scope(kind, node, self.inherit_strict_flag());
    }

    pub fn exit_ts_lookup_view(&mut self, node: NodeId) {
        debug_assert_eq!(self.current_scope().node, node);
        debug_assert!(self.current_scope().kind.is_ts_lookup_view());
        self.pop_scope();
    }

    pub fn enter_ts_module_block(&mut self, node: NodeId) {
        self.ts_namespace_depth += 1;
        self.push_scope(ScopeKind::TsModule, node, self.inherit_strict_flag());
    }

    pub fn enter_class(&mut self, class: Class) {
        // Classes are always strict mode (section 15.7.14).
        let flags = ScopeFlags {
            strict: true,
            ..ScopeFlags::default()
        };
        self.push_scope(ScopeKind::Class, class.node_id(), flags);
    }

    pub fn enter_static_block(&mut self, node: NodeId) {
        self.push_scope(ScopeKind::StaticBlock, node, self.inherit_strict_flag());
        self.current_scope_mut().flags.arguments_barrier = true;
    }

    pub fn enter_decorator(&mut self, node: NodeId, site: DecoratorSite) {
        self.decorator_saved.push(self.current_scope);
        let Some((class, outer)) = self.decorator_scopes() else {
            return;
        };
        match site {
            DecoratorSite::ClassDeclaration => self.current_scope = outer,
            DecoratorSite::ClassExpression => {
                // Decorators precede ClassTail and do not inherit its strict
                // mode, even though TS name lookup can see this class.
                let flags = ScopeFlags {
                    strict: self.scope.get(outer).flags.strict,
                    ..ScopeFlags::default()
                };
                self.current_scope = class;
                self.push_scope(ScopeKind::ClassDecorator, node, flags);
            }
            DecoratorSite::Member => self.current_scope = class,
        }
    }

    fn decorator_scopes(&self) -> Option<(ScopeId, ScopeId)> {
        for class in self.scope.ancestors(Some(self.current_scope)) {
            if self.scope.get(class).kind != ScopeKind::Class {
                continue;
            }
            let outer = self.scope.get(class).parent?;
            return Some((class, outer));
        }
        None
    }

    // "use strict" applies retroactively to the whole function including
    // its parameter list, so peek into the body and set the flag at
    // scope-creation time, before any parameter is visited
    fn has_retroactive_use_strict(&self, tree: &Ast, body: FunctionBody) -> bool {
        for directive in body.directives(tree).iter() {
            let directive = tree.get_node_in_sub_range(directive);
            if tree.get_utf8(directive.value(tree)) == "use strict" {
                return true;
            }
        }
        false
    }

    #[inline]
    fn inherit_strict_flag(&self) -> ScopeFlags {
        ScopeFlags {
            strict: self.current_scope().flags.strict,
            ..ScopeFlags::default()
        }
    }

    // A node owns its lexical scopes.
    // Lookup views have a separate lifetime: their root node may
    // create lexical scopes without owning the surrounding lookup view.
    pub fn exit_scope(&mut self, node: NodeId) {
        while self.current_scope.index() != 0
            && self.current_scope().node == node
            && !self.current_scope().kind.is_ts_lookup_view()
        {
            self.pop_scope();
        }
    }

    pub fn exit_decorator(&mut self) {
        // Every decorator enter saved exactly one scope.
        self.current_scope = self
            .decorator_saved
            .pop()
            .expect("every decorator enter saved exactly one scope");
    }

    /// The current scope.
    #[inline]
    pub fn current_scope(&self) -> Scope {
        self.scope.get(self.current_scope)
    }

    #[inline]
    // Mutable view used while collecting strict-mode flags.
    fn current_scope_mut(&mut self) -> &mut Scope {
        &mut self.scope.list[self.current_scope.index()]
    }

    /// Whether the current scope is in strict mode.
    #[inline]
    pub fn is_strict(&self) -> bool {
        self.current_scope().flags.strict
    }

    pub fn enter_for_statement(&mut self, node: NodeId) {
        self.enter_block_scope(node);
    }

    pub fn enter_for_in_statement(&mut self, node: NodeId) {
        self.enter_block_scope(node);
    }

    pub fn enter_for_of_statement(&mut self, node: NodeId) {
        self.enter_block_scope(node);
    }

    pub fn enter_catch_clause(&mut self, node: NodeId) {
        self.enter_block_scope(node);
    }

    pub fn enter_ts_interface_declaration(&mut self, node: NodeId) {
        self.enter_block_scope(node);
    }

    pub fn enter_ts_type_alias_declaration(&mut self, node: NodeId) {
        self.enter_block_scope(node);
    }

    pub fn enter_ts_function_type(&mut self, node: NodeId) {
        self.enter_block_scope(node);
    }

    pub fn enter_ts_constructor_type(&mut self, node: NodeId) {
        self.enter_block_scope(node);
    }

    pub fn enter_ts_method_signature(&mut self, node: NodeId) {
        self.enter_block_scope(node);
    }

    pub fn enter_ts_call_signature_declaration(&mut self, node: NodeId) {
        self.enter_block_scope(node);
    }

    pub fn enter_ts_construct_signature_declaration(&mut self, node: NodeId) {
        self.enter_block_scope(node);
    }

    pub fn enter_ts_index_signature(&mut self, node: NodeId) {
        self.enter_block_scope(node);
    }

    pub fn enter_ts_mapped_type(&mut self, node: NodeId) {
        self.enter_block_scope(node);
    }

    pub fn enter_ts_conditional_type(&mut self, node: NodeId) {
        self.enter_block_scope(node);
    }

    pub fn exit_ts_module_block(&mut self, node: NodeId) {
        debug_assert!(self.ts_namespace_depth > 0);
        self.exit_scope(node);
        self.ts_namespace_depth -= 1;
    }

    pub fn finalize(self, tree: &Ast<'a>) -> Semantic<'a> {
        self.symbols.finalize(tree, self.scope)
    }
}
