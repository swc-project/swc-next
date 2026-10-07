//! Expression-edge edits are private to the transformer, not AST visitors.
use super::Mutation;
use swc_next_ecma_ast::{
    Argument, ArrowFunctionBody, Ast, ExportDefaultDeclarationKind, Expr, ForStatementInit,
    JsxExpression, NodeId, NodeKindData, PropertyKey,
};

impl Mutation<'_, '_, '_> {
    /// Replace a leaf expression; its old arena storage remains allocated.
    pub(crate) fn replace_current_expr(&mut self, replacement: Expr) {
        let old = self
            .current_node()
            .expect("replacement requires a current node");
        assert!(
            matches!(
                self.ast.kind_data(old),
                NodeKindData::IdentifierReference(_)
                    | NodeKindData::ThisExpression(_)
                    | NodeKindData::MetaProperty(_)
                    | NodeKindData::RegExpLiteral(_)
                    | NodeKindData::StringLiteral(_)
                    | NodeKindData::NumericLiteral(_)
                    | NodeKindData::BigIntLiteral(_)
                    | NodeKindData::BooleanLiteral(_)
                    | NodeKindData::NullLiteral(_)
            ),
            "replacement must preserve non-leaf children explicitly"
        );
        let old = self.replace_current_expr_edge(replacement);
        if old != replacement.node_id() {
            let old = self.ast.kind_data(old);
            self.state.semantic.replace_leaf_expression(old);
        }
    }

    /// Keep the old expression and all its descendants inside the new wrapper.
    pub(crate) fn wrap_current_expr(&mut self, wrapper: Expr) {
        let old = self
            .current_node()
            .expect("wrapping requires a current node");
        self.assert_expression_wrapper(old, wrapper);
        self.replace_current_expr_edge(wrapper);
    }

    pub(super) fn assert_expression_wrapper(&mut self, old: NodeId, wrapper: Expr) {
        assert!(
            old == wrapper.node_id()
                || replace_expression_edge::<false>(self.ast, wrapper.node_id(), old, wrapper),
            "wrapper must retain the old expression as a direct child"
        );
    }

    /// Keep every child after explicitly transferring the old owner's scope.
    pub(crate) fn replace_current_expr_preserving_children(&mut self, replacement: Expr) {
        self.replace_current_expr_edge(replacement);
    }

    fn replace_current_expr_edge(&mut self, replacement: Expr) -> NodeId {
        let old = self
            .current_node()
            .expect("replacement requires a current node");
        let parent = self.parent().expect("expression has a parent");
        assert_eq!(
            self.state.semantic.scope_outside(replacement.node_id()),
            self.state.semantic.scope_outside(old),
            "replacement must be registered in the expression's evaluation scope"
        );
        assert!(
            replace_expression_edge::<true>(self.ast, parent, old, replacement),
            "current node must occupy an expression edge in its parent"
        );
        if matches!(
            self.ast.kind_data(parent),
            NodeKindData::ExportDefaultDeclaration(_)
        ) {
            self.state
                .semantic
                .replace_export_target(self.ast, parent, replacement.node_id());
        }
        old
    }
}

