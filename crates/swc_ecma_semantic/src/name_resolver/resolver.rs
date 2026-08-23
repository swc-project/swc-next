use oxc_index::{Idx, IndexVec};
use rustc_hash::FxHashSet;
use swc_next_allocator::{Allocator, hash_map::HashMap as ArenaHashMap, vec::Vec as ArenaVec};
use swc_next_ecma_ast::*;

use super::{
    scope::{DeclKind, ScopeKind},
    utils::find_binding_ids,
};

/// Resolve every value identifier in `ast` using the legacy resolver's
/// hoist-then-resolve algorithm.
pub fn resolver<'ast>(ast: &'ast Ast<'ast>) -> JsNameResolver<'ast> {
    let allocator = ast.allocator();
    let mut scopes = IndexVec::new();
    let unresolved_scope_id = scopes.push(Scope::new(ScopeKind::Fn, None, allocator));
    let top_level_scope_id = scopes.push(Scope::new(ScopeKind::Fn, None, allocator));
    let label_scope_id = ScopeId::from_usize(ScopeId::MAX);

    let mut node_scopes = ArenaVec::with_capacity_in(ast.node_count(), allocator);
    node_scopes.resize(ast.node_count(), None);

    let mut resolver = Resolver {
        ast,
        scopes,
        node_scopes,
        top_level_scope_id,
        unresolved_scope_id,
        label_scope_id,
        current: top_level_scope_id,
        is_module: false,
        decl_kind: DeclKind::Lexical,
        strict_mode: false,
    };

    ast.root_program().visit_with(&mut resolver);

    JsNameResolver {
        top_level_scope_id,
        unresolved_scope_id,
        label_scope_id,
        node_scopes: resolver.node_scopes,
    }
}

/// Scope assignments produced by [`resolver`].
///
/// The old tree AST stored a symbol id on every `Ident`. The flat AST already
/// gives every binding, reference, and label a stable [`NodeId`], so the same
/// information is kept in a side table indexed by that id.
#[derive(Debug)]
pub struct JsNameResolver<'ast> {
    top_level_scope_id: ScopeId,
    unresolved_scope_id: ScopeId,
    label_scope_id: ScopeId,
    node_scopes: ArenaVec<'ast, Option<ScopeId>>,
}

impl JsNameResolver<'_> {
    /// Returns the resolved scope for an identifier-like node. Nodes that were
    /// not visited as bindings or references use the unresolved scope.
    #[inline]
    pub fn node_scope<N: AstNode>(&self, node: N) -> ScopeId {
        self.node_scopes[node.node_id().index()].unwrap_or(self.unresolved_scope_id)
    }

    #[inline]
    pub fn top_level_scope_id(&self) -> ScopeId {
        self.top_level_scope_id
    }

    #[inline]
    pub fn unresolved_scope_id(&self) -> ScopeId {
        self.unresolved_scope_id
    }

    #[inline]
    pub fn label_scope_id(&self) -> ScopeId {
        self.label_scope_id
    }
}

#[derive(Debug)]
struct Scope<'ast> {
    parent: Option<ScopeId>,
    kind: ScopeKind,
    declared_symbols: ArenaHashMap<'ast, &'ast [u8], DeclKind>,
}

impl<'ast> Scope<'ast> {
    fn new(kind: ScopeKind, parent: Option<ScopeId>, allocator: &'ast Allocator) -> Self {
        Self {
            parent,
            kind,
            declared_symbols: ArenaHashMap::new_in(allocator),
        }
    }
}

/// The resolver has the same two phases as the legacy implementation:
/// declarations visible in a scope are hoisted first, then references are
/// resolved while walking the source in order.
struct Resolver<'ast> {
    ast: &'ast Ast<'ast>,
    scopes: IndexVec<ScopeId, Scope<'ast>>,
    node_scopes: ArenaVec<'ast, Option<ScopeId>>,
    top_level_scope_id: ScopeId,
    unresolved_scope_id: ScopeId,
    label_scope_id: ScopeId,
    current: ScopeId,
    is_module: bool,
    decl_kind: DeclKind,
    strict_mode: bool,
}

