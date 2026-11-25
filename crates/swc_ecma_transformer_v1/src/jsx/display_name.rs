use swc_next_ecma_ast::*;

use crate::common::{build as b, resolver::context::Resolver};

/// SWC's displayName hooks for legacy React class factories.
/// <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/display_name.rs>.
pub(super) fn enter_assignment(
    ast: &mut Ast<'_>,
    resolver: &mut Resolver,
    assignment: AssignmentExpression,
) {
    if assignment.operator(ast) == AssignmentOperator::Assign
        && let Some(name) = target_name(ast, assignment.left(ast))
    {
        fold(ast, resolver, assignment.right(ast), Name::Text(name));
    }
}

pub(super) fn enter_declarator(
    ast: &mut Ast<'_>,
    resolver: &mut Resolver,
    declaration: VariableDeclarator,
) {
    if let Some(binding) = declaration.id(ast).as_binding_identifier(ast)
        && let Some(init) = declaration.init(ast)
    {
        fold(ast, resolver, init, Name::Text(binding.name(ast)));
    }
}

pub(super) fn exit_export_default(
    ast: &mut Ast<'_>,
    resolver: &mut Resolver,
    export: ExportDefaultDeclaration,
) {
    if let Some(expression) = export.declaration(ast).as_expr(ast) {
        let name = ast.add_utf8("input");
        fold(ast, resolver, expression, Name::Text(name));
    }
}

pub(super) fn exit_property(ast: &mut Ast<'_>, resolver: &mut Resolver, property: ObjectProperty) {
    if property.kind(ast) == PropertyKind::Init && !property.method(ast) && !property.shorthand(ast)
    {
        let name = match ast.property_key_data(property.key(ast)) {
            PropertyKeyData::IdentifierName(key) => Name::Text(key.name(ast)),
            PropertyKeyData::Expr(key) => Name::Expr(key),
        };
        fold(ast, resolver, property.value(ast), name);
    }
}

enum Name {
    Text(AstUtf8),
    Expr(Expr),
}

fn fold(ast: &mut Ast<'_>, resolver: &mut Resolver, expression: Expr, name: Name) {
    expression.visit_mut_with(&mut Folder {
        ast,
        resolver,
        name: Some(name),
    });
}

fn target_name(ast: &Ast<'_>, target: AssignmentTarget) -> Option<AstUtf8> {
    let target = target.as_simple_assignment_target(ast)?;
    if let Some(ident) = target.as_identifier_reference(ast) {
        return Some(ident.name(ast));
    }
    let member = target.as_member_expression(ast)?;
    if member.computed(ast) {
        return None;
    }
    Some(member.property(ast).as_identifier_name(ast)?.name(ast))
}

struct Folder<'a, 'ast> {
    ast: &'a mut Ast<'ast>,
    resolver: &'a mut Resolver,
    name: Option<Name>,
}

impl<'ast> VisitMut<'ast> for Folder<'_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }

    fn visit_mut_array_expression(&mut self, _: ArrayExpression) -> Option<ArrayExpression> {
        None
    }

    fn visit_mut_object_expression(&mut self, _: ObjectExpression) -> Option<ObjectExpression> {
        None
    }

    fn visit_mut_call_expression(&mut self, call: CallExpression) -> Option<CallExpression> {
        call.visit_mut_children_with(self);
        if self.name.is_some() && is_create_class_call(self.ast, call) {
            add_display_name(self.ast, self.resolver, call, self.name.take().unwrap());
        }
        None
    }
}

fn is_create_class_call(ast: &Ast<'_>, call: CallExpression) -> bool {
    if call.optional(ast) {
        return false;
    }
    let is_ident = |expr: Expr, name| {
        expr.as_identifier_reference(ast)
            .is_some_and(|id| ast.get_utf8(id.name(ast)) == name)
    };
    let callee = call.callee(ast);
    if let Some(member) = callee.as_member_expression(ast) {
        !member.computed(ast)
            && !member.optional(ast)
            && is_ident(member.object(ast), "React")
            && member
                .property(ast)
                .as_identifier_name(ast)
                .is_some_and(|id| ast.get_utf8(id.name(ast)) == "createClass")
    } else {
        is_ident(callee, "createReactClass")
    }
}

fn add_display_name(ast: &mut Ast<'_>, resolver: &mut Resolver, call: CallExpression, name: Name) {
    let Some(argument) = call.arguments(ast).iter().next() else {
        return;
    };
    let argument = ast.get_node_in_sub_range(argument);
    let argument = match ast.argument_data(argument) {
        ArgumentData::Expr(expression) => expression,
        ArgumentData::SpreadElement(spread) => spread.argument(ast),
    };
    let Some(object) = argument.as_object_expression(ast) else {
        return;
    };
    let mut properties: Vec<_> = object
        .properties(ast)
        .iter()
        .map(|slot| ast.get_node_in_sub_range(slot))
        .collect();
    if properties
        .iter()
        .any(|&property| is_key_display_name(ast, property))
    {
        return;
    }

    let name = match name {
        Name::Text(name) => ast.expr_string_literal(DUMMY_SP, name.into(), AstUtf8::EMPTY),
        Name::Expr(name) => b::clone_expr(ast, resolver, name),
    };
    properties.push(b::property(ast, "displayName", name));
    let properties = ast.add_sub_range(properties);
    object.set_properties(ast, properties);
}

fn is_key_display_name(ast: &Ast<'_>, property: ObjectPropertyKind) -> bool {
    let Some(property) = property.as_object_property(ast) else {
        return false;
    };
    if property.computed(ast) {
        return false;
    }
    match ast.property_key_data(property.key(ast)) {
        PropertyKeyData::IdentifierName(name) => ast.get_utf8(name.name(ast)) == "displayName",
        PropertyKeyData::Expr(name) => name
            .as_string_literal(ast)
            .is_some_and(|name| ast.get_wtf8(name.value(ast)).as_str() == Some("displayName")),
    }
}
