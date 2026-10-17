//! JSX names, attributes and brace containers retain their actual source owners.

mod children;

use std::borrow::Cow;

use swc_next_ecma_ast::{
    Comment, CommentKind, Expr, ExprData, GetSpan, JsxAttribute, JsxAttributeItemData,
    JsxAttributeName, JsxAttributeNameData, JsxAttributeValueData, JsxClosingElement, JsxElement,
    JsxElementName, JsxElementNameData, JsxExpressionContainer, JsxExpressionData, JsxFragment,
    JsxMemberExpression, JsxMemberExpressionObjectData, JsxNamespacedName, JsxOpeningElement, Span,
    StringLiteral,
};

use super::arrows::ArrowLayout;
use super::binary::{BinaryOperatorKind, OperandSide};
use super::parentheses::ExpressionContent;
use super::{
    ExpressionContext, ExpressionPosition, Formatter, JsxCallbackContext, OperandIndent,
    ParenthesesOwner, TailOwner,
};
use crate::formatter::comments::{CommentBinding, CommentRunLayout, ExpressionTail, NodeHead};
use crate::formatter::diagnostics::{FlatSeparator, LineConstraint};
use crate::formatter::suppression::ContentMode;
use crate::formatter::types::generics::TypeListKind;
use crate::{
    FormatError, LineEnding, QuoteStyle,
    document::{Fragment, Writer},
};

#[derive(Default)]
struct JsxOpeningLayout {
    multiple_attributes: bool,
}

#[derive(Clone, Copy)]
enum JsxBraceRole {
    Attribute,
    Child,
}

struct JsxBraceEnd {
    position: u32,
    line_comment: bool,
    parent_tail: bool,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn jsx_expression_needs_parentheses(position: ExpressionPosition) -> bool {
        matches!(
            position,
            ExpressionPosition::UnaryArgument(_)
                | ExpressionPosition::AwaitArgument
                | ExpressionPosition::SpreadArgument
                | ExpressionPosition::JsxSpreadAttribute
                | ExpressionPosition::JsxSpreadChild
                | ExpressionPosition::SequenceMember
                | ExpressionPosition::ForSequenceMember
                | ExpressionPosition::TemplateInterpolation
                | ExpressionPosition::TypeBinaryOperand
                | ExpressionPosition::TypeAssertionOperand
                | ExpressionPosition::NonNullOperand
                | ExpressionPosition::InstantiationOperand
                | ExpressionPosition::MemberObject(_)
                | ExpressionPosition::MemberAssignmentObject
                | ExpressionPosition::CallCallee { .. }
                | ExpressionPosition::TemplateTag
                | ExpressionPosition::NewCallee
                | ExpressionPosition::Superclass
                | ExpressionPosition::UpdateArgument
                | ExpressionPosition::AssignmentTarget
                | ExpressionPosition::ForInitializer {
                    parentheses: true,
                    ..
                }
                | ExpressionPosition::BinaryOperand {
                    operator: BinaryOperatorKind::Binary(
                        swc_next_ecma_ast::BinaryOperator::LessThan
                    ),
                    side: OperandSide::Left,
                }
        )
    }

    pub(super) fn jsx_grammar_pair_is_enclosed(position: ExpressionPosition) -> bool {
        Self::jsx_expression_needs_parentheses(position)
            && !matches!(
                position,
                ExpressionPosition::CallCallee { .. } | ExpressionPosition::NewCallee
            )
    }

    pub(super) fn jsx_expression_has_optional_frame(context: ExpressionContext) -> bool {
        context.parentheses_owner == ParenthesesOwner::Expression
            && matches!(
                context.position,
                ExpressionPosition::AssignmentRight(_)
                    | ExpressionPosition::Initializer(_)
                    | ExpressionPosition::ArrowBody
                    | ExpressionPosition::RestrictedArgument
                    | ExpressionPosition::YieldArgument
                    | ExpressionPosition::PropertyValue
                    | ExpressionPosition::ExportDefault
            )
    }