/// All supported expression containers are handled here. This never descends
/// into sibling subtrees and never exposes a parent setter to a transform pass.
fn replace_expression_edge<const APPLY: bool>(
    ast: &mut Ast<'_>,
    parent: NodeId,
    old: NodeId,
    replacement: Expr,
) -> bool {
    macro_rules! field {
        ($node:ident, $get:ident, $set:ident, $value:expr) => {
            if $node.$get(ast).node_id() == old {
                if APPLY {
                    $node.$set(ast, $value);
                }
                return true;
            }
        };
    }
    macro_rules! optional_field {
        ($node:ident, $get:ident, $set:ident, $value:expr) => {
            if $node.$get(ast).is_some_and(|child| child.node_id() == old) {
                if APPLY {
                    $node.$set(ast, Some($value));
                }
                return true;
            }
        };
    }
    macro_rules! list {
        ($node:ident, $get:ident, $set:ident, $value:expr) => {
            let children = $node.$get(ast);
            for slot in children.iter() {
                if ast.get_node_in_sub_range(slot).node_id() == old {
                    if APPLY {
                        children.replace_slot(ast, slot, $value);
                    }
                    return true;
                }
            }
        };
    }
    macro_rules! optional_list {
        ($node:ident, $get:ident, $set:ident, $value:expr) => {
            let children = $node.$get(ast);
            for slot in children.iter() {
                if ast
                    .get_node_in_sub_range(slot)
                    .is_some_and(|child| child.node_id() == old)
                {
                    if APPLY {
                        children.replace_slot(ast, slot, Some($value));
                    }
                    return true;
                }
            }
        };
    }
    match ast.kind_data(parent) {
        NodeKindData::TsMethodDefinition(node) => {
            field!(node, key, set_key, PropertyKey::Expr(replacement));
        }
        NodeKindData::TsImportType(node) => {
            optional_field!(node, options, set_options, replacement);
        }
        NodeKindData::TsPropertySignature(node) => {
            field!(node, key, set_key, PropertyKey::Expr(replacement));
        }
        NodeKindData::TsMethodSignature(node) => {
            field!(node, key, set_key, PropertyKey::Expr(replacement));
        }
        NodeKindData::TsInterfaceHeritage(node) => {
            field!(node, expression, set_expression, replacement);
        }
        NodeKindData::TsClassImplements(node) => {
            field!(node, expression, set_expression, replacement);
        }
        NodeKindData::TsEnumMember(node) => {
            optional_field!(node, initializer, set_initializer, replacement);
        }
        NodeKindData::TsAsExpression(node) => {
            field!(node, expression, set_expression, replacement);
        }
        NodeKindData::TsSatisfiesExpression(node) => {
            field!(node, expression, set_expression, replacement);
        }
        NodeKindData::TsTypeAssertion(node) => {
            field!(node, expression, set_expression, replacement);
        }
        NodeKindData::TsNonNullExpression(node) => {
            field!(node, expression, set_expression, replacement);
        }
        NodeKindData::TsInstantiationExpression(node) => {
            field!(node, expression, set_expression, replacement);
        }
        NodeKindData::TsExportAssignment(node) => {
            field!(node, expression, set_expression, replacement);
        }
        NodeKindData::TemplateLiteral(node) => {
            list!(node, expressions, set_expressions, replacement);
        }
        NodeKindData::AssignmentPattern(node) => {
            field!(node, right, set_right, replacement);
        }
        NodeKindData::BindingProperty(node) => {
            field!(node, key, set_key, PropertyKey::Expr(replacement));
        }
        NodeKindData::BinaryExpression(node) => {
            field!(node, left, set_left, replacement);
            field!(node, right, set_right, replacement);
        }
        NodeKindData::LogicalExpression(node) => {
            field!(node, left, set_left, replacement);
            field!(node, right, set_right, replacement);
        }
        NodeKindData::ConditionalExpression(node) => {
            field!(node, test, set_test, replacement);
            field!(node, consequent, set_consequent, replacement);
            field!(node, alternate, set_alternate, replacement);
        }
        NodeKindData::UnaryExpression(node) => {
            field!(node, argument, set_argument, replacement);
        }
        NodeKindData::AssignmentExpression(node) => {
            field!(node, right, set_right, replacement);
        }
        NodeKindData::VariableDeclarator(node) => {
            optional_field!(node, init, set_init, replacement);
        }
        NodeKindData::IfStatement(node) => {
            field!(node, test, set_test, replacement);
        }
        NodeKindData::SwitchStatement(node) => {
            field!(node, discriminant, set_discriminant, replacement);
        }
        NodeKindData::SwitchCase(node) => {
            optional_field!(node, test, set_test, replacement);
        }
        NodeKindData::ForStatement(node) => {
            optional_field!(node, init, set_init, ForStatementInit::Expr(replacement));
            optional_field!(node, test, set_test, replacement);
            optional_field!(node, update, set_update, replacement);
        }
        NodeKindData::ForInStatement(node) => {
            field!(node, right, set_right, replacement);
        }
        NodeKindData::ForOfStatement(node) => {
            field!(node, right, set_right, replacement);
        }
        NodeKindData::ExpressionStatement(node) => {
            field!(node, expression, set_expression, replacement);
        }
        NodeKindData::ReturnStatement(node) => {
            optional_field!(node, argument, set_argument, replacement);
        }
        NodeKindData::ThrowStatement(node) => {
            field!(node, argument, set_argument, replacement);
        }
        NodeKindData::Decorator(node) => {
            field!(node, expression, set_expression, replacement);
        }
        NodeKindData::Class(node) => {
            optional_field!(node, super_class, set_super_class, replacement);
        }
        NodeKindData::MethodDefinition(node) => {
            field!(node, key, set_key, PropertyKey::Expr(replacement));
        }
        NodeKindData::PropertyDefinition(node) => {
            field!(node, key, set_key, PropertyKey::Expr(replacement));
            optional_field!(node, value, set_value, replacement);
        }
        NodeKindData::WhileStatement(node) => {
            field!(node, test, set_test, replacement);
        }
        NodeKindData::DoWhileStatement(node) => {
            field!(node, test, set_test, replacement);
        }
        NodeKindData::WithStatement(node) => {
            field!(node, object, set_object, replacement);
        }
        NodeKindData::ArrayExpression(node) => {
            optional_list!(node, elements, set_elements, Argument::Expr(replacement));
        }
        NodeKindData::SpreadElement(node) => {
            field!(node, argument, set_argument, replacement);
        }
        NodeKindData::ObjectProperty(node) => {
            field!(node, key, set_key, PropertyKey::Expr(replacement));
            field!(node, value, set_value, replacement);
        }
        NodeKindData::ParenthesizedExpression(node) => {
            field!(node, expression, set_expression, replacement);
        }
        NodeKindData::ArrowFunctionExpression(node) => {
            field!(node, body, set_body, ArrowFunctionBody::Expr(replacement));
        }
        NodeKindData::SequenceExpression(node) => {
            list!(node, expressions, set_expressions, replacement);
        }
        NodeKindData::MemberExpression(node) => {
            field!(node, object, set_object, replacement);
            field!(node, property, set_property, PropertyKey::Expr(replacement));
        }
        NodeKindData::CallExpression(node) => {
            field!(node, callee, set_callee, replacement);
            list!(node, arguments, set_arguments, Argument::Expr(replacement));
        }
        NodeKindData::ChainExpression(node) => {
            field!(node, expression, set_expression, replacement);
        }
        NodeKindData::TaggedTemplateExpression(node) => {
            field!(node, tag, set_tag, replacement);
        }
        NodeKindData::NewExpression(node) => {
            field!(node, callee, set_callee, replacement);
            list!(node, arguments, set_arguments, Argument::Expr(replacement));
        }
        NodeKindData::AwaitExpression(node) => {
            field!(node, argument, set_argument, replacement);
        }
        NodeKindData::YieldExpression(node) => {
            optional_field!(node, argument, set_argument, replacement);
        }
        NodeKindData::ImportExpression(node) => {
            field!(node, source, set_source, replacement);
            optional_field!(node, options, set_options, replacement);
        }
        NodeKindData::ExportDefaultDeclaration(node) => {
            field!(
                node,
                declaration,
                set_declaration,
                ExportDefaultDeclarationKind::Expr(replacement)
            );
        }
        NodeKindData::JsxSpreadAttribute(node) => {
            field!(node, argument, set_argument, replacement);
        }
        NodeKindData::JsxExpressionContainer(node) => {
            field!(
                node,
                expression,
                set_expression,
                JsxExpression::Expr(replacement)
            );
        }
        NodeKindData::JsxSpreadChild(node) => {
            field!(node, expression, set_expression, replacement);
        }
        _ => {}
    }
    false
}
