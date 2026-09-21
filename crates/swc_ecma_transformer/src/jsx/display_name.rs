use std::path::Path;

use swc_next_ecma_ast::{
    Ast, AstUtf8, CallExpression, DUMMY_SP, ExprData, NodeKindData, ObjectPropertyKind,
    PropertyKey, PropertyKind, SimpleAssignmentTargetData,
};

use crate::context::TransformCtx;

pub(crate) struct DisplayName {
    source_path: Option<String>,
}

impl DisplayName {
    pub(crate) fn new(source_path: Option<String>) -> Self {
        Self { source_path }
    }

    pub(crate) fn exit_call_expression(
        &self,
        call: CallExpression,
        ctx: &mut TransformCtx<'_, '_>,
    ) {
        let Some(object) = create_class_object(call, ctx.ast) else {
            return;
        };
        if has_display_name(object, ctx.ast) {
            return;
        }
        let Some(name) = infer_name(ctx, self.source_path.as_deref()) else {
            return;
        };
        let name = ctx.ast.add_utf8(name);
        let value = swc_next_ecma_ast::Expr::StringLiteral(
            ctx.add_node(|ast| ast.string_literal(DUMMY_SP, name.into(), AstUtf8::EMPTY)),
        );
        let display_name = ctx.ast.add_utf8("displayName");
        let key = PropertyKey::IdentifierName(
            ctx.add_node(|ast| ast.identifier_name(DUMMY_SP, display_name)),
        );
        let property = ObjectPropertyKind::ObjectProperty(ctx.add_node(|ast| {
            ast.object_property(
                DUMMY_SP,
                key,
                value,
                PropertyKind::Init,
                false,
                false,
                false,
            )
        }));
        let existing = object.properties(ctx.ast);
        let mut properties = Vec::with_capacity(existing.len() + 1);
        properties.push(property);
        properties.extend(existing.iter().map(|id| ctx.ast.get_node_in_sub_range(id)));
        let properties = ctx.ast.add_sub_range(properties);
        object.set_properties(ctx.ast, properties);
    }
}

fn create_class_object(
    call: CallExpression,
    ast: &Ast<'_>,
) -> Option<swc_next_ecma_ast::ObjectExpression> {
    let is_create_class = match ast.expr_data(call.callee(ast)) {
        ExprData::IdentifierReference(identifier) => {
            ast.get_utf8(identifier.name(ast)) == "createReactClass"
        }
        ExprData::MemberExpression(member) => {
            let ExprData::IdentifierReference(object) = ast.expr_data(member.object(ast)) else {
                return None;
            };
            let property = member.property(ast).as_identifier_name(ast)?;
            ast.get_utf8(object.name(ast)) == "React"
                && ast.get_utf8(property.name(ast)) == "createClass"
        }
        _ => false,
    };
    if !is_create_class || call.arguments(ast).len() != 1 {
        return None;
    }
    let argument_id = call.arguments(ast).iter().next()?;
    let argument = ast.get_node_in_sub_range(argument_id).as_expr(ast)?;
    argument.as_object_expression(ast)
}

fn has_display_name(object: swc_next_ecma_ast::ObjectExpression, ast: &Ast<'_>) -> bool {
    object.properties(ast).iter().any(|property_id| {
        let property = ast.get_node_in_sub_range(property_id);
        let Some(property) = property.as_object_property(ast) else {
            return false;
        };
        static_property_name(property.key(ast), ast).is_some_and(|name| name == "displayName")
    })
}

fn infer_name<'name, 'ast: 'name>(
    ctx: &TransformCtx<'_, 'ast>,
    source_path: Option<&'name str>,
) -> Option<&'name str> {
    let mut ancestors = ctx.ancestors();
    let mut child = ancestors.next()?;
    for ancestor in ancestors {
        let name = match ctx.ast.kind_data(ancestor) {
            NodeKindData::AssignmentExpression(assignment)
                if assignment.right(ctx.ast).node_id() == child =>
            {
                assignment_name(assignment.left(ctx.ast), ctx.ast)
            }
            NodeKindData::VariableDeclarator(declarator)
                if declarator
                    .init(ctx.ast)
                    .is_some_and(|init| init.node_id() == child) =>
            {
                declarator
                    .id(ctx.ast)
                    .as_binding_identifier(ctx.ast)
                    .map(|identifier| ctx.ast.get_utf8(identifier.name(ctx.ast)))
            }
            NodeKindData::ObjectProperty(property)
                if property.value(ctx.ast).node_id() == child =>
            {
                static_property_name(property.key(ctx.ast), ctx.ast)
            }
            NodeKindData::ExportDefaultDeclaration(declaration)
                if declaration.declaration(ctx.ast).node_id() == child =>
            {
                source_path.and_then(file_stem)
            }
            _ => None,
        };
        if name.is_some() {
            return name;
        }
        if ctx.ast.node_kind(ancestor).is_stmt() {
            return None;
        }
        child = ancestor;
    }
    None
}

fn assignment_name<'ast>(
    target: swc_next_ecma_ast::AssignmentTarget,
    ast: &Ast<'ast>,
) -> Option<&'ast str> {
    let target = target.as_simple_assignment_target(ast)?;
    match ast.simple_assignment_target_data(target) {
        SimpleAssignmentTargetData::IdentifierReference(identifier) => {
            Some(ast.get_utf8(identifier.name(ast)))
        }
        SimpleAssignmentTargetData::MemberExpression(member) => {
            static_property_name(member.property(ast), ast)
        }
        _ => None,
    }
}

fn static_property_name<'a>(key: PropertyKey, ast: &Ast<'a>) -> Option<&'a str> {
    if let Some(identifier) = key.as_identifier_name(ast) {
        return Some(ast.get_utf8(identifier.name(ast)));
    }
    key.as_expr(ast)
        .and_then(|expression| expression.as_string_literal(ast))
        .and_then(|literal| ast.get_wtf8(literal.value(ast)).as_str())
}

fn file_stem(path: &str) -> Option<&str> {
    let path = Path::new(path);
    let stem = path.file_stem()?.to_str()?;
    if stem == "index"
        && let Some(parent) = path.parent().and_then(Path::file_name)
        && let Some(name) = parent.to_str()
    {
        return Some(name);
    }
    Some(stem)
}
