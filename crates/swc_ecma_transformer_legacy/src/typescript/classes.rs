//! Parameter properties and SWC's class-field initialization ordering.
use super::{transform::Transform, utils};
use swc_next_ecma_ast::*;

impl Transform<'_, '_> {
    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L194>.
    pub(super) fn class_body(&mut self, body: ClassBody) {
        let mut elements: Vec<_> =
            body.body(self.ast)
                .iter()
                .map(|slot| self.ast.get_node_in_sub_range(slot))
                .filter(|element| match self.ast.class_element_data(*element) {
                    ClassElementData::TsIndexSignature(_)
                    | ClassElementData::TsMethodDefinition(_) => false,
                    ClassElementData::MethodDefinition(method) => !method.r#abstract(self.ast),
                    ClassElementData::PropertyDefinition(property) => {
                        !property.declare(self.ast) && !property.r#abstract(self.ast)
                    }
                    _ => true,
                })
                .collect();
        let mut properties = Vec::new();
        let mut initializers = Vec::new();
        let mut constructor = None;
        for &element in &elements {
            if let Some(method) = element.as_method_definition(self.ast)
                && method.kind(self.ast) == MethodDefinitionKind::Constructor
            {
                constructor = Some(method.value(self.ast));
                self.parameter_properties(
                    method.value(self.ast),
                    &mut properties,
                    &mut initializers,
                );
            }
            element.visit_mut_with(self);
        }
        if !properties.is_empty() {
            if !self.options.native_class_properties {
                self.move_field_initializers(&elements, &mut initializers);
            }
            if let Some(constructor) = constructor {
                inject_after_super(
                    self.ast,
                    self.resolver,
                    constructor.body(self.ast),
                    initializers,
                );
            }
            let mut fields = Vec::new();
            for id in properties {
                let name = self.ast.add_utf8(&id.name);
                let key = self.ast.identifier_name(DUMMY_SP, name);
                fields.push(self.ast.class_element_property_definition(
                    DUMMY_SP,
                    SubRange::empty(),
                    PropertyKey::IdentifierName(key),
                    None,
                    None,
                    false,
                    false,
                    false,
                    false,
                    false,
                    false,
                    false,
                    false,
                    false,
                    Accessibility::None,
                ));
            }
            fields.append(&mut elements);
            elements = fields;
        }
        let elements = self.ast.add_sub_range(elements);
        body.set_body(self.ast, elements);
    }

    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L236>.
    fn parameter_properties(
        &mut self,
        function: Function,
        properties: &mut Vec<crate::resolver::Id>,
        initializers: &mut Vec<Expr>,
    ) {
        let parameters = function.params(self.ast);
        for slot in parameters.items(self.ast).iter() {
            let item = self.ast.get_node_in_sub_range(slot);
            let Some(property) = item.as_ts_parameter_property(self.ast) else {
                continue;
            };
            let parameter = property.parameter(self.ast);
            parameter.set_decorators(self.ast, property.decorators(self.ast));
            let binding = parameter
                .pattern(self.ast)
                .as_binding_identifier(self.ast)
                .expect("parameter properties have identifier patterns");
            properties.push(self.binding_id(binding));
            let name = self.ast.get_utf8(binding.name(self.ast)).to_owned();
            let object = self.ast.expr_this_expression(DUMMY_SP);
            let left = utils::member(self.ast, object, &name);
            let right = self.resolver.reference(self.ast, binding);
            initializers.push(utils::assign(self.ast, left, right));
            parameters.items(self.ast).replace_slot(
                self.ast,
                slot,
                FormalParameterItem::FormalParameter(parameter),
            );
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L1492>.
    fn move_field_initializers(&mut self, elements: &[ClassElement], initializers: &mut Vec<Expr>) {
        for &element in elements {
            let Some(property) = element.as_property_definition(self.ast) else {
                continue;
            };
            if property.r#static(self.ast) || property.accessor(self.ast) {
                continue;
            }
            let Some(value) = property.value(self.ast) else {
                continue;
            };
            let mut key = property.key(self.ast);
            if property.computed(self.ast)
                && let Some(expression) = key.as_expr(self.ast)
            {
                let literal = matches!(
                    self.ast.expr_data(expression),
                    ExprData::StringLiteral(_)
                        | ExprData::NumericLiteral(_)
                        | ExprData::BigIntLiteral(_)
                        | ExprData::BooleanLiteral(_)
                        | ExprData::NullLiteral(_)
                        | ExprData::RegExpLiteral(_)
                );
                if !literal {
                    let name = alias_name(self.ast, expression);
                    let binding = self.resolver.private_binding(self.ast, &name);
                    self.var_list.push(self.binding_id(binding));
                    let left = self.resolver.reference(self.ast, binding);
                    let assignment = utils::assign(self.ast, left, expression);
                    property.set_key(self.ast, PropertyKey::Expr(assignment));
                    key = PropertyKey::Expr(self.resolver.reference(self.ast, binding));
                }
            }
            let object = self.ast.expr_this_expression(DUMMY_SP);
            let member = self.ast.expr_member_expression(
                DUMMY_SP,
                object,
                key,
                property.computed(self.ast),
                false,
            );
            initializers.push(utils::assign(self.ast, member, value));
            property.set_value(self.ast, None);
        }
    }
}

/// SWC `sym_for_expr`/`alias_ident_for` on computed property expressions.
/// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_utils/src/lib.rs#L1446>.
fn alias_name(ast: &Ast<'_>, expression: Expr) -> String {
    fn symbol(ast: &Ast<'_>, expression: Expr) -> Option<String> {
        match ast.expr_data(expression) {
            ExprData::StringLiteral(string) => {
                ast.get_wtf8(string.value(ast)).as_str().map(str::to_owned)
            }
            ExprData::ThisExpression(_) => Some("this".into()),
            ExprData::IdentifierReference(id) => Some(ast.get_utf8(id.name(ast)).to_owned()),
            ExprData::Function(function) => function
                .id(ast)
                .map(|id| ast.get_utf8(id.name(ast)).to_owned()),
            ExprData::Class(class) => class
                .id(ast)
                .map(|id| ast.get_utf8(id.name(ast)).to_owned()),
            ExprData::MemberExpression(member) => {
                let object = if member.object(ast).is_super(ast) {
                    "super".into()
                } else {
                    symbol(ast, member.object(ast)).unwrap_or_default()
                };
                let key = if member.computed(ast) {
                    symbol(ast, member.property(ast).as_expr(ast)?).unwrap_or_default()
                } else {
                    ast.get_utf8(member.property(ast).as_identifier_name(ast)?.name(ast))
                        .to_owned()
                };
                Some(format!("{object}_{key}"))
            }
            ExprData::CallExpression(call) => symbol(ast, call.callee(ast)),
            ExprData::ChainExpression(chain) => symbol(ast, chain.expression(ast)),
            _ => None,
        }
    }
    let name = symbol(ast, expression).unwrap_or_else(|| "_key".into());
    let mut name: String = name
        .chars()
        .filter(|c| is_identifier(format!("a{c}").as_bytes()))
        .collect();
    if name.is_empty() {
        name.push('_');
    }
    if name.starts_with('_') {
        name
    } else {
        format!("_{name}")
    }
}

/// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_utils/src/constructor.rs#L9>.
fn inject_after_super(
    ast: &mut Ast<'_>,
    resolver: &mut crate::resolver::Resolver,
    body: FunctionBody,
    expressions: Vec<Expr>,
) {
    if expressions.is_empty() {
        return;
    }
    let mut injector = Injector {
        ast,
        resolver,
        expressions,
        ignore_return_value: false,
        injected: false,
    };
    body.visit_mut_with(&mut injector);
    if !injector.injected {
        let mut statements: Vec<_> = injector
            .expressions
            .iter()
            .map(|&expression| injector.ast.stmt_expression_statement(DUMMY_SP, expression))
            .collect();
        statements.extend(
            body.body(injector.ast)
                .iter()
                .map(|slot| injector.ast.get_node_in_sub_range(slot)),
        );
        let statements = injector.ast.add_sub_range(statements);
        body.set_body(injector.ast, statements);
    }
}

struct Injector<'a, 'ast> {
    ast: &'a mut Ast<'ast>,
    resolver: &'a mut crate::resolver::Resolver,
    expressions: Vec<Expr>,
    ignore_return_value: bool,
    injected: bool,
}
impl<'ast> VisitMut<'ast> for Injector<'_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }
    fn visit_mut_function(&mut self, _: Function) -> Option<Function> {
        None
    }
    fn visit_mut_method_definition(
        &mut self,
        method: MethodDefinition,
    ) -> Option<MethodDefinition> {
        if method.kind(self.ast) != MethodDefinitionKind::Constructor {
            method.visit_mut_children_with(self);
        }
        None
    }
    fn visit_mut_expression_statement(
        &mut self,
        statement: ExpressionStatement,
    ) -> Option<ExpressionStatement> {
        let old = std::mem::replace(&mut self.ignore_return_value, true);
        statement.visit_mut_children_with(self);
        self.ignore_return_value = old;
        None
    }
    fn visit_mut_sequence_expression(
        &mut self,
        sequence: SequenceExpression,
    ) -> Option<SequenceExpression> {
        let expressions = sequence.expressions(self.ast);
        for (index, slot) in expressions.iter().enumerate() {
            let expression = self.ast.get_node_in_sub_range(slot);
            let old = self.ignore_return_value;
            if index + 1 != expressions.len() {
                self.ignore_return_value = true;
            }
            if let Some(replacement) = expression.visit_mut_with(self) {
                expressions.replace_slot(self.ast, slot, replacement);
            }
            self.ignore_return_value = old;
        }
        None
    }
    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_utils/src/constructor.rs#L72>.
    fn visit_mut_expr(&mut self, expression: Expr) -> Option<Expr> {
        let ignore = self.ignore_return_value;
        if !expression.is_parenthesized_expression(self.ast)
            && !expression.is_sequence_expression(self.ast)
        {
            self.ignore_return_value = false;
        }
        expression.visit_mut_children_with(self);
        self.ignore_return_value = ignore;
        let call = expression.as_call_expression(self.ast)?;
        if !call.callee(self.ast).is_super(self.ast) {
            return None;
        }
        self.injected = true;
        let mut expressions = vec![expression];
        for &expression in &self.expressions {
            expressions.push(utils::clone_expr(self.ast, self.resolver, expression));
        }
        Some(if ignore {
            let expressions = self.ast.add_sub_range(expressions);
            self.ast.expr_sequence_expression(DUMMY_SP, expressions)
        } else {
            let elements = self.ast.add_sub_range(
                expressions
                    .into_iter()
                    .map(|expression| Some(Argument::Expr(expression))),
            );
            let array = self.ast.expr_array_expression(DUMMY_SP, elements);
            let zero = self.ast.expr_numeric_literal(DUMMY_SP, 0.0, None);
            utils::computed(self.ast, array, zero)
        })
    }
}