impl<'ast> Resolver<'ast> {
    fn name(&self, name: AstUtf8) -> &'ast [u8] {
        self.ast.get_utf8(name).as_bytes()
    }

    fn is_declared(&self, name: &[u8], start_scope: ScopeId) -> Option<DeclKind> {
        let mut scope = Some(start_scope);
        while let Some(scope_id) = scope {
            let current = &self.scopes[scope_id];
            if let Some(kind) = current.declared_symbols.get(name) {
                return Some(*kind);
            }
            scope = current.parent;
        }
        None
    }

    fn with_child<F>(&mut self, kind: ScopeKind, owner: Option<NodeId>, op: F)
    where
        F: FnOnce(&mut Self),
    {
        let scope = self.current;
        let is_module = self.is_module;
        let decl_kind = self.decl_kind;
        let strict_mode = self.strict_mode;

        let child_scope = self
            .scopes
            .push(Scope::new(kind, Some(scope), self.ast.allocator()));
        self.current = child_scope;
        if let Some(owner) = owner {
            self.record_scope(owner, child_scope);
        }
        op(self);

        self.current = scope;
        self.is_module = is_module;
        self.decl_kind = decl_kind;
        self.strict_mode = strict_mode;
    }

    fn record_scope(&mut self, node: NodeId, scope: ScopeId) {
        let slot = &mut self.node_scopes[node.index()];
        if slot.is_none() {
            *slot = Some(scope);
        }
    }

    fn mark_current(&mut self, node: NodeId) {
        self.record_scope(node, self.current);
    }

    fn mark_for_ref(&self, name: &[u8]) -> Option<ScopeId> {
        self.mark_for_ref_inner(name, false)
    }

    fn mark_for_ref_inner(&self, name: &[u8], stop_at_fn_scope: bool) -> Option<ScopeId> {
        let mut scope = Some(self.current);
        while let Some(scope_id) = scope {
            let current = &self.scopes[scope_id];
            if current.declared_symbols.contains_key(name) {
                return if (name == b"undefined" || name == b"NaN" || name == b"Infinity")
                    && scope_id == self.top_level_scope_id
                    && !self.is_module
                {
                    Some(self.unresolved_scope_id)
                } else {
                    Some(scope_id)
                };
            }
            if current.kind == ScopeKind::Fn && stop_at_fn_scope {
                return None;
            }
            scope = current.parent;
        }
        None
    }

    fn modify(&mut self, identifier: BindingIdentifier, kind: DeclKind) {
        if self.node_scopes[identifier.node_id().index()].is_some() {
            return;
        }

        let name = self.name(identifier.name(self.ast));
        self.scopes[self.current]
            .declared_symbols
            .insert(name, kind);
        self.mark_current(identifier.node_id());
    }

    fn has_use_strict(&self, directives: TypedSubRange<Directive>) -> bool {
        directives.iter().any(|id| {
            let directive = self.ast.get_node_in_sub_range(id);
            self.name(directive.value(self.ast)) == b"use strict"
        })
    }

    fn visit_function_contents(&mut self, function: Function) {
        let parameters = function.params(self.ast);
        let mut identifiers = std::vec::Vec::new();
        for id in parameters.items(self.ast).iter() {
            let item = self.ast.get_node_in_sub_range(id);
            let FormalParameterItemData::FormalParameter(parameter) =
                self.ast.formal_parameter_item_data(item)
            else {
                continue;
            };
            let FormalParameterPatternData::BindingPattern(pattern) = self
                .ast
                .formal_parameter_pattern_data(parameter.pattern(self.ast))
            else {
                continue;
            };
            find_binding_ids(self.ast, pattern, &mut identifiers);
        }
        for identifier in identifiers {
            let name = self.name(identifier.name(self.ast));
            self.scopes[self.current]
                .declared_symbols
                .insert(name, DeclKind::Param);
        }
        let old_kind = self.decl_kind;
        self.decl_kind = DeclKind::Param;
        parameters.visit_with(self);
        self.decl_kind = old_kind;

        let body = function.body(self.ast);
        self.mark_current(body.node_id());
        let old_strict_mode = self.strict_mode;
        if !self.strict_mode {
            self.strict_mode = self.has_use_strict(body.directives(self.ast));
        }
        body.body(self.ast).visit_with(self);
        self.strict_mode = old_strict_mode;
    }

    fn visit_stmt_within_child_scope(&mut self, statement: Stmt) {
        self.with_child(ScopeKind::Block, Some(statement.node_id()), |child| {
            if let StmtData::BlockStatement(block) = child.ast.stmt_data(statement) {
                child.mark_current(block.node_id());
                block.body(child.ast).visit_with(child);
            } else {
                statement.visit_with(child);
            }
        });
    }
}

