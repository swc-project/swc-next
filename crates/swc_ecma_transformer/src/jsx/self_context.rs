use rustc_hash::FxHashSet;

use swc_next_ecma_ast::{
    Ast, MethodDefinitionKind, NodeId, NodeKindData, Program, Visit, VisitWith,
};

pub(super) struct JsxSelfContext {
    disallowed: FxHashSet<NodeId>,
}

impl JsxSelfContext {
    pub(super) fn collect(program: Program, ast: &Ast<'_>) -> Self {
        let mut collector = JsxSelfCollector {
            ast,
            path: Vec::with_capacity(64),
            disallowed: FxHashSet::default(),
        };
        program.visit_with(&mut collector);
        debug_assert!(collector.path.is_empty());
        Self {
            disallowed: collector.disallowed,
        }
    }

    pub(super) fn can_add_self(&self, node: NodeId) -> bool {
        !self.disallowed.contains(&node)
    }
}

struct JsxSelfCollector<'visit, 'ast> {
    ast: &'visit Ast<'ast>,
    path: Vec<NodeId>,
    disallowed: FxHashSet<NodeId>,
}

impl JsxSelfCollector<'_, '_> {
    fn can_add_self(&self) -> bool {
        for index in (0..self.path.len()).rev() {
            let ancestor = self.path[index];
            match self.ast.kind_data(ancestor) {
                NodeKindData::ArrowFunctionExpression(_) => {}
                NodeKindData::Function(_) => {
                    let Some(parent_index) = index.checked_sub(1) else {
                        return true;
                    };
                    let parent = self.path[parent_index];
                    let NodeKindData::MethodDefinition(method) = self.ast.kind_data(parent) else {
                        return true;
                    };
                    if method.kind(self.ast) != MethodDefinitionKind::Constructor {
                        return true;
                    }
                    return self.path[..parent_index]
                        .iter()
                        .rev()
                        .find_map(|ancestor| match self.ast.kind_data(*ancestor) {
                            NodeKindData::Class(class) => {
                                Some(class.super_class(self.ast).is_none())
                            }
                            _ => None,
                        })
                        .unwrap_or(true);
                }
                // Class heritage and computed keys retain the enclosing `this`.
                // Only field initializers and static blocks establish their own
                // environment; method bodies are handled by `Function` above.
                NodeKindData::PropertyDefinition(property)
                    if property.value(self.ast).is_some_and(|value| {
                        self.path.get(index + 1) == Some(&value.node_id())
                    }) =>
                {
                    return true;
                }
                NodeKindData::StaticBlock(_) | NodeKindData::Program(_) => return true,
                _ => {}
            }
        }
        true
    }
}

impl<'ast> Visit<'ast> for JsxSelfCollector<'_, 'ast> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }

    fn enter_node(&mut self, node: NodeKindData) {
        self.path.push(node.node_id());
    }

    fn leave_node(&mut self, node: NodeKindData) {
        let current = self.path.pop();
        debug_assert_eq!(current, Some(node.node_id()));
    }

    fn visit_jsx_opening_element(&mut self, opening: swc_next_ecma_ast::JsxOpeningElement) {
        if !self.can_add_self() {
            self.disallowed.insert(opening.node_id());
            if let Some(parent) = self.path.last().copied()
                && matches!(self.ast.kind_data(parent), NodeKindData::JsxElement(_))
            {
                self.disallowed.insert(parent);
            }
        }
        self.enter_node(NodeKindData::JsxOpeningElement(opening));
        opening.visit_children_with(self);
        self.leave_node(NodeKindData::JsxOpeningElement(opening));
    }
}
