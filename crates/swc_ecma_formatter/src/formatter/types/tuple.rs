//! Tuple slots reuse the borrowed item/comma protocol and keep label/type owners separate.

use swc_next_ecma_ast::{
    CommentKind, GetSpan, Span, TsNamedTupleMember, TsOptionalType, TsRestType, TsTupleElementData,
    TsTupleType,
};

use super::context::TypeContext;
use super::{Formatter, PreparedType, TypeBoundaryLayout, TypeLayout};
use crate::formatter::comments::{CommentRunLayout, ExpressionTail};
use crate::formatter::diagnostics::FlatSeparator;
use crate::formatter::expressions::arguments::{ArgumentLayout, FollowingItem, Separator};
use crate::formatter::suppression::{ContentMode, SuppressionPosition};
use crate::{FormatError, TrailingCommas, document::Writer};

enum PreparedTupleElement {
    Type(PreparedType),
    Named(TsNamedTupleMember),
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_tuple_type(
        &mut self,
        node: TsTupleType,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let ast = self.ast;
        let span = node.span(ast);
        let content = Span::new(span.start + 1, span.end - 1);
        let elements = node.element_types(ast);
        let layout = if elements.len() > 1 {
            TypeLayout::MultipleTupleElement
        } else {
            TypeLayout::CallerIndented
        };
        let empty_dangling = if elements.is_empty() {
            let comments = self.comments.in_range(content);
            if comments.is_empty() {
                writer.text("[]");
                return Ok(None);
            }
            if self.content_mode(comments) == ContentMode::Verbatim {
                self.format_verbatim(span, writer)?;
                return Ok(None);
            }
            let layout = if self
                .comment_binding(comments, content, None)
                .allows_relocation(None)
            {
                CommentRunLayout::CanonicalDangling
            } else {
                CommentRunLayout::Lines
            };
            Some((comments, layout))
        } else {
            None
        };
        writer.group(false, |writer| {
            writer.text("[");
            let end = writer.indent(|writer| {
                if empty_dangling.is_some() {
                    self.opening_line_constraint(content).write(
                        writer,
                        FlatSeparator::Empty,
                        Writer::soft_line,
                    );
                }
                let mut elements = elements
                    .iter()
                    .map(|id| ast.get_node_in_sub_range(id))
                    .peekable();
                let mut end = content.start;
                let mut source_end = content.start;
                let mut separator = Separator::Line;
                let mut following_mode = None;
                while let Some(element) = elements.next() {
                    let element_span = element.span(ast);
                    let leading = self
                        .comments
                        .in_range(Span::new(source_end, element_span.start));
                    let mode = following_mode.unwrap_or_else(|| self.content_mode(leading));
                    let mut element_content = self.prepare_tuple_element(element);
                    if mode == ContentMode::Verbatim
                        && let PreparedTupleElement::Type(node) = &mut element_content
                    {
                        self.select_delimited_type_head(node, leading, element_span.start, following_mode.is_none());
                    }
                    let selected_child = matches!(&element_content,
                        PreparedTupleElement::Type(node) if node.source_owner.is_some_and(|owner| owner != node.node));
                    let leading_mode = if selected_child { ContentMode::Formatted } else { mode };
                    if following_mode.is_some() {
                        let next_start = self
                            .comments
                            .between(Span::new(end, element_span.start))?
                            .first()
                            .map_or(element_span.start, |comment| comment.span.start);
                        let constraint = if leading_mode == ContentMode::Verbatim {
                            self.source_line_constraint(end, next_start)
                        } else {
                            self.line_constraint(end, next_start)
                        };
                        ArgumentLayout::Lines.separator(separator, constraint, writer);
                    } else {
                        // A whole Source target retains its leading placement so a break
                        // cannot change an Inline ignore into an OwnLine type-head ignore.
                        let opening = if leading_mode == ContentMode::Verbatim {
                            let start = leading.first().map_or(element_span.start, |comment| comment.span.start);
                            self.source_line_constraint(content.start, start)
                        } else {
                            self.opening_line_constraint(content)
                        };
                        opening.write(writer, FlatSeparator::Empty, Writer::soft_line);
                    }
                    let following = elements.peek().map_or(FollowingItem::End, |next| {
                        FollowingItem::Node(next.span(ast))
                    });
                    let mut item = self.prepare_delimited_item(
                        element_span,
                        following,
                        Span::new(end, content.end),
                        self.options.trailing_commas != TrailingCommas::None,
                        writer,
                    )?;
                    item.set_leading_mode(leading_mode);
                    let tail = self.format_tuple_element(element_content, item.mode, TypeContext::Root, layout, writer)?;
                    let boundary = self.finish_delimited_item(item, tail, writer)?;
                    end = boundary.end;
                    source_end = element_span.end;
                    separator = boundary.separator;
                    following_mode = Some(boundary.following_mode);
                }
                let (dangling, layout) = if let Some((comments, layout)) = empty_dangling {
                    self.comments.claim(comments)?;
                    (comments, layout)
                } else {
                    let comments = self.comments.between(Span::new(end, content.end))?;
                    (self.comments.take(comments.len()), CommentRunLayout::Lines)
                };
                if dangling
                    .iter()
                    .any(|comment| comment.kind == CommentKind::Line)
                {
                    writer.break_parent();
                }
                if following_mode.is_some()
                    && let Some(first) = dangling.first()
                {
                    self.comment_gap(end, first.span.start, writer);
                }
                self.format_comment_run(dangling, layout, writer)?;
                Ok::<_, FormatError>(dangling.last().map_or(end, |comment| comment.span.end))
            })?;
            self.line_constraint(end, content.end).write(
                writer,
                FlatSeparator::Empty,
                Writer::soft_line,
            );
            writer.text("]");
            Ok(None)
        })
    }