    pub(super) fn format_jsx_expression<T>(
        &mut self,
        span: Span,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
        content: impl FnOnce(&mut Self, &mut Writer<'ast>) -> Result<T, FormatError>,
    ) -> Result<T, FormatError> {
        if !Self::jsx_expression_has_optional_frame(context) {
            return content(self, writer);
        }
        let callback_body = matches!(context.jsx_callback, JsxCallbackContext::Body);
        let opening = self.line_constraint(span.start, span.start);
        let closing = self.line_constraint(span.end, span.end);
        writer.group(callback_body, |writer| {
            writer.if_break("(");
            let result = writer.indent(|writer| {
                opening.write(writer, FlatSeparator::Empty, Writer::soft_line);
                content(self, writer)
            })?;
            closing.write(writer, FlatSeparator::Empty, Writer::soft_line);
            writer.if_break(")");
            Ok(result)
        })
    }

    pub(super) fn format_jsx_element(
        &mut self,
        node: JsxElement,
        tail_owner: TailOwner,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let ast = self.ast;
        let opening = node.opening_element(ast);
        let children = node.children(ast);
        if opening.self_closing(ast) {
            if !children.is_empty() || node.closing_element(ast).is_some() {
                return Err(self.unsupported("self-closing JSX children", node));
            }
            return self.format_jsx_opening(opening, writer).map(|()| None);
        }
        let closing = node
            .closing_element(ast)
            .ok_or_else(|| self.unsupported("JSX closing element", node))?;
        if children.is_empty() {
            self.format_jsx_opening(opening, writer)?;
            return self.format_jsx_closing(closing, tail_owner, writer);
        }
        let (opening_doc, opening_layout) = self.record_jsx_opening(opening, writer)?;
        let (children_doc, children_layout) =
            writer.record_with_value(|writer| self.format_jsx_children(children, writer))?;
        let opening_constraint = children_layout.source.map_or_else(
            || self.line_constraint(opening.span(ast).end, closing.span(ast).start),
            |source| self.line_constraint(opening.span(ast).end, source.start),
        );
        let closing_constraint = children_layout
            .source
            .map_or(LineConstraint::Unconstrained, |source| {
                self.line_constraint(source.end, closing.span(ast).start)
            });
        if children_layout.template_only {
            writer.write_fragment(opening_doc);
            opening_constraint.write(writer, FlatSeparator::Empty, |_| {});
            writer.write_fragment(children_doc);
            closing_constraint.write(writer, FlatSeparator::Empty, |_| {});
            return self.format_jsx_closing(closing, tail_owner, writer);
        }
        let expand = children_layout.has_content
            && (opening_layout.multiple_attributes
                || children_layout.contains_tag
                || children_layout.multiple_expressions);
        writer.group(expand, |writer| {
            writer.write_fragment(opening_doc);
            if children_layout.has_content {
                writer.indent(|writer| {
                    opening_constraint.write(writer, FlatSeparator::Empty, |writer| {
                        if !children_layout.leading_inline {
                            writer.soft_line();
                        }
                    });
                    children_layout.write(children_doc, writer);
                });
            } else {
                opening_constraint.write(writer, FlatSeparator::Empty, |_| {});
                children_layout.write(children_doc, writer);
            }
            closing_constraint.write(writer, FlatSeparator::Empty, |writer| {
                if children_layout.has_content && !children_layout.trailing_inline {
                    writer.soft_line();
                }
            });
            self.format_jsx_closing(closing, tail_owner, writer)
        })
    }

    fn format_jsx_closing(
        &mut self,
        closing: JsxClosingElement,
        tail_owner: TailOwner,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let ast = self.ast;
        let name = closing.name(ast);
        let name_span = name.span(ast);
        let span = closing.span(ast);
        let prefix = Span::new(span.start + 1, name_span.start);
        let comments = self.comments.in_range(prefix);
        let slash = self
            .find_token(prefix, comments, "/")
            .ok_or_else(|| FormatError::unsupported("JSX closing slash", None, prefix))?;
        let binding = self.comment_binding(comments, prefix, None);
        if matches!(binding, CommentBinding::Unbound) {
            writer.text("</");
            self.format_jsx_name_head(name, comments, true, writer)?;
        } else {
            writer.text("<");
            self.format_source_token_gap(prefix.start, slash, false, writer)?;
            writer.text("/");
            self.format_source_token_gap(slash + 1, prefix.end, false, writer)?;
            self.format_jsx_name(name, writer)?;
        }
        let tail = self
            .comments
            .between(Span::new(name_span.end, span.end - 1))?;
        let trailing =
            self.prepare_delimited_trailing_run(tail, name_span.end, span.end - 1, false, None);
        // A JS expression consumer owns punctuation after the closing tag. Direct
        // JSX children and attribute values must keep native comments in the tag.
        if tail_owner == TailOwner::ParentOperation
            && trailing.is_some_and(|run| run.len() == tail.len())
        {
            writer.text(">");
            return Ok(Some(ExpressionTail {
                source_end: name_span.end,
                comments: tail,
            }));
        }
        self.format_source_token_gap(name_span.end, span.end - 1, false, writer)?;
        writer.text(">");
        Ok(None)
    }

    pub(super) fn format_jsx_fragment(
        &mut self,
        node: JsxFragment,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
        let opening = node.opening_fragment(ast).span(ast);
        let closing = node.closing_fragment(ast).span(ast);
        if node.children(ast).is_empty() {
            self.format_jsx_fragment_opening(opening, writer)?;
            self.format_jsx_fragment_closing(closing, writer)?;
            return Ok(());
        }
        let (opening_doc, ()) =
            writer.record_with_value(|writer| self.format_jsx_fragment_opening(opening, writer))?;
        let (children_doc, layout) = writer
            .record_with_value(|writer| self.format_jsx_children(node.children(ast), writer))?;
        let opening_constraint = layout.source.map_or_else(
            || self.line_constraint(opening.end, closing.start),
            |source| self.line_constraint(opening.end, source.start),
        );
        let closing_constraint = layout
            .source
            .map_or(LineConstraint::Unconstrained, |source| {
                self.line_constraint(source.end, closing.start)
            });
        if layout.template_only {
            writer.write_fragment(opening_doc);
            opening_constraint.write(writer, FlatSeparator::Empty, |_| {});
            writer.write_fragment(children_doc);
            closing_constraint.write(writer, FlatSeparator::Empty, |_| {});
            self.format_jsx_fragment_closing(closing, writer)?;
            return Ok(());
        }
        let expand = layout.has_content && (layout.contains_tag || layout.multiple_expressions);
        writer.group(expand, |writer| {
            writer.write_fragment(opening_doc);
            if layout.has_content {
                writer.indent(|writer| {
                    opening_constraint.write(writer, FlatSeparator::Empty, |writer| {
                        if !layout.leading_inline {
                            writer.soft_line();
                        }
                    });
                    layout.write(children_doc, writer);
                });
            } else {
                opening_constraint.write(writer, FlatSeparator::Empty, |_| {});
                layout.write(children_doc, writer);
            }
            closing_constraint.write(writer, FlatSeparator::Empty, |writer| {
                if layout.has_content && !layout.trailing_inline {
                    writer.soft_line();
                }
            });
            self.format_jsx_fragment_closing(closing, writer)?;
            Ok(())
        })
    }

    fn format_jsx_fragment_opening(
        &mut self,
        opening: Span,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let comments = self
            .comments
            .between(Span::new(opening.start + 1, opening.end - 1))?;
        writer.text("<");
        self.format_jsx_dangling_comments(comments, true, writer)?;
        writer.text(">");
        Ok(())
    }

    fn format_jsx_fragment_closing(
        &mut self,
        closing: Span,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let prefix = Span::new(closing.start + 1, closing.end - 1);
        let comments = self.comments.between(prefix)?;
        let slash = self
            .find_token(prefix, comments, "/")
            .ok_or_else(|| FormatError::unsupported("JSX closing slash", None, prefix))?;
        if !matches!(
            self.comment_binding(comments, prefix, None),
            CommentBinding::Unbound
        ) {
            writer.text("<");
            self.format_source_token_gap(prefix.start, slash, false, writer)?;
            writer.text("/");
            self.format_source_token_gap(slash + 1, prefix.end, false, writer)?;
        } else {
            writer.text("</");
            if !comments.is_empty()
                && !comments
                    .iter()
                    .any(|comment| comment.kind == CommentKind::Line)
            {
                writer.space();
            }
            self.format_jsx_dangling_comments(comments, true, writer)?;
        }
        writer.text(">");
        Ok(())
    }

    /// Dangling JSX packets belong to the real container, not an expression continuation.
    fn format_jsx_dangling_comments(
        &mut self,
        comments: &'ast [Comment],
        indent_blocks: bool,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let has_line = comments
            .iter()
            .any(|comment| comment.kind == CommentKind::Line);
        let layout = if comments
            .iter()
            .all(|&comment| self.comment_semantics(comment).is_ordinary())
        {
            CommentRunLayout::CanonicalDangling
        } else {
            CommentRunLayout::Lines
        };
        self.comments.claim(comments)?;
        let write = |writer: &mut Writer<'ast>| {
            if has_line {
                writer.hard_line();
            }
            self.format_comment_run(comments, layout, writer)
        };
        if has_line || indent_blocks {
            writer.indent(write)?;
        } else {
            write(writer)?;
        }
        if has_line {
            writer.hard_line();
        }
        Ok(())
    }

