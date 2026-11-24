use crate::common::{TraverseCtx, build};
use std::collections::HashSet;
use swc_next_ecma_ast::*;

pub(crate) struct ClassStaticBlock;

impl<'ast> TraverseMut<'ast, TraverseCtx<'_>> for ClassStaticBlock {
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transformer/src/es2022/class_static_block.rs#L175>.
    fn exit_class(
        &mut self,
        class: Class,
        ctx: &mut TraverseMutCtx<'_, 'ast, TraverseCtx<'_>>,
    ) -> TraverseAction<Class> {
        let body = class.body(ctx.ast);
        let mut names = self.collect_existing_private_names(ctx.ast, body.body(ctx.ast));
        let mut elements = Vec::new();
        for slot in body.body(ctx.ast).iter() {
            let element = ctx.ast.get_node_in_sub_range(slot);
            let Some(block) = element.as_static_block(ctx.ast) else {
                elements.push(element);
                continue;
            };
            if block.body(ctx.ast).is_empty() {
                continue;
            }
            elements.push(self.transform_static_block(ctx, &mut names, block));
        }
        let elements = ctx.ast.add_sub_range(elements);
        body.set_body(ctx.ast, elements);
        TraverseAction::Continue
    }
}

impl ClassStaticBlock {
    fn collect_existing_private_names(
        &self,
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
            if let Some(id) = key
                .and_then(|key| key.as_expr(ast))
                .and_then(|expression| expression.as_private_identifier(ast))
            {
                names.insert(ast.get_utf8(id.name(ast)).to_owned());
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
        &self,
        ctx: &mut TraverseMutCtx<'_, '_, TraverseCtx<'_>>,
        names: &mut HashSet<String>,
        block: StaticBlock,
    ) -> ClassElement {
        let name = ctx.ast.add_utf8(&Self::generate_unique_name(names));
        let key = ctx.ast.expr_private_identifier(DUMMY_SP, name);
        let value = if block.body(ctx.ast).len() == 1 {
            let statement = ctx
                .ast
                .get_node_in_sub_range(block.body(ctx.ast).iter().next().unwrap());
            statement
                .as_expression_statement(ctx.ast)
                .map(|statement| statement.expression(ctx.ast))
        } else {
            None
        };
        let value = value.unwrap_or_else(|| Self::wrap_in_iife(ctx.ast, block.body(ctx.ast)));
        let property = ctx.ast.property_definition(
            block.span(ctx.ast),
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
        ctx.state.static_blocks.insert(property.node_id());
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
