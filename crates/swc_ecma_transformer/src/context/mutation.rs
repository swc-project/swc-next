//! Atomic edits of existing AST edges and their semantic indexes.
//!
//! New bindings, references and scope owners are registered by construction APIs.
//! These methods attach prepared nodes or relocate existing identities. Replaced
//! shells remain in the AST arena until the entire AST is dropped.
use super::Mutation;
use swc_next_ecma_ast::{
    Ast, BlockStatement, ClassBody, ClassElement, Expr, FunctionBody, NodeId, NodeKindData,
    Program, PropertyDefinition, PropertyKey, ScopeId, StaticBlock, Stmt, SubRange, SwitchCase,
    TsModuleBlock,
};

/// The operation to apply to a class element during in-place list editing.
pub(crate) enum ClassElementAction {
    Update(ClassElement),
    Unchanged,
    Delete,
}

/// A statement-list owner; arbitrary AST nodes cannot be used as list targets.
#[derive(Clone, Copy)]
pub enum StatementList {
    Program(Program),
    FunctionBody(FunctionBody),
    Block(BlockStatement),
    StaticBlock(StaticBlock),
    SwitchCase(SwitchCase),
    TsModuleBlock(TsModuleBlock),
}

impl StatementList {
    pub fn node_id(self) -> NodeId {
        match self {
            Self::Program(node) => node.node_id(),
            Self::FunctionBody(node) => node.node_id(),
            Self::Block(node) => node.node_id(),
            Self::StaticBlock(node) => node.node_id(),
            Self::SwitchCase(node) => node.node_id(),
            Self::TsModuleBlock(node) => node.node_id(),
        }
    }

    pub fn statements(self, ast: &Ast<'_>) -> SubRange<Stmt> {
        match self {
            Self::Program(node) => node.body(ast),
            Self::FunctionBody(node) => node.body(ast),
            Self::Block(node) => node.body(ast),
            Self::StaticBlock(node) => node.body(ast),
            Self::SwitchCase(node) => node.consequent(ast),
            Self::TsModuleBlock(node) => node.body(ast),
        }
    }

    pub(super) fn set(self, ast: &mut Ast<'_>, statements: SubRange<Stmt>) {
        match self {
            Self::Program(node) => node.set_body(ast, statements),
            Self::FunctionBody(node) => node.set_body(ast, statements),
            Self::Block(node) => node.set_body(ast, statements),
            Self::StaticBlock(node) => node.set_body(ast, statements),
            Self::SwitchCase(node) => node.set_consequent(ast, statements),
            Self::TsModuleBlock(node) => node.set_body(ast, statements),
        }
    }
}