impl<'ast> Visit<'ast> for Resolver<'ast> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }

    fn visit_program(&mut self, program: Program) {
        self.mark_current(program.node_id());
        self.is_module = self.ast.is_module();
        self.strict_mode = self.is_module || self.has_use_strict(program.directives(self.ast));
        program.body(self.ast).visit_with(self);
    }

    fn visit_stmts(&mut self, statements: TypedSubRange<Stmt>) {
        {
            let kind = self.decl_kind;
            let mut hoister = Hoister {
                resolver: self,
                kind,
                in_block: false,
                in_catch_body: false,
                excluded_from_catch: FxHashSet::default(),
                catch_param_decls: FxHashSet::default(),
            };
            statements.visit_with(&mut hoister);
        }
        statements.visit_children_with(self);
    }

    fn visit_identifier_reference(&mut self, reference: IdentifierReference) {
        if self.node_scopes[reference.node_id().index()].is_some() {
            return;
        }
        let name = self.name(reference.name(self.ast));
        let scope = self.mark_for_ref(name).unwrap_or(self.unresolved_scope_id);
        self.record_scope(reference.node_id(), scope);
    }

    fn visit_binding_identifier(&mut self, identifier: BindingIdentifier) {
        self.modify(identifier, self.decl_kind);
    }

    fn visit_label_identifier(&mut self, label: LabelIdentifier) {
        self.record_scope(label.node_id(), self.label_scope_id);
    }

    fn visit_jsx_element_name(&mut self, name: JsxElementName) {
        match self.ast.jsx_element_name_data(name) {
            JsxElementNameData::JsxIdentifier(identifier) => {
                let name = self.name(identifier.name(self.ast));
                let scope = if name.first().is_some_and(u8::is_ascii_lowercase) {
                    self.unresolved_scope_id
                } else {
                    self.mark_for_ref(name).unwrap_or(self.unresolved_scope_id)
                };
                self.record_scope(identifier.node_id(), scope);
            }
            JsxElementNameData::JsxMemberExpression(member) => {
                member.object(self.ast).visit_with(self);
            }
            JsxElementNameData::JsxNamespacedName(_) => {}
        }
    }

    fn visit_jsx_member_expression_object(&mut self, object: JsxMemberExpressionObject) {
        match self.ast.jsx_member_expression_object_data(object) {
            JsxMemberExpressionObjectData::JsxIdentifier(identifier) => {
                let name = self.name(identifier.name(self.ast));
                let scope = self.mark_for_ref(name).unwrap_or(self.unresolved_scope_id);
                self.record_scope(identifier.node_id(), scope);
            }
            JsxMemberExpressionObjectData::JsxMemberExpression(member) => {
                member.object(self.ast).visit_with(self);
            }
        }
    }

    fn visit_block_statement(&mut self, block: BlockStatement) {
        self.with_child(ScopeKind::Block, Some(block.node_id()), |child| {
            block.body(child.ast).visit_with(child);
        });
    }

    fn visit_catch_clause(&mut self, catch: CatchClause) {
        let body = catch.body(self.ast);
        self.with_child(ScopeKind::Fn, Some(catch.node_id()), |child| {
            child.mark_current(body.node_id());
            catch.param(child.ast).visit_with(child);
            body.body(child.ast).visit_with(child);
        });
    }

    fn visit_function(&mut self, function: Function) {
        match function.function_type(self.ast) {
            FunctionType::FunctionDeclaration => {
                let body = function.body(self.ast);
                self.with_child(ScopeKind::Fn, Some(body.node_id()), |child| {
                    child.visit_function_contents(function);
                });
            }
            FunctionType::FunctionExpression => {
                if let Some(identifier) = function.id(self.ast) {
                    self.with_child(ScopeKind::Fn, Some(function.node_id()), |name_scope| {
                        name_scope.modify(identifier, DeclKind::Function);
                        let body = function.body(name_scope.ast);
                        name_scope.with_child(ScopeKind::Fn, Some(body.node_id()), |child| {
                            child.visit_function_contents(function);
                        });
                    });
                } else {
                    let body = function.body(self.ast);
                    self.with_child(ScopeKind::Fn, Some(body.node_id()), |child| {
                        child.visit_function_contents(function);
                    });
                }
            }
        }
    }

    fn visit_arrow_function_expression(&mut self, arrow: ArrowFunctionExpression) {
        self.with_child(ScopeKind::Fn, Some(arrow.node_id()), |child| {
            let parameters = arrow.params(child.ast);
            let mut identifiers = std::vec::Vec::new();
            for id in parameters.items(child.ast).iter() {
                let item = child.ast.get_node_in_sub_range(id);
                let FormalParameterItemData::FormalParameter(parameter) =
                    child.ast.formal_parameter_item_data(item)
                else {
                    continue;
                };
                let FormalParameterPatternData::BindingPattern(pattern) = child
                    .ast
                    .formal_parameter_pattern_data(parameter.pattern(child.ast))
                else {
                    continue;
                };
                find_binding_ids(child.ast, pattern, &mut identifiers);
            }
            for identifier in identifiers {
                let name = child.name(identifier.name(child.ast));
                child.scopes[child.current]
                    .declared_symbols
                    .insert(name, DeclKind::Param);
            }
            let old_kind = child.decl_kind;
            child.decl_kind = DeclKind::Param;
            parameters.visit_with(child);
            child.decl_kind = old_kind;

            match child.ast.arrow_function_body_data(arrow.body(child.ast)) {
                ArrowFunctionBodyData::FunctionBody(body) => {
                    child.mark_current(body.node_id());
                    let old_strict_mode = child.strict_mode;
                    if !child.strict_mode {
                        child.strict_mode = child.has_use_strict(body.directives(child.ast));
                    }
                    body.body(child.ast).visit_with(child);
                    child.strict_mode = old_strict_mode;
                }
                ArrowFunctionBodyData::Expr(expression) => expression.visit_with(child),
            }
        });
    }

    fn visit_class(&mut self, class: Class) {
        match class.class_type(self.ast) {
            ClassType::ClassDeclaration => {
                if let Some(identifier) = class.id(self.ast) {
                    self.modify(identifier, DeclKind::Lexical);
                }
                self.with_child(ScopeKind::Fn, Some(class.node_id()), |child| {
                    let old_strict_mode = child.strict_mode;
                    child.strict_mode = true;
                    class.decorators(child.ast).visit_with(child);
                    class.super_class(child.ast).visit_with(child);
                    class.body(child.ast).visit_with(child);
                    child.strict_mode = old_strict_mode;
                });
            }
            ClassType::ClassExpression => {
                class.super_class(self.ast).visit_with(self);
                self.with_child(ScopeKind::Fn, Some(class.node_id()), |child| {
                    if let Some(identifier) = class.id(child.ast) {
                        child.modify(identifier, DeclKind::Lexical);
                    }
                    let old_strict_mode = child.strict_mode;
                    child.strict_mode = true;
                    class.decorators(child.ast).visit_with(child);
                    class.super_class(child.ast).visit_with(child);
                    class.body(child.ast).visit_with(child);
                    child.strict_mode = old_strict_mode;
                });
            }
        }
    }

    fn visit_method_definition(&mut self, method: MethodDefinition) {
        method.decorators(self.ast).visit_with(self);
        method.key(self.ast).visit_with(self);
        let function = method.value(self.ast);
        let body = function.body(self.ast);
        self.with_child(ScopeKind::Fn, Some(body.node_id()), |child| {
            child.visit_function_contents(function);
        });
    }

    fn visit_property_definition(&mut self, property: PropertyDefinition) {
        property.decorators(self.ast).visit_with(self);
        if property.computed(self.ast) {
            property.key(self.ast).visit_with(self);
        }
        property.value(self.ast).visit_with(self);
    }

    fn visit_static_block(&mut self, block: StaticBlock) {
        self.with_child(ScopeKind::Block, Some(block.node_id()), |child| {
            block.body(child.ast).visit_with(child);
        });
    }

    fn visit_object_expression(&mut self, object: ObjectExpression) {
        self.with_child(ScopeKind::Block, Some(object.node_id()), |child| {
            object.visit_children_with(child);
        });
    }

    fn visit_object_property(&mut self, property: ObjectProperty) {
        property.key(self.ast).visit_with(self);
        let value = property.value(self.ast);
        let ExprData::Function(function) = self.ast.expr_data(value) else {
            value.visit_with(self);
            return;
        };

        let body = function.body(self.ast);
        match property.kind(self.ast) {
            PropertyKind::Get => {
                self.with_child(ScopeKind::Fn, Some(body.node_id()), |child| {
                    child.visit_function_contents(function);
                });
            }
            PropertyKind::Set => {
                self.with_child(ScopeKind::Fn, Some(function.node_id()), |child| {
                    let parameters = function.params(child.ast);
                    let mut identifiers = std::vec::Vec::new();
                    for id in parameters.items(child.ast).iter() {
                        let item = child.ast.get_node_in_sub_range(id);
                        let FormalParameterItemData::FormalParameter(parameter) =
                            child.ast.formal_parameter_item_data(item)
                        else {
                            continue;
                        };
                        let FormalParameterPatternData::BindingPattern(pattern) = child
                            .ast
                            .formal_parameter_pattern_data(parameter.pattern(child.ast))
                        else {
                            continue;
                        };
                        find_binding_ids(child.ast, pattern, &mut identifiers);
                    }
                    for identifier in identifiers {
                        let name = child.name(identifier.name(child.ast));
                        child.scopes[child.current]
                            .declared_symbols
                            .insert(name, DeclKind::Param);
                    }
                    let old_kind = child.decl_kind;
                    child.decl_kind = DeclKind::Param;
                    parameters.visit_with(child);
                    child.decl_kind = old_kind;
                    child.with_child(ScopeKind::Block, Some(body.node_id()), |body_child| {
                        let old_strict_mode = body_child.strict_mode;
                        if !body_child.strict_mode {
                            body_child.strict_mode =
                                body_child.has_use_strict(body.directives(body_child.ast));
                        }
                        body.body(body_child.ast).visit_with(body_child);
                        body_child.strict_mode = old_strict_mode;
                    });
                });
            }
            PropertyKind::Init if property.method(self.ast) => {
                self.with_child(ScopeKind::Fn, Some(body.node_id()), |child| {
                    child.visit_function_contents(function);
                });
            }
            PropertyKind::Init => value.visit_with(self),
        }
    }

    fn visit_for_statement(&mut self, statement: ForStatement) {
        self.with_child(ScopeKind::Block, Some(statement.node_id()), |child| {
            statement.init(child.ast).visit_with(child);
            statement.test(child.ast).visit_with(child);
            statement.update(child.ast).visit_with(child);
            child.visit_stmt_within_child_scope(statement.body(child.ast));
        });
    }

    fn visit_for_in_statement(&mut self, statement: ForInStatement) {
        self.with_child(ScopeKind::Block, Some(statement.node_id()), |child| {
            statement.left(child.ast).visit_with(child);
            statement.right(child.ast).visit_with(child);
            child.visit_stmt_within_child_scope(statement.body(child.ast));
        });
    }

    fn visit_for_of_statement(&mut self, statement: ForOfStatement) {
        self.with_child(ScopeKind::Block, Some(statement.node_id()), |child| {
            statement.left(child.ast).visit_with(child);
            statement.right(child.ast).visit_with(child);
            child.visit_stmt_within_child_scope(statement.body(child.ast));
        });
    }

    fn visit_do_while_statement(&mut self, statement: DoWhileStatement) {
        // Preserve the legacy AST traversal order. Scope allocation is
        // observable through the resolver's scope ids even though the body is
        // evaluated before the test at runtime.
        statement.test(self.ast).visit_with(self);
        statement.body(self.ast).visit_with(self);
    }

    fn visit_switch_statement(&mut self, statement: SwitchStatement) {
        statement.discriminant(self.ast).visit_with(self);
        self.with_child(ScopeKind::Block, Some(statement.node_id()), |child| {
            statement.cases(child.ast).visit_with(child);
        });
    }

    fn visit_switch_case(&mut self, case: SwitchCase) {
        case.consequent(self.ast).visit_with(self);
        case.test(self.ast).visit_with(self);
    }

    fn visit_variable_declaration(&mut self, declaration: VariableDeclaration) {
        let old_kind = self.decl_kind;
        self.decl_kind = declaration.kind(self.ast).into();
        declaration.declarators(self.ast).visit_with(self);
        self.decl_kind = old_kind;
    }

    fn visit_import_declaration(&mut self, declaration: ImportDeclaration) {
        declaration.specifiers(self.ast).visit_with(self);
    }

    fn visit_import_specifier(&mut self, specifier: ImportSpecifier) {
        self.modify(specifier.local(self.ast), DeclKind::Lexical);
    }

    fn visit_import_default_specifier(&mut self, specifier: ImportDefaultSpecifier) {
        self.modify(specifier.local(self.ast), DeclKind::Lexical);
    }

    fn visit_import_namespace_specifier(&mut self, specifier: ImportNamespaceSpecifier) {
        self.modify(specifier.local(self.ast), DeclKind::Lexical);
    }

    fn visit_export_named_declaration(&mut self, declaration: ExportNamedDeclaration) {
        if declaration.source(self.ast).is_some() {
            return;
        }
        declaration.declaration(self.ast).visit_with(self);
        declaration.specifiers(self.ast).visit_with(self);
    }

    fn visit_export_default_declaration(&mut self, declaration: ExportDefaultDeclaration) {
        match self
            .ast
            .export_default_declaration_kind_data(declaration.declaration(self.ast))
        {
            ExportDefaultDeclarationKindData::Expr(expression) => expression.visit_with(self),
            ExportDefaultDeclarationKindData::Function(function) => {
                if function.id(self.ast).is_some() {
                    let body = function.body(self.ast);
                    self.with_child(ScopeKind::Fn, Some(body.node_id()), |child| {
                        child.visit_function_contents(function);
                    });
                } else {
                    function.visit_with(self);
                }
            }
            ExportDefaultDeclarationKindData::Class(class) => {
                let old_strict_mode = self.strict_mode;
                self.strict_mode = true;
                class.decorators(self.ast).visit_with(self);
                class.super_class(self.ast).visit_with(self);
                class.body(self.ast).visit_with(self);
                self.strict_mode = old_strict_mode;
            }
            _ => {}
        }
    }

    fn visit_export_specifier(&mut self, specifier: ExportSpecifier) {
        let local = specifier.local(self.ast);
        if let ModuleExportNameData::IdentifierName(identifier) =
            self.ast.module_export_name_data(local)
        {
            let name = self.name(identifier.name(self.ast));
            let scope = self.mark_for_ref(name).unwrap_or(self.unresolved_scope_id);
            self.record_scope(identifier.node_id(), scope);
        }
        let exported = specifier.exported(self.ast);
        if exported.span(self.ast) != local.span(self.ast)
            && let ModuleExportNameData::IdentifierName(identifier) =
                self.ast.module_export_name_data(exported)
        {
            let name = self.name(identifier.name(self.ast));
            let scope = self.mark_for_ref(name).unwrap_or(self.unresolved_scope_id);
            self.record_scope(identifier.node_id(), scope);
        }
    }

    fn visit_break_statement(&mut self, statement: BreakStatement) {
        statement.label(self.ast).visit_with(self);
    }

    fn visit_continue_statement(&mut self, statement: ContinueStatement) {
        statement.label(self.ast).visit_with(self);
    }

    fn visit_labeled_statement(&mut self, statement: LabeledStatement) {
        statement.label(self.ast).visit_with(self);
        statement.body(self.ast).visit_with(self);
    }

    // Type annotations were deliberately disabled in the legacy resolver.
    fn visit_ts_type(&mut self, _: TsType) {}
}