    pub(super) fn format_optional_type(
        &mut self,
        node: TsOptionalType,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let ast = self.ast;
        let child = node.type_annotation(ast);
        let gap = Span::new(child.span(ast).end, node.span(ast).end - 1);
        let mode = self.content_mode(self.comments.in_range(gap));
        let tail =
            self.format_type_with_context(child, mode, TypeContext::JsDocPostfixOperand, writer)?;
        self.format_expression_tail(tail, writer)?;
        if mode == ContentMode::Verbatim {
            self.format_verbatim(gap, writer)?;
        } else {
            self.format_source_token_gap(gap.start, gap.end, false, writer)?;
        }
        writer.text("?");
        Ok(None)
    }

    pub(super) fn format_rest_type(
        &mut self,
        node: TsRestType,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let ast = self.ast;
        let child = node.type_annotation(ast);
        let gap = Span::new(node.span(ast).start + 3, child.span(ast).start);
        let prefix = self.prepare_type_prefix(gap);
        let mut child = self.prepare_tuple_element(child);
        if prefix.mode == ContentMode::Verbatim
            && let PreparedTupleElement::Type(node) = &mut child
        {
            self.select_type_head_comments(node, self.comments.in_range(gap), gap.end);
        }
        writer.text("...");
        match child {
            PreparedTupleElement::Type(node) => self.format_prepared_type_after_prefix(
                node,
                prefix,
                TypeContext::RestOperand,
                TypeLayout::CallerIndented,
                writer,
            ),
            PreparedTupleElement::Named(node) => {
                let mode = prefix.mode;
                self.format_type_prefix(prefix, FlatSeparator::Empty, writer)?;
                self.format_named_tuple_member(node, mode, writer)
            }
        }
    }

    fn prepare_tuple_element(
        &self,
        element: swc_next_ecma_ast::TsTupleElement,
    ) -> PreparedTupleElement {
        match self.ast.ts_tuple_element_data(element) {
            TsTupleElementData::TsType(node) => {
                PreparedTupleElement::Type(self.prepare_type(node, ContentMode::Formatted))
            }
            TsTupleElementData::TsNamedTupleMember(node) => PreparedTupleElement::Named(node),
        }
    }

    fn format_tuple_element(
        &mut self,
        element: PreparedTupleElement,
        mode: ContentMode,
        context: TypeContext,
        layout: TypeLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        match element {
            PreparedTupleElement::Type(mut node) => {
                if mode == ContentMode::Verbatim {
                    node.source_owner = Some(node.node);
                }
                self.format_prepared_type_with_layout(node, context, layout, writer)
            }
            PreparedTupleElement::Named(node) => self.format_named_tuple_member(node, mode, writer),
        }
    }

    fn format_named_tuple_member(
        &mut self,
        node: TsNamedTupleMember,
        mode: ContentMode,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let ast = self.ast;
        if mode == ContentMode::Verbatim {
            self.format_verbatim(node.span(ast), writer)?;
            return Ok(None);
        }
        let label = node.label(ast);
        let annotation = node.element_type(ast);
        let mut type_node = self.prepare_type(annotation, ContentMode::Formatted);
        let gap = Span::new(label.span(ast).end, annotation.span(ast).start);
        let comments = self.comments.in_range(gap);
        let Some(colon) = self.find_token(gap, comments, ":") else {
            return Err(self.unsupported("named tuple colon", node));
        };
        let split = comments.partition_point(|comment| comment.span.end <= colon);
        let (before, after) = comments.split_at(split);
        let mut label_mode = ContentMode::Formatted;
        if self.has_suppression && self.content_mode(comments) == ContentMode::Verbatim {
            for (_, position) in self.suppressed_comment_runs(before, colon) {
                if matches!(position, SuppressionPosition::OwnLine) {
                    self.select_type_head(&mut type_node, position);
                } else {
                    label_mode = ContentMode::Verbatim;
                }
            }
            for (_, position) in self.suppressed_comment_runs(after, gap.end) {
                if matches!(position, SuppressionPosition::EndOfLine) {
                    label_mode = ContentMode::Verbatim;
                } else {
                    self.select_type_head(&mut type_node, position);
                }
            }
        }
        let source = label_mode == ContentMode::Verbatim
            || type_node.source_owner.is_some()
            || node.optional(ast)
                && (!comments.is_empty() || self.diagnostic_range_is_sensitive(gap));
        let layout = if source {
            TypeBoundaryLayout::Source(gap)
        } else {
            TypeBoundaryLayout::Formatted {
                before: self.prepare_comment_boundary(before, gap.start, colon),
                after: self.prepare_comment_boundary(after, colon + 1, gap.end),
            }
        };
        if label_mode == ContentMode::Verbatim {
            self.format_verbatim(label.span(ast), writer)?;
        } else {
            writer.text(ast.get_utf8(label.name(ast)));
        }
        match layout {
            TypeBoundaryLayout::Source(gap) => self.format_verbatim(gap, writer)?,
            TypeBoundaryLayout::Formatted { before, after } => {
                if node.optional(ast) {
                    writer.text("?");
                }
                self.format_type_comment_gap(&before, FlatSeparator::Empty, writer)?;
                writer.text(":");
                self.format_type_comment_gap(&after, FlatSeparator::Space, writer)?;
            }
        }
        self.format_prepared_type_with_layout(
            type_node,
            TypeContext::Root,
            TypeLayout::CallerIndented,
            writer,
        )
    }
}
