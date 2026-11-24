use swc_next_ecma_ast::*;

pub(super) struct UsedNameCollector<'a, 'ast> {
    pub ast: &'a Ast<'ast>,
    pub names: Vec<String>,
}
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_compat_es2022/src/class_properties/used_name.rs#L23>.
impl<'ast> Visit<'ast> for UsedNameCollector<'_, 'ast> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }
    fn visit_function(&mut self, _: Function) {}
    fn visit_arrow_function_expression(&mut self, _: ArrowFunctionExpression) {}
    fn visit_object_property(&mut self, node: ObjectProperty) {
        if !node.method(self.ast) && node.kind(self.ast) == PropertyKind::Init {
            node.visit_children_with(self);
        }
    }
    fn visit_method_definition(&mut self, node: MethodDefinition) {
        if node.kind(self.ast) != MethodDefinitionKind::Constructor {
            node.visit_children_with(self);
        }
    }
    fn visit_identifier_reference(&mut self, node: IdentifierReference) {
        self.names
            .push(self.ast.get_utf8(node.name(self.ast)).to_owned());
    }
}
