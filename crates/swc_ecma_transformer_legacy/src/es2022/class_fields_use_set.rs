use crate::common::{build as b, constructor, resolver::context::Resolver};
use swc_next_ecma_ast::*;

/// SWC's `class_fields_use_set(pure_getters)` configuration.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ClassFieldsUseSetOptions {
    /// Assume property getters have no side effects when caching computed keys.
    pub pure_getters: bool,
}
struct ClassFieldsUseSet<'a, 'ast> {
    ast: &'a mut Ast<'ast>,
    resolver: &'a mut Resolver,
    options: ClassFieldsUseSetOptions,
}
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_compat/src/class_fields_use_set.rs#L87>.
pub(crate) fn class_fields_use_set(
    ast: &mut Ast<'_>,
    resolver: &mut Resolver,
    options: ClassFieldsUseSetOptions,
) {
    ast.root_program().visit_mut_with(&mut ClassFieldsUseSet {
        ast,
        resolver,
        options,
    });
}
impl ClassFieldsUseSet<'_, '_> {
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_compat/src/class_fields_use_set.rs#L125>.
    fn visit_mut_stmts_like(&mut self, body: SubRange<Stmt>) -> SubRange<Stmt> {
        let mut statements = Vec::new();
        let mut computed = ComputedFieldsHandler {
            ast: self.ast,
            resolver: self.resolver,
            pure_getters: self.options.pure_getters,
            var_decls: Vec::new(),
            static_init_blocks: Vec::new(),
        };
        for slot in body.iter() {
            let statement = computed.ast.get_node_in_sub_range(slot);
            statement.visit_mut_with(&mut computed);
            if !computed.var_decls.is_empty() {
                statements.push(b::declaration(
                    computed.ast,
                    VariableKind::Let,
                    std::mem::take(&mut computed.var_decls),
                ));
            }
            // Reborrow after the computed-key pass, matching SWC's two visitors.
            let mut pass = FieldsHandler {
                ast: computed.ast,
                resolver: computed.resolver,
                options: self.options,
            };
            statements.push(statement.visit_mut_with(&mut pass).unwrap_or(statement));
        }
        computed.ast.add_sub_range(statements)
    }
}
impl<'ast> VisitMut<'ast> for ClassFieldsUseSet<'_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }
    fn visit_mut_stmts(&mut self, body: SubRange<Stmt>) -> Option<SubRange<Stmt>> {
        Some(self.visit_mut_stmts_like(body))
    }
}

struct FieldsHandler<'a, 'ast> {
    ast: &'a mut Ast<'ast>,
    resolver: &'a mut Resolver,
    options: ClassFieldsUseSetOptions,
}