struct Hoister<'resolver, 'ast> {
    resolver: &'resolver mut Resolver<'ast>,
    kind: DeclKind,
    in_block: bool,
    in_catch_body: bool,
    excluded_from_catch: FxHashSet<&'ast [u8]>,
    catch_param_decls: FxHashSet<&'ast [u8]>,
}

impl<'resolver, 'ast> Hoister<'resolver, 'ast> {
    fn add_binding(&mut self, identifier: BindingIdentifier) {
        let name = self.resolver.name(identifier.name(self.resolver.ast));
        if self.in_catch_body {
            if self.resolver.mark_for_ref_inner(name, true).is_some()
                && self.catch_param_decls.contains(name)
            {
                return;
            }
            self.excluded_from_catch.insert(name);
        } else if self.catch_param_decls.contains(name) && !self.excluded_from_catch.contains(name)
        {
            return;
        }
        self.resolver.modify(identifier, self.kind);
    }
}

impl<'ast> Visit<'ast> for Hoister<'_, 'ast> {
    fn ast(&self) -> &Ast<'ast> {
        self.resolver.ast
    }

    fn visit_binding_pattern(&mut self, pattern: BindingPattern) {
        if let BindingPatternData::BindingIdentifier(identifier) =
            self.resolver.ast.binding_pattern_data(pattern)
        {
            self.add_binding(identifier);
        } else {
            pattern.visit_children_with(self);
        }
    }

    fn visit_stmts(&mut self, statements: TypedSubRange<Stmt>) {
        for id in statements.iter() {
            let statement = self.resolver.ast.get_node_in_sub_range(id);
            if let StmtData::Declaration(declaration) = self.resolver.ast.stmt_data(statement)
                && matches!(
                    self.resolver.ast.decl_data(declaration),
                    DeclData::VariableDeclaration(_) | DeclData::Function(_)
                )
            {
                statement.visit_with(self);
            }
        }
        for id in statements.iter() {
            let statement = self.resolver.ast.get_node_in_sub_range(id);
            if !matches!(
                self.resolver.ast.stmt_data(statement),
                StmtData::Declaration(declaration)
                    if matches!(
                        self.resolver.ast.decl_data(declaration),
                        DeclData::VariableDeclaration(_) | DeclData::Function(_)
                    )
            ) {
                statement.visit_with(self);
            }
        }
    }

    fn visit_expr(&mut self, _: Expr) {}

    fn visit_arrow_function_expression(&mut self, _: ArrowFunctionExpression) {}

    fn visit_function(&mut self, function: Function) {
        if function.function_type(self.resolver.ast) != FunctionType::FunctionDeclaration {
            return;
        }
        let Some(identifier) = function.id(self.resolver.ast) else {
            return;
        };
        let name = self.resolver.name(identifier.name(self.resolver.ast));
        if self.catch_param_decls.contains(name) {
            return;
        }
        if self.in_block {
            if self.resolver.strict_mode {
                return;
            }
            if matches!(
                self.resolver.is_declared(name, self.resolver.current),
                Some(DeclKind::Lexical | DeclKind::Param)
            ) {
                return;
            }
        }
        self.resolver.modify(identifier, DeclKind::Function);
    }

    fn visit_class(&mut self, class: Class) {
        if class.class_type(self.resolver.ast) != ClassType::ClassDeclaration || self.in_block {
            return;
        }
        if let Some(identifier) = class.id(self.resolver.ast) {
            self.resolver.modify(identifier, DeclKind::Lexical);
        }
    }

    fn visit_export_default_declaration(&mut self, declaration: ExportDefaultDeclaration) {
        match self
            .resolver
            .ast
            .export_default_declaration_kind_data(declaration.declaration(self.resolver.ast))
        {
            ExportDefaultDeclarationKindData::Function(function) => {
                if let Some(identifier) = function.id(self.resolver.ast) {
                    self.resolver.modify(identifier, DeclKind::Var);
                }
                function.visit_with(self);
            }
            ExportDefaultDeclarationKindData::Class(class) => {
                if let Some(identifier) = class.id(self.resolver.ast) {
                    self.resolver.modify(identifier, DeclKind::Lexical);
                }
                class.visit_with(self);
            }
            _ => {}
        }
    }

    fn visit_block_statement(&mut self, block: BlockStatement) {
        let old_in_block = self.in_block;
        self.in_block = true;
        block.body(self.resolver.ast).visit_with(self);
        self.in_block = old_in_block;
    }

    fn visit_catch_clause(&mut self, catch: CatchClause) {
        let old_excluded = self.excluded_from_catch.clone();
        self.excluded_from_catch.clear();
        let old_in_catch_body = self.in_catch_body;

        let mut params = std::vec::Vec::new();
        if let Some(param) = catch.param(self.resolver.ast) {
            find_binding_ids(self.resolver.ast, param, &mut params);
        }
        let original_params = self.catch_param_decls.clone();
        self.catch_param_decls.extend(
            params
                .into_iter()
                .map(|id| self.resolver.name(id.name(self.resolver.ast))),
        );

        self.in_catch_body = true;
        catch.body(self.resolver.ast).visit_with(self);
        self.in_catch_body = false;

        catch.param(self.resolver.ast).visit_with(self);

        self.catch_param_decls = original_params;
        self.in_catch_body = old_in_catch_body;
        self.excluded_from_catch = old_excluded;
    }

    fn visit_variable_declaration(&mut self, declaration: VariableDeclaration) {
        if self.in_block && declaration.kind(self.resolver.ast) != VariableKind::Var {
            return;
        }
        let old_kind = self.kind;
        self.kind = declaration.kind(self.resolver.ast).into();
        for id in declaration.declarators(self.resolver.ast).iter() {
            let declarator = self.resolver.ast.get_node_in_sub_range(id);
            declarator.id(self.resolver.ast).visit_with(self);
        }
        self.kind = old_kind;
    }

    fn visit_import_specifier(&mut self, specifier: ImportSpecifier) {
        self.resolver
            .modify(specifier.local(self.resolver.ast), DeclKind::Lexical);
    }

    fn visit_import_default_specifier(&mut self, specifier: ImportDefaultSpecifier) {
        self.resolver
            .modify(specifier.local(self.resolver.ast), DeclKind::Lexical);
    }

    fn visit_import_namespace_specifier(&mut self, specifier: ImportNamespaceSpecifier) {
        self.resolver
            .modify(specifier.local(self.resolver.ast), DeclKind::Lexical);
    }

    fn visit_switch_statement(&mut self, statement: SwitchStatement) {
        let old_in_block = self.in_block;
        self.in_block = true;
        statement.cases(self.resolver.ast).visit_with(self);
        self.in_block = old_in_block;
    }

    fn visit_for_statement_init(&mut self, init: ForStatementInit) {
        if let ForStatementInitData::VariableDeclaration(declaration) =
            self.resolver.ast.for_statement_init_data(init)
            && declaration.kind(self.resolver.ast) == VariableKind::Var
        {
            declaration.visit_with(self);
        }
    }

    fn visit_for_statement_left(&mut self, left: ForStatementLeft) {
        if let ForStatementLeftData::VariableDeclaration(declaration) =
            self.resolver.ast.for_statement_left_data(left)
            && declaration.kind(self.resolver.ast) == VariableKind::Var
        {
            declaration.visit_with(self);
        }
    }

    fn visit_static_block(&mut self, _: StaticBlock) {}

    fn visit_assignment_target(&mut self, _: AssignmentTarget) {}
}
