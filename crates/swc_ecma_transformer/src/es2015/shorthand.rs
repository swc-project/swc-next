use swc_next_ecma_ast::{GetSpan, ObjectProperty, PropertyKey};

use crate::context::Mutation;

pub(super) struct Shorthand;

impl Shorthand {
    pub fn exit_object_property(
        &mut self,
        property: ObjectProperty,
        ctx: &mut Mutation<'_, '_, '_>,
    ) {
        if !property.shorthand(ctx.ast()) && !property.method(ctx.ast()) {
            return;
        }
        let key = property.key(ctx.ast());
        let proto = key
            .as_identifier_name(ctx.ast())
            .is_some_and(|key| ctx.ast().get_utf8(key.name(ctx.ast())) == "__proto__")
            || key
                .as_expr(ctx.ast())
                .and_then(|key| key.as_string_literal(ctx.ast()))
                .is_some_and(|key| {
                    ctx.ast().get_wtf8(key.value(ctx.ast())).as_bytes() == b"__proto__"
                });
        if proto {
            let span = key.span(ctx.ast());
            let value = ctx.intern_wtf8("__proto__");
            let key = ctx.add_node(ctx.current_scope(), |ast| {
                ast.string_literal(span, value, ast.empty_utf8())
            });
            ctx.set_property_key(
                property.node_id(),
                PropertyKey::Expr(swc_next_ecma_ast::Expr::StringLiteral(key)),
            );
            ctx.set_property_computed(property.node_id(), true);
        }
        ctx.set_property_shorthand(property, false);
        ctx.set_property_method(property, false);
    }
}
