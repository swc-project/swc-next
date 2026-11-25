use crate::common::build;
use std::collections::HashSet;
use swc_next_ecma_ast::*;

pub(crate) fn pass(ast: &mut Ast<'_>, static_blocks: &mut HashSet<NodeId>) {
    ast.root_program()
        .visit_mut_with(&mut ClassStaticBlock { ast, static_blocks });
}

struct ClassStaticBlock<'a, 'ast> {
    ast: &'a mut Ast<'ast>,
    static_blocks: &'a mut HashSet<NodeId>,
}

impl<'ast> VisitMut<'ast> for ClassStaticBlock<'_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transformer/src/es2022/class_static_block.rs#L175>.
    fn visit_mut_class(&mut self, class: Class) -> Option<Class> {
        class.visit_mut_children_with(self);
        let body = class.body(self.ast);
        let members = body.body(self.ast);
        let mut names = Self::collect_existing_private_names(self.ast, members);
        let mut elements = Vec::new();
        for slot in members.iter() {
            let element = self.ast.get_node_in_sub_range(slot);
            let Some(block) = element.as_static_block(self.ast) else {
                elements.push(element);
                continue;
            };
            if block.body(self.ast).is_empty() {
                continue;
            }
            elements.push(self.transform_static_block(&mut names, block));
        }
        let elements = self.ast.add_sub_range(elements);
        body.set_body(self.ast, elements);
        None
    }
}

impl ClassStaticBlock<'_, '_> {
    fn collect_existing_private_names(
        ast: &Ast<'_>,
        body: SubRange<ClassElement>,
    ) -> HashSet<String> {
        let mut names = HashSet::new();
        for slot in body.iter() {
            let element = ast.get_node_in_sub_range(slot);
            let key = match ast.class_element_data(element) {
                ClassElementData::PropertyDefinition(property) if !property.accessor(ast) => {
                    Some(property.key(ast))
                }
                ClassElementData::MethodDefinition(method) => Some(method.key(ast)),
                _ => None,
            };
            if let Some(key) = key
                && let Some(name) = build::private_name(ast, key)
            {
                names.insert(name);
            }
        }
        names
    }

    fn generate_unique_name(names: &mut HashSet<String>) -> String {
        let mut name = "_".to_owned();
        let mut suffix = 2;
        while names.contains(&name) {
            name = format!("_{suffix}");
            suffix += 1;
        }
        names.insert(name.clone());
        name
    }

    fn transform_static_block(
        &mut self,
        names: &mut HashSet<String>,
        block: StaticBlock,
    ) -> ClassElement {
        let name = self.ast.add_utf8(&Self::generate_unique_name(names));
        let key = self.ast.expr_private_identifier(DUMMY_SP, name);
        let statements = block.body(self.ast);
        let value = if statements.len() == 1 {
            let statement = statements.get_node(self.ast, 0).unwrap();
            statement
                .as_expression_statement(self.ast)
                .map(|statement| statement.expression(self.ast))
        } else {
            None
        };
        let value = value.unwrap_or_else(|| Self::wrap_in_iife(self.ast, statements));
        let property = self.ast.property_definition(
            block.span(self.ast),
            SubRange::empty(),
            PropertyKey::Expr(key),
            None,
            Some(value),
            false,
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            Accessibility::None,
        );
        // SWC uses PLACEHOLDER_SP on the generated key. The flat AST keeps
        // that marker out of source spans and records it by node id instead.
        self.static_blocks.insert(property.node_id());
        ClassElement::PropertyDefinition(property)
    }

    fn wrap_in_iife(ast: &mut Ast<'_>, statements: SubRange<Stmt>) -> Expr {
        let body = ast.function_body(DUMMY_SP, SubRange::empty(), statements);
        let params = ast.formal_parameters(
            DUMMY_SP,
            SubRange::empty(),
            None,
            FormalParameterKind::ArrowFormalParameters,
        );
        let arrow = ast.expr_arrow_function_expression(
            DUMMY_SP,
            false,
            false,
            None,
            params,
            None,
            ArrowFunctionBody::FunctionBody(body),
        );
        build::call(ast, arrow, [], false)
    }
}