impl<'ast> VisitMut<'ast> for FieldsHandler<'_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }

    fn visit_mut_stmts(&mut self, body: SubRange<Stmt>) -> Option<SubRange<Stmt>> {
        ClassFieldsUseSet {
            ast: self.ast,
            resolver: self.resolver,
            options: self.options,
        }
        .visit_mut_stmts(body)
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_compat/src/class_fields_use_set.rs#L177>.
    fn visit_mut_class(&mut self, class: Class) -> Option<Class> {
        class.visit_mut_children_with(self);
        let body = class.body(self.ast);
        let mut members = Vec::new();
        let mut initializers = Vec::new();
        let mut constructor = None;
        for slot in body.body(self.ast).iter() {
            let member = self.ast.get_node_in_sub_range(slot);
            if let Some(method) = member
                .as_method_definition(self.ast)
                .filter(|method| method.kind(self.ast) == MethodDefinitionKind::Constructor)
            {
                constructor = Some(method);
            }
            if let Some(prop) = member
                .as_property_definition(self.ast)
                .filter(|prop| !prop.accessor(self.ast))
            {
                let private = b::private_name(self.ast, prop.key(self.ast)).is_some();
                if !private || !prop.r#static(self.ast) {
                    if let Some(value) = prop.value(self.ast) {
                        let this = self.ast.expr_this_expression(DUMMY_SP);
                        let target = self.ast.expr_member_expression(
                            DUMMY_SP,
                            this,
                            prop.key(self.ast),
                            prop.computed(self.ast)
                                || !prop.key(self.ast).is_identifier_name(self.ast) && !private,
                            false,
                        );
                        let assignment = b::assign(self.ast, target, value);
                        prop.set_value(self.ast, None);
                        if prop.r#static(self.ast) {
                            let statement =
                                self.ast.stmt_expression_statement(DUMMY_SP, assignment);
                            let statements = self.ast.add_sub_range([statement]);
                            members.push(ClassElement::StaticBlock(
                                self.ast.static_block(DUMMY_SP, statements),
                            ));
                            continue;
                        }
                        initializers.push(assignment);
                    }
                    if !private {
                        continue;
                    }
                }
            }
            members.push(member);
        }
        if !initializers.is_empty() {
            let constructor = constructor.unwrap_or_else(|| {
                let method = constructor::default_constructor(
                    self.ast,
                    self.resolver,
                    class.super_class(self.ast).is_some(),
                );
                members.push(ClassElement::MethodDefinition(method));
                method
            });
            constructor::inject_after_super(
                self.ast,
                self.resolver,
                constructor.value(self.ast).body(self.ast),
                initializers,
            );
        }
        let members = self.ast.add_sub_range(members);
        body.set_body(self.ast, members);
        None
    }
}
struct ComputedFieldsHandler<'a, 'ast> {
    ast: &'a mut Ast<'ast>,
    resolver: &'a mut Resolver,
    pure_getters: bool,
    var_decls: Vec<VariableDeclarator>,
    static_init_blocks: Vec<Stmt>,
}
impl<'ast> VisitMut<'ast> for ComputedFieldsHandler<'_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_compat/src/class_fields_use_set.rs#L271>.
    fn visit_mut_property_definition(
        &mut self,
        prop: PropertyDefinition,
    ) -> Option<PropertyDefinition> {
        if prop.computed(self.ast) {
            let expression = prop.key(self.ast).as_expr(self.ast).unwrap();
            if !b::is_literal(self.ast, expression)
                && !is_simple_pure_expr(self.ast, expression, self.pure_getters)
            {
                let binding = self.resolver.private_binding(self.ast, "prop");
                self.var_decls.push(b::declarator(self.ast, binding, None));
                let reference = self.resolver.reference(self.ast, binding);
                prop.set_key(self.ast, PropertyKey::Expr(reference));
                let reference = self.resolver.reference(self.ast, binding);
                let assignment = b::assign(self.ast, reference, expression);
                self.static_init_blocks
                    .push(self.ast.stmt_expression_statement(DUMMY_SP, assignment));
            }
        }
        None
    }
    fn visit_mut_class_element(&mut self, node: ClassElement) -> Option<ClassElement> {
        if let Some(prop) = node.as_property_definition(self.ast).filter(|prop| {
            !prop.accessor(self.ast) && b::private_name(self.ast, prop.key(self.ast)).is_none()
        }) {
            prop.visit_mut_with(self);
        }
        None
    }
    fn visit_mut_class_body(&mut self, node: ClassBody) -> Option<ClassBody> {
        node.visit_mut_children_with(self);
        if !self.static_init_blocks.is_empty() {
            let statements = self
                .ast
                .add_sub_range(std::mem::take(&mut self.static_init_blocks));
            let block = ClassElement::StaticBlock(self.ast.static_block(DUMMY_SP, statements));
            let body: Vec<_> = std::iter::once(block)
                .chain(
                    node.body(self.ast)
                        .iter()
                        .map(|slot| self.ast.get_node_in_sub_range(slot)),
                )
                .collect();
            let body = self.ast.add_sub_range(body);
            node.set_body(self.ast, body);
        }
        None
    }
}
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_utils/src/lib.rs#L1346>.
fn is_simple_pure_expr(ast: &Ast<'_>, expression: Expr, pure_getters: bool) -> bool {
    match ast.expr_data(expression) {
        ExprData::IdentifierReference(_)
        | ExprData::ThisExpression(_)
        | ExprData::StringLiteral(_)
        | ExprData::NumericLiteral(_)
        | ExprData::BigIntLiteral(_)
        | ExprData::BooleanLiteral(_)
        | ExprData::NullLiteral(_)
        | ExprData::RegExpLiteral(_) => true,
        ExprData::UnaryExpression(unary)
            if matches!(
                unary.operator(ast),
                UnaryOperator::Void | UnaryOperator::LogicalNot
            ) =>
        {
            is_simple_pure_expr(ast, unary.argument(ast), pure_getters)
        }
        ExprData::MemberExpression(member) if pure_getters => simple_member(ast, member),
        _ => false,
    }
}
fn simple_member(ast: &Ast<'_>, member: MemberExpression) -> bool {
    is_simple_pure_expr(ast, member.object(ast), true)
        && (!member.computed(ast)
            || is_simple_pure_expr(ast, member.property(ast).as_expr(ast).unwrap(), true))
}
