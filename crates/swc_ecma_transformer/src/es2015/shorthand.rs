use swc_next_ecma_ast::{Ast, GetSpan, ObjectProperty, PropertyKey};

use crate::state::TransformerCtx;

pub(super) struct Shorthand;

impl Shorthand {
    pub fn exit_object_property(
        &mut self,
        property: ObjectProperty,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        let (is_shorthand, is_method) = {
            let ast = &*ctx.ast;
            (property.shorthand(ast), property.method(ast))
        };
        if !is_shorthand && !is_method {
            return;
        }

        if self.is_proto_key(property, ctx.ast) {
            let span = property.key(ctx.ast).span(ctx.ast);
            let ast = &mut *ctx.ast;
            let value = ast.add_wtf8("__proto__".into());
            let key = ast.property_key_string_literal(span, value, ast.empty_utf8());
            property.set_key(ast, key);
            property.set_computed(ast, true);
        }

        if is_shorthand {
            property.set_shorthand(ctx.ast, false);
        }
        if is_method {
            property.set_method(ctx.ast, false);
        }
    }

    fn is_proto_key(&self, property: ObjectProperty, ast: &Ast) -> bool {
        let key: PropertyKey = property.key(ast);
        if let Some(identifier) = key.as_identifier_name(ast) {
            return ast.get_utf8(identifier.name(ast)) == "__proto__";
        }
        if let Some(string) = key.as_string_literal(ast) {
            return ast.get_wtf8(string.value(ast)).as_bytes() == b"__proto__";
        }
        false
    }
}
