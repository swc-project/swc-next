use swc_next_ecma_ast::{Ast, MethodDefinitionKind, NodeId, NodeKindData};

pub(super) struct JsxSelfContext {
    environment: SelfEnvironment,
    environments: Vec<(NodeId, SelfEnvironment)>,
}

#[derive(Clone, Copy)]
struct SelfEnvironment {
    allowed: bool,
    derived_class: bool,
    derived_constructor: Option<NodeId>,
    field_initializer: Option<NodeId>,
}

impl JsxSelfContext {
    pub(super) fn new() -> Self {
        Self {
            environment: SelfEnvironment {
                allowed: true,
                derived_class: false,
                derived_constructor: None,
                field_initializer: None,
            },
            environments: Vec::new(),
        }
    }

    pub(super) fn can_add_self(&self) -> bool {
        self.environment.allowed
    }

    pub(super) fn is_empty(&self) -> bool {
        self.environments.is_empty()
    }

    pub(super) fn enter_expression(&mut self, node: NodeId) {
        if self.environment.field_initializer == Some(node) {
            self.environments.push((node, self.environment));
            self.environment.allowed = true;
        }
    }

    pub(super) fn enter_node(&mut self, node: NodeKindData, ast: &Ast<'_>) {
        let previous = self.environment;
        match node {
            NodeKindData::Class(class) => {
                self.environment.derived_class = class.super_class(ast).is_some();
            }
            NodeKindData::MethodDefinition(method) => {
                self.environment.derived_constructor = (self.environment.derived_class
                    && method.kind(ast) == MethodDefinitionKind::Constructor)
                    .then(|| method.value(ast).node_id());
            }
            NodeKindData::Function(function) => {
                self.environment.allowed =
                    self.environment.derived_constructor != Some(function.node_id());
            }
            // Heritage and computed keys retain the enclosing `this`.
            // Field initializers and static blocks establish their own environment.
            NodeKindData::PropertyDefinition(property) => {
                self.environment.field_initializer =
                    property.value(ast).map(|value| value.node_id());
            }
            NodeKindData::StaticBlock(_) => self.environment.allowed = true,
            _ => return,
        }
        // Expression and scope entries can share a node id. Each scope owns a
        // frame even when its environment is unchanged, so exits remain paired.
        self.environments.push((node.node_id(), previous));
    }

    pub(super) fn leave_node(&mut self, node: NodeId) {
        if let Some(&(owner, environment)) = self.environments.last()
            && owner == node
        {
            self.environments.pop();
            self.environment = environment;
        }
    }
}
