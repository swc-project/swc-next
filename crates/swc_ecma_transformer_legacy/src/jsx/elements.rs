use swc_next_ecma_ast::*;

use super::{Jsx, JsxRuntime, build::*, pragma, text};

impl Jsx<'_> {
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L610>.
    pub(super) fn jsx_frag_to_expr(&mut self, fragment: JsxFragment, ast: &mut Ast<'_>) -> Expr {
        let span = fragment.span(ast);
        self.add_pure_comment(span);
        match self.runtime {
            JsxRuntime::Automatic => {
                let binding = *self
                    .import_fragment
                    .get_or_insert_with(|| self.resolver.private_binding(ast, "_Fragment"));
                let name = self.resolver.reference(ast, binding);
                let mut properties = Vec::new();
                let mut children = self.children(fragment.children(ast), ast);
                let use_jsxs = match children.len() {
                    0 => false,
                    1 if children[0].is_expr(ast) => {
                        let child = children.pop().unwrap().as_expr(ast).unwrap();
                        properties.push(property(ast, "children", child));
                        false
                    }
                    _ => {
                        let children = array(ast, children);
                        properties.push(property(ast, "children", children));
                        true
                    }
                };
                let jsx = self.jsx_binding(use_jsxs, false, ast);
                let callee = self.resolver.reference(ast, jsx);
                let props = object(ast, properties);
                let mut args = vec![Argument::Expr(name), Argument::Expr(props)];
                if self.development {
                    args.push(Argument::Expr(undefined(ast)));
                    args.push(Argument::Expr(ast.expr_boolean_literal(DUMMY_SP, use_jsxs)));
                    args.push(Argument::Expr(self.source.expression(
                        ast,
                        fragment.opening_fragment(ast).span(ast),
                        &self.filename,
                    )));
                    let this = if self.self_ctx.can_add_self() {
                        ast.expr_this_expression(DUMMY_SP)
                    } else {
                        undefined(ast)
                    };
                    args.push(Argument::Expr(this));
                }
                call(ast, span, callee, args)
            }
            JsxRuntime::Classic => {
                let name = pragma::expression(ast, self.resolver, &self.pragma_frag);
                let props = ast.expr_null_literal(DUMMY_SP);
                let mut args = vec![Argument::Expr(name), Argument::Expr(props)];
                args.extend(self.children(fragment.children(ast), ast));
                let callee = pragma::expression(ast, self.resolver, &self.pragma);
                call(ast, span, callee, args)
            }
            JsxRuntime::Preserve => unreachable!(),
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L734>.
    pub(super) fn jsx_elem_to_expr(&mut self, element: JsxElement, ast: &mut Ast<'_>) -> Expr {
        let top_level_node = self.top_level_node;
        let span = element.span(ast);
        let opening = element.opening_element(ast);
        let attributes = opening
            .attributes(ast)
            .iter()
            .map(|id| ast.get_node_in_sub_range(id))
            .collect::<Vec<_>>();
        let use_create_element = should_use_create_element(&attributes, ast);
        self.top_level_node = false;
        let name = self.jsx_name(opening.name(ast), ast);
        self.add_pure_comment(span);
        match self.runtime {
            JsxRuntime::Automatic => {
                let mut properties = Vec::new();
                let mut key = None;
                let mut source = None;
                let mut this = None;
                for attribute in attributes {
                    match ast.jsx_attribute_item_data(attribute) {
                        JsxAttributeItemData::JsxAttribute(attribute) => {
                            match ast.jsx_attribute_name_data(attribute.name(ast)) {
                                JsxAttributeNameData::JsxIdentifier(identifier) => {
                                    let attr_name = ast.get_utf8(identifier.name(ast));
                                    if !use_create_element && attr_name == "key" {
                                        key = attribute
                                            .value(ast)
                                            .and_then(|value| self.attr_value_to_expr(value, ast));
                                        if key.is_none() {
                                            self.diagnostics.push(Diagnostic::error(
                                                "The value of property 'key' should not be empty",
                                                identifier.span(ast),
                                            ));
                                        }
                                        continue;
                                    }
                                    if !use_create_element
                                        && attr_name == "__source"
                                        && self.development
                                    {
                                        assert!(source.is_none(), "Duplicate __source is found");
                                        source = attribute
                                            .value(ast)
                                            .and_then(|value| self.attr_value_to_expr(value, ast));
                                        assert!(
                                            source.is_some(),
                                            "value of property '__source' should not be empty"
                                        );
                                        continue;
                                    }
                                    if !use_create_element
                                        && attr_name == "__self"
                                        && self.development
                                    {
                                        assert!(this.is_none(), "Duplicate __self is found");
                                        this = attribute
                                            .value(ast)
                                            .and_then(|value| self.attr_value_to_expr(value, ast));
                                        assert!(
                                            this.is_some(),
                                            "value of property '__self' should not be empty"
                                        );
                                        continue;
                                    }
                                    let value = self.process_attr_value(attribute.value(ast), ast);
                                    let key = self.attribute_name(attribute.name(ast), ast);
                                    properties.push(key_value(ast, key, value));
                                }
                                JsxAttributeNameData::JsxNamespacedName(name) => {
                                    self.namespace_diagnostic(span);
                                    let value = self.process_attr_value(attribute.value(ast), ast);
                                    let name = format!(
                                        "{}:{}",
                                        ast.get_utf8(name.namespace(ast).name(ast)),
                                        ast.get_utf8(name.name(ast).name(ast))
                                    );
                                    let key = PropertyKey::Expr(string(ast, span, &name));
                                    properties.push(key_value(ast, key, value));
                                }
                            }
                        }
                        JsxAttributeItemData::JsxSpreadAttribute(spread) => {
                            self.spread_properties(spread, &mut properties, ast)
                        }
                    }
                }
                let mut children = self.children(element.children(ast), ast);
                let use_jsxs = match children.len() {
                    0 => false,
                    1 if children[0].is_expr(ast) => {
                        if !use_create_element {
                            let child = children.pop().unwrap().as_expr(ast).unwrap();
                            properties.push(property(ast, "children", child));
                        }
                        false
                    }
                    _ => {
                        let children = array(ast, std::mem::take(&mut children));
                        properties.push(property(ast, "children", children));
                        true
                    }
                };
                let binding = self.jsx_binding(use_jsxs, use_create_element, ast);
                self.top_level_node = top_level_node;
                let callee = self.resolver.reference(ast, binding);
                let props = object(ast, properties);
                let mut args = vec![Argument::Expr(name), Argument::Expr(props)];
                if use_create_element {
                    args.extend(children);
                } else if self.development {
                    args.push(Argument::Expr(key.unwrap_or_else(|| undefined(ast))));
                    args.push(Argument::Expr(ast.expr_boolean_literal(DUMMY_SP, use_jsxs)));
                    args.push(Argument::Expr(source.unwrap_or_else(|| undefined(ast))));
                    args.push(Argument::Expr(this.unwrap_or_else(|| undefined(ast))));
                } else if let Some(key) = key {
                    args.push(Argument::Expr(key));
                }
                call(ast, span, callee, args)
            }
            JsxRuntime::Classic => {
                let props = self.fold_attrs_for_classic(attributes, ast);
                let mut args = vec![Argument::Expr(name), Argument::Expr(props)];
                args.extend(self.children(element.children(ast), ast));
                let callee = pragma::expression(ast, self.resolver, &self.pragma);
                call(ast, span, callee, args)
            }
            JsxRuntime::Preserve => unreachable!(),
        }
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1026>.
    fn children(&mut self, children: SubRange<JsxChild>, ast: &mut Ast<'_>) -> Vec<Argument> {
        let children = children
            .iter()
            .map(|id| ast.get_node_in_sub_range(id))
            .collect::<Vec<_>>();
        children
            .into_iter()
            .filter_map(|child| self.jsx_elem_child_to_expr(child, ast))
            .collect()
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1026>.
    fn jsx_elem_child_to_expr(&mut self, child: JsxChild, ast: &mut Ast<'_>) -> Option<Argument> {
        self.top_level_node = false;
        Some(match ast.jsx_child_data(child) {
            JsxChildData::JsxText(node) => {
                let raw = ast.get_utf8(node.value(ast));
                let decoded = text::decode_entities(raw);
                let value = text::jsx_text_to_str_with_raw(&decoded, raw);
                if value.is_empty() {
                    return None;
                }
                let value = ast.add_wtf8(&value);
                Argument::Expr(ast.expr_string_literal(node.span(ast), value, AstUtf8::EMPTY))
            }
            JsxChildData::JsxExpressionContainer(container) => {
                match ast.jsx_expression_data(container.expression(ast)) {
                    JsxExpressionData::Expr(expression) => Argument::Expr(expression),
                    JsxExpressionData::JsxEmptyExpression(_) => return None,
                }
            }
            JsxChildData::JsxElement(element) => {
                Argument::Expr(self.jsx_elem_to_expr(element, ast))
            }
            JsxChildData::JsxFragment(fragment) => {
                Argument::Expr(self.jsx_frag_to_expr(fragment, ast))
            }
            JsxChildData::JsxSpreadChild(spread) => Argument::SpreadElement(
                ast.spread_element(spread.span(ast), spread.expression(ast)),
            ),
        })
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1064>.
    fn fold_attrs_for_classic(
        &mut self,
        attributes: Vec<JsxAttributeItem>,
        ast: &mut Ast<'_>,
    ) -> Expr {
        if attributes.is_empty() {
            return ast.expr_null_literal(DUMMY_SP);
        }
        let count = attributes.len();
        let mut properties = Vec::new();
        for attribute in attributes {
            match ast.jsx_attribute_item_data(attribute) {
                JsxAttributeItemData::JsxAttribute(attribute) => {
                    properties.push(self.attr_to_prop(attribute, ast));
                }
                JsxAttributeItemData::JsxSpreadAttribute(spread) => {
                    if count == 1 {
                        return spread.argument(ast);
                    }
                    self.spread_properties(spread, &mut properties, ast);
                }
            }
        }
        object(ast, properties)
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1099>.
    fn attr_to_prop(&mut self, attribute: JsxAttribute, ast: &mut Ast<'_>) -> ObjectPropertyKind {
        let key = self.attribute_name(attribute.name(ast), ast);
        let value = if let Some(value) = attribute.value(ast) {
            self.process_attr_value(Some(value), ast)
        } else {
            ast.expr_boolean_literal(key.span(ast), true)
        };
        key_value(ast, key, value)
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L881>.
    fn spread_properties(
        &mut self,
        spread: JsxSpreadAttribute,
        properties: &mut Vec<ObjectPropertyKind>,
        ast: &mut Ast<'_>,
    ) {
        let expression = spread.argument(ast);
        if let Some(object) = expression.as_object_expression(ast) {
            properties.extend(
                object
                    .properties(ast)
                    .iter()
                    .map(|id| ast.get_node_in_sub_range(id)),
            );
        } else {
            properties.push(ObjectPropertyKind::SpreadElement(
                ast.spread_element(spread.span(ast), expression),
            ));
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L539>.
    fn process_attr_value(&mut self, value: Option<JsxAttributeValue>, ast: &mut Ast<'_>) -> Expr {
        match value.map(|value| ast.jsx_attribute_value_data(value)) {
            Some(JsxAttributeValueData::JsxElement(element)) => self.jsx_elem_to_expr(element, ast),
            Some(JsxAttributeValueData::JsxFragment(fragment)) => {
                self.jsx_frag_to_expr(fragment, ast)
            }
            Some(_) => self
                .attr_value_to_expr(value.unwrap(), ast)
                .expect("empty expression container"),
            None => ast.expr_boolean_literal(DUMMY_SP, true),
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1967>.
    fn attr_value_to_expr(&mut self, value: JsxAttributeValue, ast: &mut Ast<'_>) -> Option<Expr> {
        match ast.jsx_attribute_value_data(value) {
            JsxAttributeValueData::StringLiteral(literal) => {
                // Next stores quoted JSX text without decoding its entities.
                let raw = ast.get_wtf8(literal.value(ast)).as_str().unwrap();
                let value = text::decode_attribute(raw);
                let value = ast.add_wtf8(&value);
                Some(ast.expr_string_literal(literal.span(ast), value, AstUtf8::EMPTY))
            }
            JsxAttributeValueData::JsxExpressionContainer(container) => {
                match ast.jsx_expression_data(container.expression(ast)) {
                    JsxExpressionData::Expr(expression) => Some(expression),
                    JsxExpressionData::JsxEmptyExpression(_) => None,
                }
            }
            JsxAttributeValueData::JsxElement(element) => Some(Expr::JsxElement(element)),
            JsxAttributeValueData::JsxFragment(fragment) => Some(Expr::JsxFragment(fragment)),
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1541>.
    fn attribute_name(&mut self, name: JsxAttributeName, ast: &mut Ast<'_>) -> PropertyKey {
        match ast.jsx_attribute_name_data(name) {
            JsxAttributeNameData::JsxIdentifier(identifier) => {
                let name = identifier.name(ast);
                if ast.get_utf8(name).contains('-') {
                    PropertyKey::Expr(ast.expr_string_literal(
                        identifier.span(ast),
                        name.into(),
                        AstUtf8::EMPTY,
                    ))
                } else {
                    PropertyKey::IdentifierName(ast.identifier_name(identifier.span(ast), name))
                }
            }
            JsxAttributeNameData::JsxNamespacedName(name) => {
                let value = format!(
                    "{}:{}",
                    ast.get_utf8(name.namespace(ast).name(ast)),
                    ast.get_utf8(name.name(ast).name(ast))
                );
                PropertyKey::Expr(string(ast, name.span(ast), &value))
            }
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1459>.
    fn jsx_name(&mut self, name: JsxElementName, ast: &mut Ast<'_>) -> Expr {
        match ast.jsx_element_name_data(name) {
            JsxElementNameData::JsxIdentifier(identifier) => {
                let name = ast.get_utf8(identifier.name(ast));
                if name == "this" {
                    ast.expr_this_expression(identifier.span(ast))
                } else if name.starts_with(|character: char| character.is_ascii_lowercase()) {
                    string(ast, identifier.span(ast), name)
                } else {
                    self.resolver.source_reference(ast, identifier)
                }
            }
            JsxElementNameData::JsxNamespacedName(name) => {
                self.namespace_diagnostic(name.span(ast));
                let value = format!(
                    "{}:{}",
                    ast.get_utf8(name.namespace(ast).name(ast)),
                    ast.get_utf8(name.name(ast).name(ast))
                );
                string(ast, name.span(ast), &value)
            }
            JsxElementNameData::JsxMemberExpression(member) => self.jsx_member(member, ast),
        }
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1506>.
    fn jsx_member(&mut self, node: JsxMemberExpression, ast: &mut Ast<'_>) -> Expr {
        let object = match ast.jsx_member_expression_object_data(node.object(ast)) {
            JsxMemberExpressionObjectData::JsxIdentifier(identifier) => {
                if ast.get_utf8(identifier.name(ast)) == "this" {
                    ast.expr_this_expression(identifier.span(ast))
                } else {
                    self.resolver.source_reference(ast, identifier)
                }
            }
            JsxMemberExpressionObjectData::JsxMemberExpression(member) => {
                self.jsx_member(member, ast)
            }
        };
        let name = node.property(ast).name(ast);
        let property = ast.identifier_name(node.property(ast).span(ast), name);
        ast.expr_member_expression(
            node.span(ast),
            object,
            PropertyKey::IdentifierName(property),
            false,
            false,
        )
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L1483>.
    fn namespace_diagnostic(&mut self, span: Span) {
        if self.throw_if_namespace {
            self.diagnostics.push(Diagnostic::error("JSX Namespace is disabled by default because react does not support it yet. You can specify jsc.transform.react.throwIfNamespace to false to override default behavior", span));
        }
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/mod.rs#L936>.
    fn jsx_binding(
        &mut self,
        use_jsxs: bool,
        use_create_element: bool,
        ast: &mut Ast<'_>,
    ) -> BindingIdentifier {
        let (binding, name) = if use_create_element {
            (&mut self.import_create_element, "_createElement")
        } else if use_jsxs && !self.development {
            (&mut self.import_jsxs, "_jsxs")
        } else {
            (
                &mut self.import_jsx,
                if self.development { "_jsxDEV" } else { "_jsx" },
            )
        };
        *binding.get_or_insert_with(|| self.resolver.private_binding(ast, name))
    }
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/jsx/static_check.rs#L8>.
fn should_use_create_element(attributes: &[JsxAttributeItem], ast: &Ast<'_>) -> bool {
    let mut seen_spread = false;
    for &attribute in attributes {
        match ast.jsx_attribute_item_data(attribute) {
            JsxAttributeItemData::JsxAttribute(attribute) if seen_spread => {
                if let Some(identifier) = attribute.name(ast).as_jsx_identifier(ast)
                    && ast.get_utf8(identifier.name(ast)) == "key"
                {
                    return true;
                }
            }
            JsxAttributeItemData::JsxSpreadAttribute(_) => seen_spread = true,
            _ => {}
        }
    }
    false
}
