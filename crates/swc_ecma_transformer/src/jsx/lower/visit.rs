use super::{JsxEmitter, JsxLowerer, JsxRoot};
use crate::syntax::{FollowingTransform, NamespaceParameter};
use rustc_hash::FxHashSet;
use swc_next_ecma_ast::SymbolId;
use swc_next_ecma_ast::VisitMutWith;
use swc_next_ecma_ast::{
    Ast, AstUtf8, Class, Expr, Function, Lang, MethodDefinition, NodeId, NodeKindData, Program,
    PropertyDefinition, StaticBlock, VisitMut,
};
use swc_next_ecma_semantic::Semantic;

impl<'ast, const TRACK_SELF: bool> FollowingTransform<'ast> for JsxLowerer<'_, 'ast, TRACK_SELF> {
    fn enter_node(&mut self, node: NodeKindData, ast: &Ast<'ast>) {
        if TRACK_SELF {
            self.self_context.enter_node(node, ast);
        }
    }

    fn enter_expression(&mut self, expression: Expr) {
        if TRACK_SELF {
            self.self_context.enter_expression(expression.node_id());
        }
    }

    fn leave_node(&mut self, node: NodeId) {
        if TRACK_SELF {
            self.self_context.leave_node(node);
        }
    }

    fn fresh_uid(&mut self, base: &'ast str, ast: &mut Ast<'ast>) -> Option<AstUtf8> {
        Some(self.uid_generator.fresh(base, ast))
    }

    fn runtime_dependencies(&mut self, semantic: &Semantic<'_>) -> FxHashSet<SymbolId> {
        self.bindings.runtime_dependencies(semantic)
    }

    fn exit_expression(
        &mut self,
        expression: Expr,
        ast: &mut Ast<'ast>,
        namespace_parameter: &NamespaceParameter<'_>,
    ) -> Option<Expr> {
        let root = JsxRoot::from_expression(expression, ast)?;
        let namespace = self
            .bindings
            .project_namespace(expression, ast, namespace_parameter);
        Some(
            JsxEmitter {
                state: self,
                namespace,
            }
            .lower_root(root, ast),
        )
    }

    fn finish_program(&mut self, program: Program, ast: &mut Ast<'ast>) {
        debug_assert!(!TRACK_SELF || self.self_context.is_empty());
        self.insert_program_bindings(program, ast);
        let source_type = ast.source_type();
        let lang = match ast.lang() {
            Lang::Jsx | Lang::Tsx => Lang::Js,
            lang => lang,
        };
        ast.set_source_metadata(source_type, lang);
    }
}

pub(in crate::jsx) struct JsxVisitor<'visit, 'options, 'ast, const TRACK_SELF: bool> {
    ast: &'visit mut Ast<'ast>,
    lowerer: &'visit mut JsxLowerer<'options, 'ast, TRACK_SELF>,
}

impl<'visit, 'options, 'ast, const TRACK_SELF: bool>
    JsxVisitor<'visit, 'options, 'ast, TRACK_SELF>
{
    pub(in crate::jsx) fn new(
        ast: &'visit mut Ast<'ast>,
        lowerer: &'visit mut JsxLowerer<'options, 'ast, TRACK_SELF>,
    ) -> Self {
        Self { ast, lowerer }
    }
}

impl<'ast, const TRACK_SELF: bool> VisitMut<'ast> for JsxVisitor<'_, '_, 'ast, TRACK_SELF> {
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
