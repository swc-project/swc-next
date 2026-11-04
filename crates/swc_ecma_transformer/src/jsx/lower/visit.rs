use super::JsxLowerer;
use swc_next_ecma_ast::VisitMutWith;
use swc_next_ecma_ast::{
    Ast, Class, Expr, Function, Lang, MethodDefinition, NodeId, NodeKindData, Program,
    PropertyDefinition, StaticBlock, VisitMut,
};

impl<'ast> JsxLowerer<'_, 'ast> {
    fn enter_node(&mut self, node: NodeKindData, ast: &Ast<'ast>) {
        if self.options.development && self.options.source_self {
            self.self_context.enter_node(node, ast);
        }
    }

    fn enter_expression(&mut self, expression: Expr) {
        if self.options.development && self.options.source_self {
            self.self_context.enter_expression(expression.node_id());
        }
    }

    fn leave_node(&mut self, node: NodeId) {
        if self.options.development && self.options.source_self {
            self.self_context.leave_node(node);
        }
    }

    pub(in crate::jsx) fn finish_program(&mut self, program: Program, ast: &mut Ast<'ast>) {
        debug_assert!(self.self_context.is_empty());
        self.insert_program_bindings(program, ast);
        let source_type = ast.source_type();
        let lang = match ast.lang() {
            Lang::Jsx => Lang::Js,
            Lang::Tsx => Lang::Ts,
            lang => lang,
        };
        ast.set_source_metadata(source_type, lang);
    }
}

pub(in crate::jsx) struct JsxVisitor<'visit, 'options, 'ast> {
    ast: &'visit mut Ast<'ast>,
    lowerer: &'visit mut JsxLowerer<'options, 'ast>,
}

impl<'visit, 'options, 'ast> JsxVisitor<'visit, 'options, 'ast> {
    pub(in crate::jsx) fn new(
        ast: &'visit mut Ast<'ast>,
        lowerer: &'visit mut JsxLowerer<'options, 'ast>,
    ) -> Self {
        Self { ast, lowerer }
    }
}

impl<'ast> VisitMut<'ast> for JsxVisitor<'_, '_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }

    #[inline]
    fn visit_mut_expr(&mut self, expression: Expr) -> Option<Expr> {
        self.lowerer.enter_expression(expression);
        expression.visit_mut_children_with(self);
        let replacement = self.lowerer.lower_expression(expression, self.ast);
        self.lowerer.leave_node(expression.node_id());
        replacement
    }

    fn visit_mut_class(&mut self, node: Class) -> Option<Class> {
        self.lowerer.enter_node(NodeKindData::Class(node), self.ast);
        let replacement = node.visit_mut_children_with(self);
        self.lowerer.leave_node(node.node_id());
        replacement
    }

    fn visit_mut_method_definition(&mut self, node: MethodDefinition) -> Option<MethodDefinition> {
        self.lowerer
            .enter_node(NodeKindData::MethodDefinition(node), self.ast);
        let replacement = node.visit_mut_children_with(self);
        self.lowerer.leave_node(node.node_id());
        replacement
    }

    fn visit_mut_function(&mut self, node: Function) -> Option<Function> {
        self.lowerer
            .enter_node(NodeKindData::Function(node), self.ast);
        let replacement = node.visit_mut_children_with(self);
        self.lowerer.leave_node(node.node_id());
        replacement
    }

    fn visit_mut_property_definition(
        &mut self,
        node: PropertyDefinition,
    ) -> Option<PropertyDefinition> {
        self.lowerer
            .enter_node(NodeKindData::PropertyDefinition(node), self.ast);
        let replacement = node.visit_mut_children_with(self);
        self.lowerer.leave_node(node.node_id());
        replacement
    }

    fn visit_mut_static_block(&mut self, node: StaticBlock) -> Option<StaticBlock> {
        self.lowerer
            .enter_node(NodeKindData::StaticBlock(node), self.ast);
        let replacement = node.visit_mut_children_with(self);
        self.lowerer.leave_node(node.node_id());
        replacement
    }
}