    fn format_jsx_opening(
        &mut self,
        node: JsxOpeningElement,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        if node.attributes(self.ast).is_empty() {
            self.format_jsx_opening_content(node, writer)?;
            return Ok(());
        }
        let (opening, _) = self.record_jsx_opening(node, writer)?;
        writer.write_fragment(opening);
        Ok(())
    }

    fn record_jsx_opening(
        &mut self,
        node: JsxOpeningElement,
        writer: &mut Writer<'ast>,
    ) -> Result<(Fragment<'ast>, JsxOpeningLayout), FormatError> {
        writer.record_with_value(|writer| {
            writer.group(false, |writer| {
                self.format_jsx_opening_content(node, writer)
            })
        })
    }

    fn format_jsx_opening_content(
        &mut self,
        node: JsxOpeningElement,
        writer: &mut Writer<'ast>,
    ) -> Result<JsxOpeningLayout, FormatError> {
        let ast = self.ast;
        let name = node.name(ast);
        let name_span = name.span(ast);
        let arguments = node.type_arguments(ast).map(|arguments| {
            let prefix = self.prepare_type_list_prefix(
                Span::new(name_span.end, arguments.span(ast).start),
                TypeListKind::Arguments,
            );
            (arguments, prefix)
        });
        let attributes = node.attributes(ast);
        let closing = node.span(ast).end - if node.self_closing(ast) { 2 } else { 1 };
        let mut attribute_nodes = attributes.iter().map(|id| {
            let attribute = ast.get_node_in_sub_range(id);
            let data = ast.jsx_attribute_item_data(attribute);
            let string_has_lf = if let JsxAttributeItemData::JsxAttribute(attribute) = data
                && let Some(value) = attribute.value(ast)
                && let JsxAttributeValueData::StringLiteral(literal) =
                    ast.jsx_attribute_value_data(value)
            {
                Some(jsx_attribute_has_lf(ast.get_utf8(literal.raw(ast))))
            } else {
                None
            };
            (attribute, data, string_has_lf)
        });
        let first_attribute = attribute_nodes.next();
        let single_string = attributes.len() == 1
            && first_attribute.is_some_and(|(attribute, _, string_has_lf)| {
                let span = attribute.span(ast);
                string_has_lf == Some(false)
                    && self
                        .comments
                        .in_range(Span::new(node.span(ast).start, span.start))
                        .is_empty()
                    && self
                        .comments
                        .in_range(Span::new(span.end, node.span(ast).end))
                        .is_empty()
            });
        let single_attribute_per_line =
            self.options.single_attribute_per_line && attributes.len() > 1;
        let name_prefix = Span::new(node.span(ast).start + 1, name_span.start);
        let name_head = self.comments.in_range(name_prefix);
        let name_head_is_formatted = !arguments
            .as_ref()
            .is_some_and(|(_, prefix)| prefix.left_mode == ContentMode::Verbatim);
        let name_head_is_unbound = matches!(
            self.comment_binding(name_head, name_prefix, None),
            CommentBinding::Unbound
        );
        writer.text("<");
        if name_head_is_formatted && name_head_is_unbound {
            self.format_jsx_name_head(name, name_head, false, writer)?;
        } else {
            self.format_source_token_gap(name_prefix.start, name_prefix.end, false, writer)?;
            if name_head_is_formatted {
                self.format_jsx_name(name, writer)?;
            } else {
                self.format_verbatim(name_span, writer)?;
            }
        }
        let mut end = name_span.end;
        if let Some((arguments, ref prefix)) = arguments {
            let mode = prefix.right_mode;
            self.format_type_list_prefix(prefix, writer)?;
            self.format_type_arguments(arguments, mode, writer)?;
            end = arguments.span(ast).end;
        }
        if attributes.is_empty() {
            let comments = self.comments.between(Span::new(end, closing))?;
            let boundary = self.prepare_comment_boundary(comments, end, closing);
            self.format_comment_boundary(&boundary, writer)?;
            if node.self_closing(ast) {
                boundary
                    .gap()
                    .write(writer, FlatSeparator::Space, Writer::space);
                writer.text("/>");
            } else {
                boundary
                    .gap()
                    .write(writer, FlatSeparator::Empty, |writer| {
                        if name_head_is_unbound && !name_head.is_empty() {
                            writer.soft_line();
                        }
                    });
                writer.text(">");
            }
            return Ok(JsxOpeningLayout::default());
        }
        if let Some((attribute, _, _)) = first_attribute {
            let start = attribute.span(ast).start;
            let comments = self.comments.between(Span::new(end, start))?;
            if let Some(trailing) =
                self.prepare_delimited_trailing_run(comments, end, start, true, None)
            {
                self.claim_prepared_trailing_run(trailing, writer)?;
            }
        }
        let format_attributes = |writer: &mut Writer<'ast>| {
            let mut after_attribute = false;
            for (attribute, data, string_has_lf) in
                first_attribute.into_iter().chain(attribute_nodes)
            {
                let span = attribute.span(ast);
                let comments = self.comments.between(Span::new(end, span.start))?;
                let first = comments
                    .first()
                    .map_or(span.start, |comment| comment.span.start);
                if single_string {
                    writer.space();
                } else {
                    self.line_constraint(end, first).write(
                        writer,
                        FlatSeparator::Space,
                        |writer| {
                            if after_attribute && self.next_line_is_empty(end, first) {
                                writer.hard_line();
                                writer.hard_line();
                            } else if single_attribute_per_line {
                                writer.hard_line();
                            } else {
                                writer.line();
                            }
                        },
                    );
                }
                if string_has_lf == Some(true) {
                    writer.break_parent();
                }
                let mode = self.content_mode(comments);
                self.comments.claim(comments)?;
                self.format_comment_run(
                    comments,
                    CommentRunLayout::GroupedLeading(span.start),
                    writer,
                )?;
                if mode == ContentMode::Verbatim {
                    self.format_verbatim(span, writer)?;
                } else {
                    match data {
                        JsxAttributeItemData::JsxAttribute(attribute) => {
                            self.format_jsx_attribute(attribute, writer)?
                        }
                        JsxAttributeItemData::JsxSpreadAttribute(spread) => {
                            self.format_jsx_braced_expression(
                                spread.argument(ast),
                                span,
                                true,
                                JsxBraceRole::Attribute,
                                writer,
                            )?;
                        }
                    }
                }
                end = span.end;
                after_attribute = true;
            }
            let comments = self.comments.between(Span::new(end, closing))?;
            let boundary = self.prepare_comment_boundary(comments, end, closing);
            self.format_comment_boundary(&boundary, writer)?;
            Ok::<_, FormatError>(boundary.gap())
        };
        let closing_gap = if single_string {
            format_attributes(writer)
        } else {
            writer.indent(format_attributes)
        }?;
        closing_gap.write(writer, FlatSeparator::Empty, |writer| {
            if single_string {
                if node.self_closing(ast) {
                    writer.space();
                }
            } else if node.self_closing(ast) {
                writer.line();
            } else if !self.options.bracket_same_line {
                writer.soft_line();
            }
        });
        writer.text(if node.self_closing(ast) { "/>" } else { ">" });
        Ok(JsxOpeningLayout {
            multiple_attributes: attributes.len() > 1,
        })
    }

    /// The tag name owns its complete leading packet and any line continuation.
    fn format_jsx_name_head(
        &mut self,
        name: JsxElementName,
        comments: &'ast [Comment],
        closing: bool,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let has_line = comments
            .iter()
            .any(|comment| comment.kind == CommentKind::Line);
        self.comments.claim(comments)?;
        let mut write = |writer: &mut Writer<'ast>| {
            self.format_comment_run(
                comments,
                CommentRunLayout::GroupedLeading(name.span(self.ast).start),
                writer,
            )?;
            self.format_jsx_name(name, writer)
        };
        if has_line {
            writer.indent(|writer| {
                writer.hard_line();
                write(writer)
            })?;
            if closing {
                writer.hard_line();
            }
        } else {
            if !comments.is_empty() {
                writer.space();
            }
            write(writer)?;
        }
        Ok(())
    }

    fn format_jsx_name(
        &mut self,
        name: JsxElementName,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        match self.ast.jsx_element_name_data(name) {
            JsxElementNameData::JsxIdentifier(identifier) => {
                writer.text(self.ast.get_utf8(identifier.name(self.ast)))
            }
            JsxElementNameData::JsxNamespacedName(name) => {
                self.format_jsx_namespace(name, writer)?
            }
            JsxElementNameData::JsxMemberExpression(member) => {
                self.format_jsx_member(member, writer)?
            }
        }
        Ok(())
    }

    fn format_jsx_member(
        &mut self,
        member: JsxMemberExpression,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
        let object = member.object(ast);
        match ast.jsx_member_expression_object_data(object) {
            JsxMemberExpressionObjectData::JsxIdentifier(identifier) => {
                writer.text(ast.get_utf8(identifier.name(ast)))
            }
            JsxMemberExpressionObjectData::JsxMemberExpression(member) => {
                self.format_jsx_member(member, writer)?
            }
        }
        let property = member.property(ast);
        let gap = Span::new(object.span(ast).end, property.span(ast).start);
        let comments = self.comments.in_range(gap);
        let dot = self
            .find_token(gap, comments, ".")
            .ok_or_else(|| self.unsupported("JSX member dot", member))?;
        self.format_source_token_gap(gap.start, dot, false, writer)?;
        writer.text(".");
        self.format_source_token_gap(dot + 1, gap.end, false, writer)?;
        writer.text(ast.get_utf8(property.name(ast)));
        Ok(())
    }

    fn format_jsx_namespace(
        &mut self,
        name: JsxNamespacedName,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
        let namespace = name.namespace(ast);
        let local = name.name(ast);
        let gap = Span::new(namespace.span(ast).end, local.span(ast).start);
        let comments = self.comments.in_range(gap);
        let colon = self
            .find_token(gap, comments, ":")
            .ok_or_else(|| self.unsupported("JSX namespace colon", name))?;
        writer.text(ast.get_utf8(namespace.name(ast)));
        self.format_source_token_gap(gap.start, colon, false, writer)?;
        writer.text(":");
        self.format_source_token_gap(colon + 1, gap.end, false, writer)?;
        writer.text(ast.get_utf8(local.name(ast)));
        Ok(())
    }

    fn format_jsx_attribute_name(
        &mut self,
        name: JsxAttributeName,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        match self.ast.jsx_attribute_name_data(name) {
            JsxAttributeNameData::JsxIdentifier(identifier) => {
                writer.text(self.ast.get_utf8(identifier.name(self.ast)))
            }
            JsxAttributeNameData::JsxNamespacedName(name) => {
                self.format_jsx_namespace(name, writer)?
            }
        }
        Ok(())
    }

    fn format_jsx_attribute(
        &mut self,
        attribute: JsxAttribute,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
        let name = attribute.name(ast);
        self.format_jsx_attribute_name(name, writer)?;
        let Some(value) = attribute.value(ast) else {
            return Ok(());
        };
        let gap = Span::new(name.span(ast).end, value.span(ast).start);
        let comments = self.comments.in_range(gap);
        let equals = self
            .find_token(gap, comments, "=")
            .ok_or_else(|| self.unsupported("JSX attribute equals", attribute))?;
        self.format_source_token_gap(gap.start, equals, false, writer)?;
        writer.text("=");
        let leading = self.comments.between(Span::new(equals + 1, gap.end))?;
        let mode = self.content_mode(leading);
        self.comments.claim(leading)?;
        self.format_comment_run(leading, CommentRunLayout::FlatLeading(gap.end), writer)?;
        if mode == ContentMode::Verbatim {
            return self.format_verbatim(value.span(ast), writer);
        }
        match ast.jsx_attribute_value_data(value) {
            JsxAttributeValueData::StringLiteral(literal) => {
                self.format_jsx_string(literal, writer)?
            }
            JsxAttributeValueData::JsxExpressionContainer(container) => {
                self.format_jsx_container(container, JsxBraceRole::Attribute, writer)?
            }
            JsxAttributeValueData::JsxElement(element) => self
                .format_jsx_element(element, TailOwner::Expression, writer)
                .map(|_| ())?,
            JsxAttributeValueData::JsxFragment(fragment) => {
                self.format_jsx_fragment(fragment, writer)?
            }
        }
        Ok(())
    }

    fn format_jsx_string(
        &self,
        literal: StringLiteral,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let raw = self.ast.get_utf8(literal.raw(self.ast));
        let quote = raw.as_bytes().first().copied();
        if !matches!(quote, Some(b'\'' | b'"'))
            || raw.len() < 2
            || raw.as_bytes().last().copied() != quote
        {
            return Err(self.unsupported("JSX quoted attribute", literal));
        }
        writer.literal_text(jsx_attribute_string(
            raw,
            &raw[1..raw.len() - 1],
            self.options.jsx_quote_style,
            self.options.line_ending,
        ));
        Ok(())
    }

    fn format_jsx_container(
        &mut self,
        container: JsxExpressionContainer,
        role: JsxBraceRole,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        match self.ast.jsx_expression_data(container.expression(self.ast)) {
            JsxExpressionData::Expr(expression) => self.format_jsx_braced_expression(
                expression,
                container.span(self.ast),
                false,
                role,
                writer,
            ),
            JsxExpressionData::JsxEmptyExpression(empty) => {
                let span = empty.span(self.ast);
                let comments = self.comments.between(span)?;
                if comments.is_empty() {
                    writer.text("{}");
                    return Ok(());
                }
                writer.group(false, |writer| {
                    writer.text("{");
                    self.format_jsx_dangling_comments(comments, false, writer)?;
                    writer.text("}");
                    Ok(())
                })
            }
        }
    }

    fn format_jsx_braced_expression(
        &mut self,
        expression: Expr,
        envelope: Span,
        spread: bool,
        role: JsxBraceRole,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        self.format_jsx_braced_expression_prepared(expression, envelope, spread, role, None, writer)
    }

    fn format_jsx_braced_expression_prepared(
        &mut self,
        expression: Expr,
        envelope: Span,
        spread: bool,
        role: JsxBraceRole,
        content: Option<ExpressionContent>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        if spread {
            return self.format_jsx_spread_expression(expression, envelope, role, content, writer);
        }
        let ast = self.ast;
        let expression_span = expression.span(ast);
        let opening = envelope.start + 1;
        let closing = envelope.end - 1;
        if expression_span.start < opening || expression_span.end > closing {
            return Err(self.unsupported("JSX expression containment", expression));
        }
        let leading = self
            .comments
            .in_range(Span::new(opening, expression_span.start));
        let mode = if self.content_mode(leading) == ContentMode::Verbatim
            || self.trailing_content_mode(Span::new(expression_span.end, closing), true)
                == ContentMode::Verbatim
        {
            ContentMode::Verbatim
        } else {
            ContentMode::Formatted
        };
        let closing_constraint = self.line_constraint(expression_span.end, closing);
        let mut content = content;
        let (hug, arrow_layout, jsx_callback) = if mode == ContentMode::Formatted {
            let data = content
                .as_ref()
                .map_or_else(|| ast.expr_data(expression), |content| content.data);
            let data = if content.is_none() && matches!(data, ExprData::ParenthesizedExpression(_))
            {
                let prepared = self.expression_content(expression);
                let data = prepared.data;
                content = Some(prepared);
                data
            } else {
                data
            };
            let jsx_callback = self.jsx_container_callback(data);
            let core_span = content
                .as_ref()
                .map_or(expression_span, |content| content.expression.span(ast));
            let hug = self
                .comments
                .in_range(Span::new(opening, core_span.start))
                .is_empty()
                && self
                    .comments
                    .in_range(Span::new(core_span.end, closing))
                    .is_empty()
                && self.jsx_expression_hugs(data, role, jsx_callback);
            // A source-constrained closing gap belongs to the container alone.
            let arrow_layout = if hug
                && matches!(data, ExprData::ArrowFunctionExpression(_))
                && closing_constraint == LineConstraint::Unconstrained
            {
                if content.is_none() {
                    content = Some(ExpressionContent {
                        expression,
                        data,
                        parentheses: None,
                    });
                }
                ArrowLayout::JsxExpression
            } else {
                ArrowLayout::Ordinary
            };
            (hug, arrow_layout, jsx_callback)
        } else {
            (false, ArrowLayout::Ordinary, JsxCallbackContext::Ordinary)
        };
        // Attribute braces own the indent only on this actual non-hug path.
        let operand_indent =
            if matches!(role, JsxBraceRole::Attribute) && mode == ContentMode::Formatted && !hug {
                OperandIndent::Parent
            } else {
                OperandIndent::Expression
            };
        writer.group(false, |writer| {
            writer.text("{");
            self.comments.claim(leading)?;
            let first = leading
                .first()
                .map_or(expression_span.start, |comment| comment.span.start);
            let opening_constraint = self.line_constraint(opening, first);
            let inspect_tail = hug && !self.comments.in_range(expression_span).is_empty();
            let format_content = |writer: &mut Writer<'ast>| {
                let head = NodeHead {
                    comments: leading,
                    content_start: expression_span.start,
                    layout: CommentRunLayout::GroupedLeading(expression_span.start),
                    gap_end: None,
                };
                let context = ExpressionContext::at(ExpressionPosition::JsxExpression)
                    .with_content_mode(mode)
                    .with_arrow_layout(arrow_layout)
                    .with_jsx_callback(jsx_callback)
                    .with_operand_indent(operand_indent)
                    .with_tail_owner(TailOwner::ParentOperation);
                let tail = match content {
                    Some(content) => self.format_prepared_expression_with_leading_comments(
                        expression, content, head, context, writer,
                    ),
                    None => self.format_expression_with_leading_comments(
                        expression, head, context, None, writer,
                    ),
                }?;
                let parent_tail = tail.as_ref().is_some_and(|tail| !tail.comments.is_empty());
                let tail =
                    self.capture_expression_tail(Span::new(expression_span.end, closing), tail)?;
                let last = tail.as_ref().and_then(|tail| tail.comments.last());
                let end = last.map_or(expression_span.end, |comment| comment.span.end);
                self.format_expression_tail(tail, writer)?;
                Ok::<_, FormatError>(JsxBraceEnd {
                    position: end,
                    line_comment: last.is_some_and(|comment| comment.kind == CommentKind::Line),
                    parent_tail,
                })
            };
            let (end, hug) = Self::format_jsx_brace_content(
                opening_constraint,
                hug,
                inspect_tail,
                writer,
                format_content,
            )?;
            let line_comment = end.line_comment;
            let end = end.position;
            let closing_constraint = if end == expression_span.end {
                closing_constraint
            } else {
                self.line_constraint(end, closing)
            };
            closing_constraint.write(writer, FlatSeparator::Empty, |writer| {
                if line_comment {
                    writer.hard_line();
                } else if !hug {
                    writer.soft_line();
                }
            });
            writer.text("}");
            Ok(())
        })
    }

    fn format_jsx_spread_expression(
        &mut self,
        expression: Expr,
        envelope: Span,
        role: JsxBraceRole,
        content: Option<ExpressionContent>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let span = expression.span(self.ast);
        let opening = envelope.start + 1;
        let closing = envelope.end - 1;
        let gap = Span::new(opening, span.start);
        let mut leading = self.comments.in_range(gap);
        let marker = self
            .find_token(gap, leading, "...")
            .ok_or_else(|| self.unsupported("JSX spread boundary", expression))?;
        if span.start < marker + 3 || span.end > closing {
            return Err(self.unsupported("JSX spread containment", expression));
        }
        let before_count = leading.partition_point(|comment| comment.span.end <= marker);
        let exterior_head = &leading[before_count..];
        let mode = if self.content_mode(leading) == ContentMode::Verbatim
            || self.trailing_content_mode(Span::new(span.end, closing), true)
                == ContentMode::Verbatim
        {
            ContentMode::Verbatim
        } else {
            ContentMode::Formatted
        };
        let position = match role {
            JsxBraceRole::Attribute => ExpressionPosition::JsxSpreadAttribute,
            JsxBraceRole::Child => ExpressionPosition::JsxSpreadChild,
        };
        let mut context = ExpressionContext::at(position)
            .with_content_mode(mode)
            .with_tail_owner(TailOwner::ParentOperation);
        let mut content = if mode == ContentMode::Verbatim {
            None
        } else {
            content.or_else(|| {
                expression
                    .as_parenthesized_expression(self.ast)
                    .is_some()
                    .then(|| self.expression_content(expression))
            })
        };
        let source_head = content.as_ref().and_then(|content| {
            (self.has_suppression && matches!(content.data, ExprData::ParenthesizedExpression(_)))
                .then(|| self.source_parentheses_head_content(content.data, context))
        });
        let exterior_binding = self.expression_head_binding(exterior_head, expression);
        let source_binding = content
            .as_ref()
            .and_then(|content| content.parentheses)
            .map_or(exterior_binding, |removed| {
                exterior_binding.combine(removed.head.binding)
            });
        let ignored = source_head
            .as_ref()
            .and_then(|head| head.ignored.as_ref())
            .filter(|_| {
                matches!(source_binding, CommentBinding::Unbound)
                    && !self.diagnostic_range_is_sensitive(envelope)
            });
        let mut body = ignored.map_or(expression, |ignored| ignored.expression);
        let mut body_span = body.span(self.ast);
        if ignored.is_some() {
            leading = self.comments.in_range(Span::new(opening, body_span.start));
            context = context.with_content_mode(ContentMode::Verbatim);
            content = None;
        }
        // Resolve the emitted ignored operand before relocating its complete head.
        // A retained source pair remains opaque; an exposed child keeps its raw body.
        let retained_target = (context.content_mode == ContentMode::Verbatim).then_some(body_span);
        let exterior_head = &leading[before_count..];
        let mut binding = if retained_target.is_some() {
            let binding = self.comment_binding(leading, envelope, retained_target);
            let ordinary = CommentBinding::combine_all(
                exterior_head
                    .iter()
                    .filter(|&&comment| self.comment_semantics(comment).is_ordinary())
                    .map(|comment| {
                        self.expression_head_binding(std::slice::from_ref(comment), body)
                    }),
            );
            if matches!(ordinary, CommentBinding::Unbound) {
                binding
            } else {
                CommentBinding::SourceBound
            }
        } else {
            exterior_binding
        };
        if let Some(parenthesized) = expression.as_parenthesized_expression(self.ast)
            && let Some(prepared) = content.as_ref()
            && self.source_parentheses_head_is_removable(parenthesized, prepared, context)
            && let Some(removed) = prepared.parentheses
        {
            binding = binding.combine(removed.head.binding);
            if matches!(binding, CommentBinding::Unbound) {
                // Select the complete emitted operand before relocating its head.
                // Tail comments stay fenced by the spread's actual closing brace.
                body = prepared.expression;
                body_span = body.span(self.ast);
                leading = self.comments.in_range(Span::new(opening, body_span.start));
                content = Some(ExpressionContent {
                    expression: body,
                    data: prepared.data,
                    parentheses: None,
                });
            }
        }
        let (before, after) = leading.split_at(before_count);
        // A bound head moves only with the complete source operand it still targets.
        let relocate = !after.is_empty() && binding.allows_relocation(retained_target);
        let (prefix, head, prefix_end) = if relocate {
            (leading, &[][..], body_span.start)
        } else {
            (before, after, marker)
        };
        let removed_comments = content.as_ref().is_some_and(|content| {
            if self.jsx_source_pair_is_enclosed(body, content, context) {
                return false;
            }
            if content.parentheses.is_some_and(|removed| {
                removed.head.has_comments
                    || !self
                        .comments
                        .in_range(Span::new(
                            removed.source.inner.end,
                            removed.source.outer.end,
                        ))
                        .is_empty()
            }) {
                return true;
            }
            // An ignored child can emerge from a source pair that the pure
            // removable-content view correctly keeps opaque.
            if self.has_suppression && matches!(content.data, ExprData::ParenthesizedExpression(_))
            {
                return source_head
                    .as_ref()
                    .and_then(|head| head.leading())
                    .is_some_and(|span| !self.comments.in_range(span).is_empty());
            }
            false
        });
        let has_comments = !leading.is_empty()
            || removed_comments
            || !self
                .comments
                .in_range(Span::new(body_span.end, closing))
                .is_empty();
        let first = prefix.first().map_or(marker, |comment| comment.span.start);
        let opening_constraint = self.line_constraint(opening, first);
        let closing_constraint = self.line_constraint(span.end, closing);
        let hug = !has_comments;
        let inspect_tail = hug && !self.comments.in_range(body_span).is_empty();
        writer.group(false, |writer| {
            writer.text("{");
            let format_inner = |writer: &mut Writer<'ast>| {
                self.comments.claim(prefix)?;
                self.format_comment_run(
                    prefix,
                    CommentRunLayout::GroupedLeading(prefix_end),
                    writer,
                )?;
                writer.text("...");
                let first = head
                    .first()
                    .map_or(body_span.start, |comment| comment.span.start);
                self.line_constraint(marker + 3, first)
                    .write(writer, FlatSeparator::Empty, |_| {});
                self.comments.claim(head)?;
                let head = NodeHead {
                    comments: head,
                    content_start: body_span.start,
                    layout: CommentRunLayout::GroupedLeading(body_span.start),
                    gap_end: None,
                };
                let tail = if retained_target.is_some() {
                    self.format_comment_run(head.comments, head.layout, writer)?;
                    self.format_verbatim_expression(body, context, writer)?;
                    None
                } else {
                    match content {
                        Some(content) => self.format_prepared_expression_with_leading_comments(
                            body, content, head, context, writer,
                        ),
                        None => self.format_expression_with_leading_comments(
                            body, head, context, None, writer,
                        ),
                    }?
                };
                let parent_tail = tail.as_ref().is_some_and(|tail| !tail.comments.is_empty());
                let tail = self.capture_expression_tail(Span::new(body_span.end, closing), tail)?;
                let last = tail.as_ref().and_then(|tail| tail.comments.last());
                let end = last.map_or(body_span.end, |comment| comment.span.end);
                self.format_expression_tail(tail, writer)?;
                Ok::<_, FormatError>(JsxBraceEnd {
                    position: end,
                    line_comment: last.is_some_and(|comment| comment.kind == CommentKind::Line),
                    parent_tail,
                })
            };
            let (end, hug) = Self::format_jsx_brace_content(
                opening_constraint,
                hug,
                inspect_tail,
                writer,
                format_inner,
            )?;
            let line_comment = end.line_comment;
            let end = end.position;
            let closing_constraint = if end == body_span.end {
                closing_constraint
            } else {
                self.line_constraint(end, closing)
            };
            closing_constraint.write(writer, FlatSeparator::Empty, |writer| {
                if line_comment {
                    writer.hard_line();
                } else if !hug {
                    writer.soft_line();
                }
            });
            writer.text("}");
            Ok(())
        })
    }

    /// Brace layout follows the complete emitted operand, including comments
    /// transferred from a removed source pair. Record only when such a tail
    /// can change an otherwise inline container.
    fn format_jsx_brace_content(
        opening: LineConstraint,
        hug: bool,
        inspect_tail: bool,
        writer: &mut Writer<'ast>,
        format: impl FnOnce(&mut Writer<'ast>) -> Result<JsxBraceEnd, FormatError>,
    ) -> Result<(JsxBraceEnd, bool), FormatError> {
        if inspect_tail {
            let (fragment, end) = writer.record_with_value(format)?;
            let hug = hug && !end.parent_tail;
            let write = |writer: &mut Writer<'ast>| {
                opening.write(writer, FlatSeparator::Empty, |writer| {
                    if !hug {
                        writer.soft_line();
                    }
                });
                writer.write_fragment(fragment);
            };
            if hug {
                write(writer);
            } else {
                writer.indent(write);
            }
            return Ok((end, hug));
        }
        let write = |writer: &mut Writer<'ast>| {
            opening.write(writer, FlatSeparator::Empty, |writer| {
                if !hug {
                    writer.soft_line();
                }
            });
            format(writer)
        };
        let end = if hug {
            write(writer)
        } else {
            writer.indent(write)
        }?;
        Ok((end, hug))
    }

    /// Inline braces let the expression own its continuation indentation.
    /// Binary and conditional expressions inline only as element children.
    fn jsx_expression_hugs(
        &self,
        mut data: ExprData,
        role: JsxBraceRole,
        mut callback: JsxCallbackContext,
    ) -> bool {
        let mut child = matches!(role, JsxBraceRole::Child);
        loop {
            match data {
                ExprData::ObjectExpression(_)
                | ExprData::ArrayExpression(_)
                | ExprData::CallExpression(_)
                | ExprData::Function(_)
                | ExprData::ArrowFunctionExpression(_)
                | ExprData::TemplateLiteral(_)
                | ExprData::TaggedTemplateExpression(_) => return true,
                ExprData::BinaryExpression(_)
                | ExprData::LogicalExpression(_)
                | ExprData::ConditionalExpression(_) => return child,
                ExprData::AwaitExpression(awaited) => {
                    let argument = self.expression_content(awaited.argument(self.ast));
                    let argument_span = argument.expression.span(self.ast);
                    let awaited_span = awaited.span(self.ast);
                    if !self
                        .comments
                        .in_range(Span::new(awaited_span.start + 5, argument_span.start))
                        .is_empty()
                        || !self
                            .comments
                            .in_range(Span::new(argument_span.end, awaited_span.end))
                            .is_empty()
                    {
                        return false;
                    }
                    data = argument.data;
                    if matches!(data, ExprData::JsxElement(_) | ExprData::JsxFragment(_)) {
                        return true;
                    }
                    child = false;
                    callback = self.jsx_container_callback(data);
                }
                _ => return matches!(callback, JsxCallbackContext::ContainerCall(_)),
            }
        }
    }

    fn jsx_container_callback(&self, mut data: ExprData) -> JsxCallbackContext {
        loop {
            let expression = match data {
                ExprData::ChainExpression(chain) => chain.expression(self.ast),
                ExprData::TsNonNullExpression(assertion) => {
                    let Ok(source) = self.source_non_null(assertion) else {
                        return JsxCallbackContext::Ordinary;
                    };
                    if source.source_break() {
                        return JsxCallbackContext::Ordinary;
                    }
                    source.expression
                }
                _ => break,
            };
            data = self.expression_content(expression).data;
        }
        match data {
            ExprData::CallExpression(call) => JsxCallbackContext::ContainerCall(call),
            _ => JsxCallbackContext::Ordinary,
        }
    }
}

