//! Atomic edits of existing AST edges and their semantic indexes.
//!
//! New bindings, references and scope owners are registered by construction APIs.
//! These methods attach prepared nodes or relocate existing identities. Replaced
//! shells remain in the AST arena until the entire AST is dropped.
use super::Mutation;
use swc_next_ecma_ast::ObjectProperty;
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
        self.state
            .semantic
            .invalidate_static_block_layout(body.node_id());
    }

    pub fn set_property_computed(&mut self, owner: NodeId, computed: bool) {
        match self.ast.kind_data(owner) {
            NodeKindData::ObjectProperty(property) => property.set_computed(self.ast, computed),
            NodeKindData::PropertyDefinition(property) => property.set_computed(self.ast, computed),
            _ => panic!("computed key owner must be a property"),
        }
    }

    pub fn set_property_shorthand(&mut self, property: ObjectProperty, shorthand: bool) {
        property.set_shorthand(self.ast, shorthand);
    }

    pub fn set_property_method(&mut self, property: ObjectProperty, method: bool) {
        property.set_method(self.ast, method);
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

    pub(crate) fn set_property_key(&mut self, parent: NodeId, key: PropertyKey) {
        let old = match self.ast.kind_data(parent) {
            NodeKindData::ObjectProperty(property) => property.key(self.ast),
            NodeKindData::PropertyDefinition(property) => property.key(self.ast),
            _ => panic!("property key owner must be a property"),
        };
        assert!(
            matches!(
                self.ast.kind_data(old.node_id()),
                NodeKindData::IdentifierName(_) | NodeKindData::StringLiteral(_)
            ),
            "replacement must preserve evaluated key expressions"
        );
        self.replace_property_key_edge(parent, key);
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
    /// The callback prepares replacements; Semantic validates and updates each edit.
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
                    self.state
                        .semantic
                        .replace_class_element(self.ast, element, Some(replacement));
                    replacement
                }
                ClassElementAction::Delete => {
                    self.state
                        .semantic
                        .replace_class_element(self.ast, element, None);
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