impl Mutation<'_, '_, '_> {
    pub fn statement_list(&self, node: NodeId) -> StatementList {
        match self.ast.kind_data(node) {
            NodeKindData::Program(node) => StatementList::Program(node),
            NodeKindData::FunctionBody(node) => StatementList::FunctionBody(node),
            NodeKindData::BlockStatement(node) => StatementList::Block(node),
            NodeKindData::StaticBlock(node) => StatementList::StaticBlock(node),
            NodeKindData::SwitchCase(node) => StatementList::SwitchCase(node),
            NodeKindData::TsModuleBlock(node) => StatementList::TsModuleBlock(node),
            _ => panic!("node must own a statement list"),
        }
    }

    /// Attach a prepared list. The caller has already removed or moved old elements.
    pub(super) fn attach_statements(&mut self, list: StatementList, statements: &[Stmt]) {
        let statements = self.ast.add_sub_range(statements.iter().copied());
        list.set(self.ast, statements);
    }

    pub(super) fn statement_scope(&self, list: StatementList) -> ScopeId {
        let scope = self
            .state
            .semantic
            .scope_at(list.node_id())
            .expect("statement container is registered");
        assert!(
            self.state.semantic.scope(scope).active,
            "container scope must be active"
        );
        scope
    }

    pub(super) fn assert_statement_scope(&self, list: StatementList, statement: Stmt) {
        assert_eq!(
            self.state.semantic.scope_outside(statement.node_id()),
            Some(self.statement_scope(list)),
            "statement must be registered in its destination scope"
        );
    }

    pub fn insert_statement(&mut self, list: StatementList, index: usize, statement: Stmt) {
        self.assert_statement_scope(list, statement);
        let mut statements: Vec<_> = list
            .statements(self.ast)
            .iter()
            .map(|id| self.ast.get_node_in_sub_range(id))
            .collect();
        statements.insert(index, statement);
        self.attach_statements(list, &statements);
    }

    /// Fill a fresh arrow's empty body with content registered in its scope.
    pub(crate) fn set_arrow_expression_body(
        &mut self,
        arrow: swc_next_ecma_ast::ArrowFunctionExpression,
        expression: Expr,
    ) {
        let scope = self
            .scope_for_node(arrow.node_id())
            .expect("arrow scope exists");
        let body = arrow
            .body(self.ast)
            .as_function_body(self.ast)
            .expect("fresh arrow has an empty function body");
        assert!(body.body(self.ast).is_empty() && body.directives(self.ast).is_empty());
        assert_eq!(
            self.state.semantic.scope_outside(expression.node_id()),
            Some(scope)
        );
        arrow.set_body(
            self.ast,
            swc_next_ecma_ast::ArrowFunctionBody::Expr(expression),
        );
        arrow.set_expression(self.ast, true);
    }

    pub fn insert_class_element(&mut self, body: ClassBody, index: usize, element: ClassElement) {
        let mut elements: Vec<_> = body
            .body(self.ast)
            .iter()
            .map(|id| self.ast.get_node_in_sub_range(id))
            .collect();
        elements.insert(index, element);
        self.attach_class_elements(body, &elements);
        self.invalidate_static_block_layout(body.node_id());
    }

    pub(crate) fn set_property_value(&mut self, property: PropertyDefinition, value: Expr) {
        assert!(
            property.value(self.ast).is_none(),
            "initializer must be absent"
        );
        property.set_value(self.ast, Some(value));
    }

    /// The wrapper contains the original initializer, in the same environment.
    pub(crate) fn wrap_property_value(&mut self, property: PropertyDefinition, wrapper: Expr) {
        let old = property
            .value(self.ast)
            .expect("property must have an initializer");
        self.assert_expression_wrapper(old.node_id(), wrapper);
        property.set_value(self.ast, Some(wrapper));
    }

    /// Preserve the old computed key within a new wrapper.
    pub(crate) fn wrap_property_key(&mut self, parent: NodeId, key: PropertyKey) {
        let old = match self.ast.kind_data(parent) {
            NodeKindData::ObjectProperty(property) => property.key(self.ast),
            NodeKindData::PropertyDefinition(property) => property.key(self.ast),
            _ => panic!("property key owner must be a property"),
        };
        let wrapper = key
            .as_expr(self.ast)
            .expect("computed key wrapper must be an expression");
        self.assert_expression_wrapper(old.node_id(), wrapper);
        self.replace_property_key_edge(parent, key);
    }

    fn replace_property_key_edge(&mut self, parent: NodeId, key: PropertyKey) {
        match self.ast.kind_data(parent) {
            NodeKindData::ObjectProperty(property) => {
                property.set_key(self.ast, key);
            }
            NodeKindData::PropertyDefinition(property) => {
                property.set_key(self.ast, key);
            }
            _ => panic!("property key owner must be a property"),
        }
    }

    /// Rewrite and compact existing slots without allocating a new list.
    /// The callback prepares replacements; Mutation validates each edit and updates
    /// name resolution and transform metadata together.
    pub(crate) fn filter_map_class_elements(
        &mut self,
        body: ClassBody,
        mut transform: impl FnMut(&mut Self, ClassElement) -> ClassElementAction,
    ) {
        let mut elements = body.body(self.ast);
        let mut write_index = 0;
        for (read_index, slot) in elements.iter().enumerate() {
            let element = self.ast.get_node_in_sub_range(slot);
            let replacement = match transform(self, element) {
                ClassElementAction::Unchanged => {
                    if write_index == read_index {
                        write_index += 1;
                        continue;
                    }
                    element
                }
                ClassElementAction::Update(replacement) => {
                    self.replace_class_element(element, Some(replacement));
                    replacement
                }
                ClassElementAction::Delete => {
                    self.replace_class_element(element, None);
                    continue;
                }
            };
            elements.replace_slot(
                self.ast,
                elements.get(write_index).expect("write index is in range"),
                replacement,
            );
            write_index += 1;
        }
        if write_index != elements.len() {
            elements.split_off(write_index);
            body.set_body(self.ast, elements);
        }
    }

    fn invalidate_static_block_layout(&mut self, node: NodeId) {
        let mut scope = self.state.semantic.scope_at(node);
        while let Some(id) = scope {
            let current = self.state.semantic.scope(id);
            if current.kind == swc_next_ecma_semantic::scope::ScopeKind::Class {
                self.state
                    .analysis
                    .invalidate_static_block_layout(current.node);
                break;
            }
            scope = current.parent;
        }
    }

    fn replace_class_element(&mut self, old: ClassElement, replacement: Option<ClassElement>) {
        if replacement.map(|node| node.node_id()) == Some(old.node_id()) {
            return;
        }
        self.validate_class_element(old, replacement);
        if matches!(
            self.ast.kind_data(old.node_id()),
            NodeKindData::StaticBlock(_)
        ) {
            self.state.analysis.replace_static_block(old.node_id());
        }
        self.invalidate_static_block_layout(old.node_id());
    }

    fn validate_class_element(&mut self, old: ClassElement, replacement: Option<ClassElement>) {
        let ast = &*self.ast;
        match ast.kind_data(old.node_id()) {
            NodeKindData::StaticBlock(block) => {
                if let Some(scope) = self.scope_for_node(block.node_id()) {
                    debug_assert!(
                        block.body(ast).is_empty(),
                        "static block contents must be moved first"
                    );
                    self.state.semantic.remove_scope(scope);
                }
            }
            NodeKindData::PropertyDefinition(property) => {
                let Some(new) = replacement.and_then(|node| node.as_property_definition(ast))
                else {
                    debug_assert!(false, "property replacement must preserve its children");
                    return;
                };
                debug_assert_eq!(
                    self.state.semantic.scope_at(property.node_id()),
                    self.state.semantic.scope_at(new.node_id())
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
            }
            NodeKindData::MethodDefinition(method) => {
                let Some(new) = replacement.and_then(|node| node.as_method_definition(ast)) else {
                    debug_assert!(false, "method replacement must preserve its children");
                    return;
                };
                debug_assert_eq!(
                    self.state.semantic.scope_at(method.node_id()),
                    self.state.semantic.scope_at(new.node_id())
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
    }

    fn attach_class_elements(&mut self, body: ClassBody, elements: &[ClassElement]) {
        let mut slots = body.body(self.ast);
        if elements.len() <= slots.len() {
            for (slot, element) in slots.iter().zip(elements.iter().copied()) {
                slots.replace_slot(self.ast, slot, element);
            }
            slots.split_off(elements.len());
        } else {
            slots = self.ast.add_sub_range(elements.iter().copied());
        }
        body.set_body(self.ast, slots);
    }
}

#[cfg(test)]
#[path = "../../tests/context/mutation.rs"]
mod tests;