/// The lexer keeps JSX attribute entities in its raw value. The emitted LF
/// entity must keep the same opening layout on the next formatting pass.
fn jsx_attribute_has_lf(raw: &str) -> bool {
    raw.contains('\n')
        || raw.split("&#").skip(1).any(|entity| {
            let Some((digits, _)) = entity.split_once(';') else {
                return false;
            };
            let (digits, radix) = digits
                .strip_prefix(['x', 'X'])
                .map_or((digits, 10), |digits| (digits, 16));
            let valid = digits.bytes().all(|byte| {
                if radix == 10 {
                    byte.is_ascii_digit()
                } else {
                    byte.is_ascii_hexdigit()
                }
            });
            valid && u32::from_str_radix(digits, radix) == Ok(10)
        })
}

fn jsx_attribute_string<'text>(
    raw: &'text str,
    interior: &'text str,
    preferred: QuoteStyle,
    line_ending: LineEnding,
) -> Cow<'text, str> {
    let decoded = if interior.contains("&apos;") || interior.contains("&quot;") {
        Cow::Owned(interior.replace("&apos;", "'").replace("&quot;", "\""))
    } else {
        Cow::Borrowed(interior)
    };
    let single = decoded.bytes().filter(|&byte| byte == b'\'').count();
    let double = decoded.bytes().filter(|&byte| byte == b'"').count();
    let quote = match preferred {
        QuoteStyle::Double if double <= single => '"',
        QuoteStyle::Single if single <= double => '\'',
        QuoteStyle::Double => '\'',
        QuoteStyle::Single => '"',
    };
    let encode_lines = !jsx_attribute_lines_match(&decoded, line_ending);
    if !encode_lines
        && matches!(decoded, Cow::Borrowed(_))
        && raw.starts_with(quote)
        && !decoded.contains(quote)
    {
        return Cow::Borrowed(raw);
    }
    let entity = if quote == '"' { "&quot;" } else { "&apos;" };
    let mut output = String::with_capacity(raw.len());
    output.push(quote);
    for character in decoded.chars() {
        match character {
            character if character == quote => output.push_str(entity),
            '\r' if encode_lines => output.push_str("&#13;"),
            '\n' if encode_lines => output.push_str("&#10;"),
            character => output.push(character),
        }
    }
    output.push(quote);
    Cow::Owned(output)
}

/// A literal document line uses the selected output EOL. Attribute values are
/// literal JSX data, so incompatible physical newlines need JSX entity spelling.
fn jsx_attribute_lines_match(raw: &str, ending: LineEnding) -> bool {
    let mut bytes = raw.bytes().peekable();
    while let Some(byte) = bytes.next() {
        match byte {
            b'\r' if bytes.peek() == Some(&b'\n') => {
                bytes.next();
                if ending != LineEnding::CrLf {
                    return false;
                }
            }
            b'\r' if ending != LineEnding::Cr => return false,
            b'\n' if ending != LineEnding::Lf => return false,
            _ => {}
        }
    }
    true
}
