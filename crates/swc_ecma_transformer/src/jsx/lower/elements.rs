use super::super::{JsxRuntime, text};
use super::{JsxLowerer, LoweringContext, build::*, runtime::RuntimeHelper};
use swc_next_ecma_ast::GetSpan;
use swc_next_ecma_ast::{
    Argument, Ast, CommentKind, CommentPosition, DUMMY_SP, Diagnostic, Expr, ExprData,
    JsxAttributeItem, JsxAttributeItemData, JsxAttributeNameData, JsxAttributeValueData, JsxChild,
    JsxChildData, JsxElement, JsxElementName, JsxElementNameData, JsxExpressionData, JsxFragment,
    JsxMemberExpression, JsxMemberExpressionObjectData, ObjectPropertyKind, PropertyKey,
    PropertyKind, Span, SubRange,
};

impl<'ast> JsxLowerer<'_, 'ast> {
    pub(super) fn lower_expression(
        &mut self,
        expression: Expr,
        ast: &mut Ast<'ast>,
    ) -> Option<Expr> {
        match ast.expr_data(expression) {
            ExprData::JsxElement(element) => Some(self.lower_element(element, ast)),
            ExprData::JsxFragment(fragment) => Some(self.lower_fragment(fragment, ast)),
            _ => None,
        }
    }

    fn lower_element(&mut self, element: JsxElement, ast: &mut Ast<'ast>) -> Expr {
        let span = element.span(ast);
        let can_add_self = self
            .self_context
            .as_ref()
            .is_some_and(|context| context.can_add_self(element.node_id()));
        let opening = element.opening_element(ast);
        let name = self.lower_element_name(opening.name(ast), ast);
        let attributes = opening.attributes(ast);
        let children = element.children(ast);
        let automatic_fallback = self.options.runtime == JsxRuntime::Automatic
            && self.has_key_after_spread(attributes, ast);
        let context = LoweringContext {
            opening_end: opening.span(ast).end,
            automatic_fallback,
            can_add_self,
        };
        self.lower_jsx(span, Some(name), attributes, children, context, ast)
    }

    fn lower_fragment(&mut self, fragment: JsxFragment, ast: &mut Ast<'ast>) -> Expr {
        let span = fragment.span(ast);
        self.lower_jsx(
            span,
            None,
            SubRange::empty(),
            fragment.children(ast),
            LoweringContext {
                opening_end: span.start,
                automatic_fallback: false,
                can_add_self: false,
            },
            ast,
        )
    }

    fn lower_jsx(
        &mut self,
        span: Span,
        name: Option<Expr>,
        attributes: SubRange<swc_next_ecma_ast::JsxAttributeItem>,
        children: SubRange<JsxChild>,
        context: LoweringContext,
        ast: &mut Ast<'ast>,
    ) -> Expr {
        let classic = self.options.runtime == JsxRuntime::Classic || context.automatic_fallback;
        let mut arguments = Vec::with_capacity(children.len() + 2);

        let mut properties = Vec::with_capacity(attributes.len() + 3);
        let mut key = None;
        let source_self = self.options.development && self.options.source_self;
        let inject_self = name.is_some()
            && source_self
            && (!classic || !has_attribute(attributes, "__self", ast));
        let inject_source = name.is_some()
            && source_self
            && (!classic || !has_attribute(attributes, "__source", ast));
        let direct_spread = classic && attributes.len() == 1 && !(inject_self || inject_source);
        let mut direct_props = None;
        let mut flattened_props_end = None;
        for attribute_id in attributes.iter() {
            let attribute = ast.get_node_in_sub_range(attribute_id);
            match ast.jsx_attribute_item_data(attribute) {
                JsxAttributeItemData::JsxAttribute(attribute) => {
                    let is_key = self.attribute_is_key(attribute.name(ast), ast);
                    // Babel 8 rejects shorthand `key` in both classic and automatic runtimes.
                    if is_key && attribute.value(ast).is_none() {
                        self.diagnostics.push(Diagnostic::error(
                            "Please provide an explicit key value. Using `key` as shorthand for `key={true}` is not allowed.",
                            attribute.name(ast).span(ast),
                        ));
                    }
                    if !classic && is_key {
                        key = Some(self.lower_attribute_value(attribute.value(ast), ast));
                        continue;
                    }
                    let property_span = attribute.span(ast);
                    let property_key = self.lower_attribute_name(attribute.name(ast), ast);
                    let value = self.lower_attribute_value(attribute.value(ast), ast);
                    let shorthand = is_shorthand_property(property_key, value, ast);
                    properties.push(ObjectPropertyKind::ObjectProperty(ast.object_property(
                        property_span,
                        property_key,
                        value,
                        PropertyKind::Init,
                        false,
                        shorthand,
                        false,
                    )));
                }
                JsxAttributeItemData::JsxSpreadAttribute(spread) => {
                    let argument = spread.argument(ast);
                    if direct_spread && !has_proto_property(argument, ast) {
                        direct_props = Some(argument);
                    } else if let ExprData::ObjectExpression(object) = ast.expr_data(argument)
                        && !has_proto_property(argument, ast)
                        && !has_accessor_property(object, ast)
                    {
                        if attributes.len() == 1 {
                            flattened_props_end = Some(argument.span(ast).end);
                        }
                        for property_id in object.properties(ast).iter() {
                            properties.push(ast.get_node_in_sub_range(property_id));
                        }
                    } else {
                        properties.push(ObjectPropertyKind::SpreadElement(
                            ast.spread_element(spread.span(ast), argument),
                        ));
                    }
                }
            }
        }
        // Automatic runtime includes children in the props object.
        let properties_end = if classic {
            context.opening_end
        } else {
            span.end
        };
        let attributes_span = Span::new(span.start, properties_end);
        let mut properties_span = properties.first().map_or(attributes_span, |property| {
            let property_start = property.span(ast).start;
            let start = if ast.comments().is_empty() {
                property_start
            } else {
                leading_property_comment_start(ast, span.start, property_start)
            };
            Span::new(start, properties_end)
        });
        if let Some(end) = flattened_props_end {
            properties_span.end = end;
        }

        if classic {
            if inject_self && context.can_add_self {
                let value = ast.expr_this_expression(DUMMY_SP);
                properties.push(object_property(ast, "__self", value, DUMMY_SP));
            }
            if inject_source {
                let source = self.source_object(span.start, ast);
                properties.push(object_property(ast, "__source", source, DUMMY_SP));
            }
        }

        if classic {
            let props = if let Some(props) = direct_props {
                props
            } else if properties.is_empty() {
                ast.expr_null_literal(DUMMY_SP)
            } else {
                let properties = ast.add_sub_range(properties);
                ast.expr_object_expression(properties_span, properties)
            };
            arguments.push(Argument::Expr(props));
            for child_id in children.iter() {
                let child = ast.get_node_in_sub_range(child_id);
                if let Some(argument) = self.lower_child(child, ast) {
                    arguments.push(argument);
                }
            }
            let callee = if context.automatic_fallback {
                self.create_element_helper(ast)
            } else {
                self.pragma.create_expression(ast, self.pragma_namespace)
            };
            let name = name.unwrap_or_else(|| {
                self.pragma_frag
                    .create_expression(ast, self.pragma_frag_namespace)
            });
            arguments.insert(0, Argument::Expr(name));
            return call(ast, span, callee, arguments, self.pure);
        }

        let mut lowered_children = Vec::with_capacity(children.len());
        for child_id in children.iter() {
            let child = ast.get_node_in_sub_range(child_id);
            if let Some(child) = self.lower_child(child, ast) {
                lowered_children.push(Some(child));
            }
        }
        let child_count = lowered_children.len();
        if child_count != 0 {
            let single_expression = lowered_children
                .first()
                .and_then(|child| *child)
                .and_then(|child| child.as_expr(ast));
            let value = match (child_count, single_expression) {
                (1, Some(expression)) => expression,
                _ => {
                    let elements = ast.add_sub_range(lowered_children);
                    ast.expr_array_expression(span, elements)
                }
            };
            properties.push(object_property(ast, "children", value, DUMMY_SP));
        }
        let properties = ast.add_sub_range(properties);
        let props = ast.expr_object_expression(properties_span, properties);
        arguments.push(Argument::Expr(props));
        if self.options.development {
            let key = key.unwrap_or_else(|| void_zero(ast));
            arguments.push(Argument::Expr(key));
            arguments.push(Argument::Expr(
                ast.expr_boolean_literal(DUMMY_SP, child_count > 1),
            ));
            if inject_source {
                let source = self.source_object(span.start, ast);
                arguments.push(Argument::Expr(source));
            }
            if inject_self && context.can_add_self {
                if !inject_source {
                    arguments.push(Argument::Expr(void_zero(ast)));
                }
                arguments.push(Argument::Expr(ast.expr_this_expression(DUMMY_SP)));
            }
        } else if let Some(key) = key {
            arguments.push(Argument::Expr(key));
        }
        let helper = if self.options.development {
            RuntimeHelper::JsxDev
        } else if child_count > 1 {
            RuntimeHelper::Jsxs
        } else {
            RuntimeHelper::Jsx
        };
        let name = name.unwrap_or_else(|| self.runtime_helper(RuntimeHelper::Fragment, ast));
        let callee = self.runtime_helper(helper, ast);
        arguments.insert(0, Argument::Expr(name));
        call(ast, span, callee, arguments, self.pure)
    }

    fn lower_element_name(&mut self, name: JsxElementName, ast: &mut Ast<'ast>) -> Expr {
        match ast.jsx_element_name_data(name) {
            JsxElementNameData::JsxIdentifier(identifier) => {
                let text = ast.get_utf8(identifier.name(ast));
                if text == "this" {
                    ast.expr_this_expression(identifier.span(ast))
                } else if text.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
                    || text.contains('-')
                {
                    string_expression(ast, identifier.span(ast), text)
                } else {
                    let name = identifier.name(ast);
                    ast.expr_identifier_reference(identifier.span(ast), name)
                }
            }
            JsxElementNameData::JsxMemberExpression(member) => {
                self.lower_member_expression(member, ast)
            }
            JsxElementNameData::JsxNamespacedName(name) => {
                self.check_namespace(name.span(ast));
                let namespace = ast.get_utf8(name.namespace(ast).name(ast));
                let local = ast.get_utf8(name.name(ast).name(ast));
                let value = format!("{namespace}:{local}");
                string_expression(ast, name.span(ast), &value)
            }
        }
    }

    fn lower_member_expression(
        &mut self,
        member: JsxMemberExpression,
        ast: &mut Ast<'ast>,
    ) -> Expr {
        let object = match ast.jsx_member_expression_object_data(member.object(ast)) {
            JsxMemberExpressionObjectData::JsxIdentifier(identifier) => {
                let name = ast.get_utf8(identifier.name(ast));
                if name == "this" {
                    ast.expr_this_expression(identifier.span(ast))
                } else {
                    let name = identifier.name(ast);
                    ast.expr_identifier_reference(identifier.span(ast), name)
                }
            }
            JsxMemberExpressionObjectData::JsxMemberExpression(object) => {
                self.lower_member_expression(object, ast)
            }
        };
        let property = member.property(ast);
        let property_name = property.name(ast);
        let property = ast.identifier_name(property.span(ast), property_name);
        ast.expr_member_expression(
            member.span(ast),
            object,
            PropertyKey::IdentifierName(property),
            false,
            false,
        )
    }

    fn lower_attribute_name(
        &mut self,
        name: swc_next_ecma_ast::JsxAttributeName,
        ast: &mut Ast<'ast>,
    ) -> PropertyKey {
        match ast.jsx_attribute_name_data(name) {
            JsxAttributeNameData::JsxIdentifier(identifier) => {
                let text = ast.get_utf8(identifier.name(ast));
                if text.contains('-') {
                    string_property(ast, identifier.span(ast), text)
                } else {
                    let name = identifier.name(ast);
                    PropertyKey::IdentifierName(ast.identifier_name(identifier.span(ast), name))
                }
            }
            JsxAttributeNameData::JsxNamespacedName(name) => {
                self.check_namespace(name.span(ast));
                let namespace = ast.get_utf8(name.namespace(ast).name(ast));
                let local = ast.get_utf8(name.name(ast).name(ast));
                let value = format!("{namespace}:{local}");
                string_property(ast, name.span(ast), &value)
            }
        }
    }

    fn check_namespace(&mut self, span: Span) {
        if self.options.throw_if_namespace {
            self.diagnostics.push(Diagnostic::error(
                "Namespace tags are not supported by default because React's JSX does not support them. Set `throwIfNamespace: false` to allow this tag.",
                span,
            ));
        }
    }

    fn lower_attribute_value(
        &mut self,
        value: Option<swc_next_ecma_ast::JsxAttributeValue>,
        ast: &mut Ast<'ast>,
    ) -> Expr {
        let Some(value) = value else {
            return ast.expr_boolean_literal(DUMMY_SP, true);
        };
        match ast.jsx_attribute_value_data(value) {
            JsxAttributeValueData::StringLiteral(literal) => {
                let value = ast.get_wtf8(literal.value(ast)).to_string_lossy();
                if let std::borrow::Cow::Borrowed(text) = &value
                    && !text.contains('&')
                {
                    return ast.expr_string_literal(
                        literal.span(ast),
                        literal.value(ast),
                        swc_next_ecma_ast::AstUtf8::EMPTY,
                    );
                }
                let decoded = text::decode_entities(&value);
                string_expression(ast, literal.span(ast), &decoded)
            }
            JsxAttributeValueData::JsxExpressionContainer(container) => container
                .expression(ast)
                .as_expr(ast)
                .unwrap_or_else(|| ast.expr_boolean_literal(container.span(ast), true)),
            JsxAttributeValueData::JsxElement(element) => self.lower_element(element, ast),
            JsxAttributeValueData::JsxFragment(fragment) => self.lower_fragment(fragment, ast),
        }
    }

    fn lower_child(&mut self, child: JsxChild, ast: &mut Ast<'ast>) -> Option<Argument> {
        let expression = match ast.jsx_child_data(child) {
            JsxChildData::JsxText(text_node) => {
                let value = ast.get_utf8(text_node.value(ast));
                if !value.contains(['\r', '\n', '&']) {
                    return Some(Argument::Expr(ast.expr_string_literal(
                        text_node.span(ast),
                        text_node.value(ast).into(),
                        swc_next_ecma_ast::AstUtf8::EMPTY,
                    )));
                }
                let value = text::normalize_jsx_text(value)?;
                string_expression(ast, text_node.span(ast), &value)
            }
            JsxChildData::JsxExpressionContainer(container) => {
                match ast.jsx_expression_data(container.expression(ast)) {
                    JsxExpressionData::Expr(expression) => expression,
                    JsxExpressionData::JsxEmptyExpression(_) => return None,
                }
            }
            JsxChildData::JsxSpreadChild(spread) => {
                self.diagnostics.push(Diagnostic::error(
                    "Spread children are not supported in React.",
                    spread.span(ast),
                ));
                return None;
            }
            JsxChildData::JsxElement(element) => self.lower_element(element, ast),
            JsxChildData::JsxFragment(fragment) => self.lower_fragment(fragment, ast),
        };
        Some(Argument::Expr(expression))
    }

    fn attribute_is_key(&self, name: swc_next_ecma_ast::JsxAttributeName, ast: &Ast<'ast>) -> bool {
        name.as_jsx_identifier(ast)
            .is_some_and(|identifier| ast.get_utf8(identifier.name(ast)) == "key")
    }

    fn has_key_after_spread(
        &self,
        attributes: SubRange<swc_next_ecma_ast::JsxAttributeItem>,
        ast: &Ast<'ast>,
    ) -> bool {
        let mut saw_spread = false;
        for attribute_id in attributes.iter() {
            let attribute = ast.get_node_in_sub_range(attribute_id);
            match ast.jsx_attribute_item_data(attribute) {
                JsxAttributeItemData::JsxSpreadAttribute(_) => saw_spread = true,
                JsxAttributeItemData::JsxAttribute(attribute)
                    if saw_spread && self.attribute_is_key(attribute.name(ast), ast) =>
                {
                    return true;
                }
                JsxAttributeItemData::JsxAttribute(_) => {}
            }
        }
        false
    }
}

fn has_attribute(attributes: SubRange<JsxAttributeItem>, name: &str, ast: &Ast<'_>) -> bool {
    attributes.iter().any(|id| {
        ast.get_node_in_sub_range(id)
            .as_jsx_attribute(ast)
            .and_then(|attribute| attribute.name(ast).as_jsx_identifier(ast))
            .is_some_and(|identifier| ast.get_utf8(identifier.name(ast)) == name)
    })
}

// Keep optional comment searches out of the JSX call-building frame.
#[inline(never)]
fn leading_property_comment_start(ast: &Ast<'_>, element_start: u32, property_start: u32) -> u32 {
    let comments = ast.comments();
    let first = comments.partition_point(|comment| comment.span.start < element_start);
    comments[first..]
        .iter()
        .take_while(|comment| comment.span.start <= property_start)
        .find(|comment| {
            (comment.kind == CommentKind::Line
                || (comment.position == CommentPosition::Leading
                    && comment.attached_to == property_start))
                && comment.span.end <= property_start
        })
        .map_or(property_start, |comment| comment.span.start)
}
