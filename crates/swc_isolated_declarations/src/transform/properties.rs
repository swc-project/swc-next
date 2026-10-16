//! Validate, spell, and normalize property keys for declaration output.

use std::borrow::Cow;

use swc_next_ecma_ast::{
    Ast, Diagnostic, Expr, ExprData, GetSpan, PropertyKey, PropertyKeyData, SubRange,
    TsTypeElement, TsTypeElementData, is_identifier,
};
use zmij_ecma::Buffer;

use super::Transformer;
use super::diagnostics::COMPUTED_SIGNATURE_NOT_SUPPORTED;
use crate::surface::{
    first_template_quasi, signed_numeric_value, supported_computed_key, unparenthesized,
};

impl<'ast> Transformer<'_, 'ast> {
    pub(super) fn is_supported_computed_key(&self, key: PropertyKey, ast: &Ast<'ast>) -> bool {
        supported_computed_key(key, ast, self.semantic)
    }

    pub(super) fn property_key_is_private(key: PropertyKey, ast: &Ast<'ast>) -> bool {
        matches!(
            ast.property_key_data(key),
            PropertyKeyData::Expr(expression)
                if matches!(ast.expr_data(expression), ExprData::PrivateIdentifier(_))
        )
    }

    pub(super) fn validate_computed_type_elements(
        &mut self,
        elements: SubRange<TsTypeElement>,
        ast: &Ast<'ast>,
    ) {
        for element in elements.iter() {
            let element = ast.get_node_in_sub_range(element);
            let key = match ast.ts_type_element_data(element) {
                TsTypeElementData::TsPropertySignature(signature) if signature.computed(ast) => {
                    Some(signature.key(ast))
                }
                TsTypeElementData::TsMethodSignature(signature) if signature.computed(ast) => {
                    Some(signature.key(ast))
                }
                _ => None,
            };
            if let Some(key) = key
                && !matches!(
                    ast.property_key_data(key),
                    PropertyKeyData::Expr(expression)
                        if matches!(ast.expr_data(expression), ExprData::ChainExpression(_))
                )
                && !self.is_supported_computed_key(key, ast)
            {
                self.diagnostics.push(Diagnostic::error(
                    COMPUTED_SIGNATURE_NOT_SUPPORTED,
                    key.span(ast),
                ));
            }
        }
    }

    pub(super) fn property_key_name(
        &self,
        key: PropertyKey,
        ast: &Ast<'ast>,
    ) -> Option<Cow<'ast, str>> {
        match ast.property_key_data(key) {
            PropertyKeyData::IdentifierName(identifier) => {
                Some(Cow::Borrowed(ast.get_utf8(identifier.name(ast))))
            }
            PropertyKeyData::Expr(expression) => self.property_expression_name(expression, ast),
        }
    }

    fn property_expression_name(
        &self,
        expression: Expr,
        ast: &Ast<'ast>,
    ) -> Option<Cow<'ast, str>> {
        match ast.expr_data(expression) {
            ExprData::IdentifierReference(identifier) => {
                Some(Cow::Borrowed(ast.get_utf8(identifier.name(ast))))
            }
            ExprData::StringLiteral(literal) => {
                Some(ast.get_wtf8(literal.value(ast)).to_string_lossy())
            }
            ExprData::NumericLiteral(literal) => {
                Some(Cow::Owned(self.number_property_name(literal.value(ast))))
            }
            ExprData::TemplateLiteral(template) if template.expressions(ast).is_empty() => {
                let quasi = ast.get_node_in_sub_range(template.quasis(ast).first()?);
                Some(ast.get_wtf8(quasi.cooked(ast)).to_string_lossy())
            }
            ExprData::MemberExpression(member) if !member.computed(ast) => {
                let object = self.property_expression_name(member.object(ast), ast)?;
                let property = self.property_key_name(member.property(ast), ast)?;
                Some(Cow::Owned(format!("{object}.{property}")))
            }
            ExprData::ParenthesizedExpression(parenthesized) => {
                self.property_expression_name(parenthesized.expression(ast), ast)
            }
            _ => None,
        }
    }

    fn number_property_name(&self, value: f64) -> String {
        let mut buffer = Buffer::new();
        buffer.format(value).to_owned()
    }

    pub(super) fn declaration_property_key(
        &self,
        key: PropertyKey,
        ast: &mut Ast<'ast>,
    ) -> PropertyKey {
        let PropertyKeyData::Expr(expression) = ast.property_key_data(key) else {
            return key;
        };
        let expression = unparenthesized(expression, ast);
        if let ExprData::UnaryExpression(unary) = ast.expr_data(expression)
            && signed_numeric_value(unary, ast) == Some(0.0)
        {
            // TypeScript treats [-0] and [0] as different accessor names. Emit
            // the same spelling so consumers preserve JavaScript key identity.
            return PropertyKey::Expr(ast.expr_numeric_literal(expression.span(ast), 0.0, None));
        }
        PropertyKey::Expr(expression)
    }

    pub(super) fn object_declaration_key(
        &self,
        key: PropertyKey,
        computed: bool,
        preserve_syntax: bool,
        ast: &mut Ast<'ast>,
    ) -> (PropertyKey, bool) {
        let key = self.declaration_property_key(key, ast);
        if preserve_syntax {
            if computed
                && let PropertyKeyData::Expr(expression) = ast.property_key_data(key)
                && let ExprData::TemplateLiteral(template) = ast.expr_data(expression)
                && template.expressions(ast).is_empty()
            {
                let quasi = first_template_quasi(template, ast);
                let expression = ast.expr_string_literal(
                    expression.span(ast),
                    quasi.cooked(ast),
                    ast.empty_utf8(),
                );
                return (PropertyKey::Expr(expression), true);
            }
            return (key, computed);
        }

        if !computed {
            let key = self.normalize_plain_property_key(key, ast);
            return (key, false);
        }

        let expression = match ast.property_key_data(key) {
            PropertyKeyData::IdentifierName(_) => return (key, computed),
            PropertyKeyData::Expr(expression) => expression,
        };
        match ast.expr_data(expression) {
            ExprData::StringLiteral(literal) => {
                let key = PropertyKey::Expr(Expr::StringLiteral(literal));
                return (self.normalize_plain_property_key(key, ast), false);
            }
            ExprData::NumericLiteral(literal) => {
                return (PropertyKey::Expr(Expr::NumericLiteral(literal)), false);
            }
            _ => {}
        }
        if let ExprData::TemplateLiteral(template) = ast.expr_data(expression)
            && template.expressions(ast).is_empty()
        {
            let quasi = first_template_quasi(template, ast);
            let expression =
                ast.expr_string_literal(expression.span(ast), quasi.cooked(ast), ast.empty_utf8());
            let key = PropertyKey::Expr(expression);
            return (self.normalize_plain_property_key(key, ast), false);
        }
        let ExprData::MemberExpression(member) = ast.expr_data(expression) else {
            return (key, true);
        };
        let ExprData::MemberExpression(symbol_member) = ast.expr_data(member.object(ast)) else {
            return (key, true);
        };
        if symbol_member.computed(ast)
            || symbol_member.optional(ast)
            || self
                .property_key_name(symbol_member.property(ast), ast)
                .as_deref()
                != Some("Symbol")
        {
            return (key, true);
        }
        let ExprData::IdentifierReference(global_this) = ast.expr_data(symbol_member.object(ast))
        else {
            return (key, true);
        };
        if ast.get_utf8(global_this.name(ast)) != "globalThis"
            || self
                .semantic
                .reference_of(global_this.node_id())
                .is_none_or(|reference| self.semantic.reference(reference).symbol.is_some())
        {
            return (key, true);
        }
        let symbol_name = ast.add_interned_utf8("Symbol");
        let symbol = ast.expr_identifier_reference(symbol_member.span(ast), symbol_name);
        let expression = ast.expr_member_expression(
            member.span(ast),
            symbol,
            member.property(ast),
            member.computed(ast),
            member.optional(ast),
        );
        (PropertyKey::Expr(expression), true)
    }

    fn normalize_plain_property_key(&self, key: PropertyKey, ast: &mut Ast<'ast>) -> PropertyKey {
        let PropertyKeyData::Expr(expression) = ast.property_key_data(key) else {
            return key;
        };
        let ExprData::StringLiteral(literal) = ast.expr_data(expression) else {
            return key;
        };
        let Some(value) = ast.get_wtf8(literal.value(ast)).as_str() else {
            return key;
        };
        if !is_identifier(value.as_bytes()) {
            return key;
        }
        let name = ast.add_interned_utf8(value);
        ast.property_key_identifier_name(literal.span(ast), name)
    }
}
