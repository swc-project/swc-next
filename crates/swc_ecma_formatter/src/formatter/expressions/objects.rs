//! Object literal properties and their source-ordered comment boundaries.

use swc_next_ecma_ast::{
    Argument, Comment, CommentKind, Expr, ExprData, GetSpan, ObjectExpression, ObjectProperty,
    ObjectPropertyKind, ObjectPropertyKindData, PropertyKey, PropertyKeyData, PropertyKind,
    SimpleAssignmentTargetData, Span, StringLiteral, SubRange,
};

use super::assignment::RhsLayout;
use super::parentheses::ResolvedExpressionHead;
use super::{ExpressionContext, ExpressionPosition, Formatter, OperandIndent};
use crate::formatter::comments::{CommentBinding, CommentRunLayout, ExpressionTail, NodeHead};
use crate::formatter::diagnostics::{FlatSeparator, LineConstraint};

use crate::formatter::literals::StringContext;
use crate::formatter::suppression::{ContentMode, DelimitedFollowing, SuppressionPosition};
use crate::{
    FormatError, QuoteProperties, TrailingCommas,
    document::{Fragment, GroupRequest, Writer},
};

/// A whole key group resolves `consistent` once before formatting any member.
#[derive(Clone, Copy, Eq, PartialEq)]
pub(in crate::formatter) enum PropertyKeyQuoting {
    AsNeeded,
    Preserve,
}

enum SourceObject<'ast> {
    Verbatim(Span),
    Formatted(FormattedObject<'ast>),
}

struct FormattedObject<'ast> {
    expand: bool,
    contents: ObjectContents<'ast>,
}

enum ObjectContents<'ast> {
    Empty {
        content: Span,
        comments: &'ast [Comment],
    },
    Properties {
        content: Span,
        quoting: PropertyKeyQuoting,
        first: ObjectPropertyKind,
        remaining: SubRange<ObjectPropertyKind>,
    },
}

struct ObjectBoundary<'ast> {
    inline: &'ast [Comment],
    own_line: &'ast [Comment],
    leading: &'ast [Comment],
    trailing_end: u32,
    separator: Span,
}

pub(in crate::formatter) struct PropertySuppression {
    pub(in crate::formatter) key: ContentMode,
    pub(in crate::formatter) value: ContentMode,
    pub(in crate::formatter) enclosing: ContentMode,
}

#[derive(Clone, Copy)]
enum ComputedKeyScope {
    Text,
    Expression,
}

struct PreparedComputedKey<'ast> {
    inner: Fragment<'ast>,
    scope: ComputedKeyScope,
    opening: LineConstraint,
    closing: LineConstraint,
}

impl<'ast> PreparedComputedKey<'ast> {
    fn write(self, writer: &mut Writer<'ast>) {
        let clean = matches!(self.scope, ComputedKeyScope::Text)
            && self.opening == LineConstraint::Unconstrained
            && self.closing == LineConstraint::Unconstrained
            && self.inner.clean_text_width().is_some();
        if clean {
            writer.text("[");
            writer.write_fragment(self.inner);
            writer.text("]");
        } else {
            writer.group(false, |writer| {
                writer.text("[");
                writer.indent(|writer| {
                    self.opening
                        .write(writer, FlatSeparator::Empty, Writer::soft_line);
                    writer.write_fragment(self.inner);
                });
                self.closing
                    .write(writer, FlatSeparator::Empty, Writer::soft_line);
                writer.text("]");
            });
        }
    }
}

struct ObjectPropertyBoundary<'ast> {
    suppression: PropertySuppression,
    before: &'ast [Comment],
    moved: &'ast [Comment],
    operator_line: &'ast [Comment],
    operator_line_start: u32,
    leading: &'ast [Comment],
    key_end: u32,
    colon: Span,
    value_start: u32,
    line_before_colon: bool,
    line_after_colon: bool,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(in crate::formatter) fn object_body_expands(&self, object: ObjectExpression) -> bool {
        let ast = self.ast;
        object.properties(ast).first().is_some_and(|id| {
            let property: ObjectPropertyKind = ast.get_node_in_sub_range(id);
            self.line_breaks(object.span(ast).start + 1, property.span(ast).start) != 0
        })
    }

    pub(super) fn format_object(
        &mut self,
        object: ObjectExpression,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        match self.source_object(object)? {
            SourceObject::Verbatim(span) => self.format_verbatim(span, writer),
            SourceObject::Formatted(source) => {
                let request = if source.expand {
                    GroupRequest::PreserveSource
                } else {
                    GroupRequest::Auto
                };
                writer.group_with_request(request, |writer| {
                    self.format_object_contents(source.contents, writer)
                })
            }
        }
    }

    fn source_object(
        &mut self,
        object: ObjectExpression,
    ) -> Result<SourceObject<'ast>, FormatError> {
        let ast = self.ast;
        let span = object.span(ast);
        let content = Span::new(span.start + 1, span.end - 1);
        let quoting = match self.options.quote_properties {
            QuoteProperties::AsNeeded => PropertyKeyQuoting::AsNeeded,
            QuoteProperties::Preserve => PropertyKeyQuoting::Preserve,
        };
        let mut remaining = object.properties(ast);
        let Some(first) = remaining.first().map(|id| ast.get_node_in_sub_range(id)) else {
            let comments = self.comments.between(content)?;
            if self.content_mode(comments) == ContentMode::Verbatim {
                return Ok(SourceObject::Verbatim(span));
            }
            let comments = self.comments.take(comments.len());
            return Ok(SourceObject::Formatted(FormattedObject {
                expand: false,
                contents: ObjectContents::Empty { content, comments },
            }));
        };
        let remaining = remaining.split_off(1);
        let expand = self.object_body_expands(object);
        Ok(SourceObject::Formatted(FormattedObject {
            expand,
            contents: ObjectContents::Properties {
                content,
                quoting,
                first,
                remaining,
            },
        }))
    }

    fn format_object_contents(
        &mut self,
        contents: ObjectContents<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
        match contents {
            ObjectContents::Empty { content, comments } => {
                writer.text("{");
                if !comments.is_empty() {
                    writer.indent(|writer| {
                        self.opening_line_constraint(content).write(
                            writer,
                            FlatSeparator::Empty,
                            Writer::soft_line,
                        );
                        let layout = if self.diagnostic_range_is_sensitive(content) {
                            CommentRunLayout::Source
                        } else {
                            CommentRunLayout::Lines
                        };
                        self.format_comment_run(comments, layout, writer)
                    })?;
                    let end = comments
                        .last()
                        .map_or(content.start, |comment| comment.span.end);
                    self.line_constraint(end, content.end).write(
                        writer,
                        FlatSeparator::Empty,
                        Writer::soft_line,
                    );
                    if comments
                        .iter()
                        .any(|comment| comment.kind == CommentKind::Line)
                    {
                        writer.break_parent();
                    }
                } else {
                    self.opening_line_constraint(content).write(
                        writer,
                        FlatSeparator::Empty,
                        |_| {},
                    );
                }
                writer.text("}");
                Ok(())
            }
            ObjectContents::Properties {
                content,
                quoting,
                first,
                remaining,
            } => {
                writer.text("{");
                let end = writer.indent(|writer| {
                    self.opening_line_constraint(content).write(
                        writer,
                        if self.options.bracket_spacing {
                            FlatSeparator::Space
                        } else {
                            FlatSeparator::Empty
                        },
                        |writer| self.object_brace_line(writer),
                    );
                    let leading = self
                        .comments
                        .between(Span::new(content.start, first.span(ast).start))?;
                    let mut leading = leading;
                    let mut following_mode = None;
                    let mut properties = remaining
                        .iter()
                        .map(|id| ast.get_node_in_sub_range(id))
                        .peekable();
                    let mut property = first;
                    loop {
                        let property_span = property.span(ast);
                        let terminal = properties.peek().is_none();
                        let next_start = properties
                            .peek()
                            .map_or(content.end, |next| next.span(ast).start);
                        let boundary = self
                            .object_boundary(Span::new(property_span.end, next_start), terminal);
                        let suppression = self.delimited_suppression(
                            property_span,
                            next_start,
                            if terminal {
                                DelimitedFollowing::End
                            } else {
                                DelimitedFollowing::Node
                            },
                        );
                        let leading_mode =
                            following_mode.unwrap_or_else(|| self.content_mode(leading));
                        let mode = if leading_mode == ContentMode::Verbatim
                            || suppression
                                .as_ref()
                                .is_some_and(|source| source.preceding.is_some())
                        {
                            ContentMode::Verbatim
                        } else {
                            ContentMode::Formatted
                        };
                        self.comments.claim(leading)?;
                        self.format_comment_run(leading, CommentRunLayout::Source, writer)?;
                        if let Some(last) = leading.last() {
                            self.comment_gap(last.span.end, property_span.start, writer);
                        }
                        let tail = if mode == ContentMode::Verbatim {
                            self.format_verbatim(property_span, writer)?;
                            None
                        } else {
                            match ast.object_property_kind_data(property) {
                                ObjectPropertyKindData::ObjectProperty(property) => {
                                    self.format_object_property(property, quoting, writer)?
                                }
                                ObjectPropertyKindData::SpreadElement(spread) => self
                                    .format_argument(
                                        Argument::SpreadElement(spread),
                                        mode,
                                        writer,
                                    )?,
                            }
                        };
                        let protected_gap = !terminal
                            && suppression
                                .as_ref()
                                .is_some_and(|source| source.preceding.is_some());
                        let (end, next_leading) = if protected_gap {
                            let content_end = tail
                                .as_ref()
                                .map_or(property_span.end, |tail| tail.source_end);
                            let source = self.format_delimited_source_gap(
                                Span::new(content_end, next_start),
                                DelimitedFollowing::Node,
                                writer,
                            )?;
                            source
                                .following
                                .write(writer, FlatSeparator::Space, Writer::line);
                            (source.end, &[][..])
                        } else {
                            self.format_expression_tail(tail, writer)?;
                            self.comments.claim(boundary.inline)?;
                            self.format_trailing_comment_run(boundary.inline, writer)?;
                            if !terminal {
                                writer.text(",");
                                let constraint = self.line_constraint(
                                    boundary.separator.start,
                                    boundary.separator.end,
                                );
                                constraint.write(writer, FlatSeparator::Space, Writer::line);
                                if constraint == LineConstraint::Unconstrained
                                    && self.next_line_is_empty(
                                        boundary.separator.start,
                                        boundary.separator.end,
                                    )
                                {
                                    writer.hard_line();
                                }
                            } else {
                                if self.options.trailing_commas != TrailingCommas::None {
                                    writer.if_break(",");
                                }
                                self.comments.claim(boundary.own_line)?;
                                if let Some(first) = boundary.own_line.first() {
                                    let constraint = self
                                        .line_constraint(boundary.trailing_end, first.span.start);
                                    constraint.write(
                                        writer,
                                        FlatSeparator::Space,
                                        Writer::hard_line,
                                    );
                                    if constraint == LineConstraint::Unconstrained
                                        && self.next_line_is_empty(
                                            boundary.trailing_end,
                                            first.span.start,
                                        )
                                    {
                                        writer.hard_line();
                                    }
                                    self.format_comment_run(
                                        boundary.own_line,
                                        if constraint != LineConstraint::Unconstrained {
                                            CommentRunLayout::Source
                                        } else if boundary.own_line.iter().all(|&comment| {
                                            self.comment_semantics(comment).is_ordinary()
                                        }) {
                                            CommentRunLayout::OwnLineTrailing
                                        } else {
                                            CommentRunLayout::Lines
                                        },
                                        writer,
                                    )?;
                                }
                            }
                            let end = boundary
                                .own_line
                                .last()
                                .map_or(boundary.trailing_end, |comment| comment.span.end);
                            (end, boundary.leading)
                        };
                        following_mode = suppression.map(|source| source.following);
                        leading = next_leading;
                        let Some(next) = properties.next() else {
                            return Ok::<_, FormatError>(end);
                        };
                        property = next;
                    }
                })?;
                self.line_constraint(end, content.end).write(
                    writer,
                    if self.options.bracket_spacing {
                        FlatSeparator::Space
                    } else {
                        FlatSeparator::Empty
                    },
                    |writer| self.object_brace_line(writer),
                );
                writer.text("}");
                Ok(())
            }
        }
    }

    fn object_boundary(&self, span: Span, terminal: bool) -> ObjectBoundary<'ast> {
        let comments = self.comments.in_range(span);
        let comma = self.find_token(span, comments, ",");
        let mut inline_end = span.start;
        let mut count = 0;
        for (index, comment) in comments.iter().enumerate() {
            if self.line_breaks(inline_end, comment.span.start) != 0 {
                break;
            }
            inline_end = comment.span.end;
            if comment.kind == CommentKind::Line {
                count = index + 1;
                break;
            }
            let following_start = comments
                .get(index + 1)
                .map_or(span.end, |next| next.span.start);
            if terminal
                || comma.is_none_or(|comma| comment.span.start < comma)
                || self.line_breaks(comment.span.end, following_start) != 0
            {
                count = index + 1;
            }
        }
        let (inline, remaining) = comments.split_at(count);
        let trailing_end = inline.last().map_or(span.start, |comment| comment.span.end);
        let next_start = remaining
            .first()
            .map_or(span.end, |comment| comment.span.start);
        // The next property's leading comment can precede the source comma.
        let separator_start = match comma {
            Some(comma) if comma < next_start => trailing_end.max(comma + 1),
            _ => trailing_end,
        };
        ObjectBoundary {
            inline,
            own_line: if terminal { remaining } else { &[] },
            leading: if terminal { &[] } else { remaining },
            trailing_end,
            separator: Span::new(separator_start, next_start),
        }
    }

    fn object_brace_line(&self, writer: &mut Writer<'ast>) {
        if self.options.bracket_spacing {
            writer.line();
        } else {
            writer.soft_line();
        }
    }

    fn format_object_property(
        &mut self,
        property: ObjectProperty,
        quoting: PropertyKeyQuoting,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let ast = self.ast;
        if property.method(ast) || property.kind(ast) != PropertyKind::Init {
            return Err(self.unsupported("object method", property));
        }
        let key_end = self.object_key_end(property)?;
        if property.shorthand(ast) {
            self.format_object_key(
                property,
                key_end,
                ContentMode::Formatted,
                PropertyKeyQuoting::Preserve,
                writer,
            )?;
            return Ok(None);
        }
        let boundary = self.object_property_boundary(property, key_end)?;
        if boundary.suppression.enclosing == ContentMode::Verbatim {
            self.format_verbatim(property.span(ast), writer)?;
            return Ok(None);
        }
        let value = property.value(ast);
        let content = self.expression_content(value);
        let before_colon = boundary
            .before
            .last()
            .map_or(boundary.key_end, |comment| comment.span.end);
        let before_colon_constraint = self.line_constraint(before_colon, boundary.colon.start);
        let prefix_start = boundary
            .leading
            .first()
            .map_or(boundary.value_start, |comment| comment.span.start);
        writer.group(false, |writer| {
            let clean_width = {
                let mut write_left = |writer: &mut Writer<'ast>| {
                    self.format_object_key(
                        property,
                        boundary.key_end,
                        boundary.suppression.key,
                        quoting,
                        writer,
                    )?;
                    self.comments.claim(boundary.before)?;
                    self.format_comment_run(
                        boundary.before,
                        CommentRunLayout::SourceAfter(boundary.key_end),
                        writer,
                    )?;
                    before_colon_constraint.write(writer, FlatSeparator::Empty, |writer| {
                        if boundary.line_before_colon {
                            writer.hard_line();
                        }
                    });
                    Ok::<_, FormatError>(())
                };
                if boundary.line_after_colon {
                    write_left(writer)?;
                    None
                } else {
                    writer.write_with_clean_text_width(write_left)?
                }
            };
            // A break is useful only when the key overlaps its indented value by three columns.
            let overlap = usize::from(self.options.indent_width.value()) + 3;
            let has_short_key = clean_width.is_some_and(|width| width < overlap);
            let layout = if boundary.line_after_colon {
                RhsLayout::BreakAfterOperator
            } else {
                let data = self.property_rhs_content(&content, boundary.suppression.value);
                let layout = self.property_rhs_layout(data, prefix_start, has_short_key);
                if matches!(layout, RhsLayout::Fluid)
                    && clean_width.is_some()
                    && matches!(
                        data,
                        ExprData::BooleanLiteral(_)
                            | ExprData::NumericLiteral(_)
                            | ExprData::TemplateLiteral(_)
                            | ExprData::TaggedTemplateExpression(_)
                            | ExprData::Class(_)
                    )
                {
                    RhsLayout::NeverBreakAfterOperator
                } else {
                    layout
                }
            };
            writer.text(":");
            self.comments.claim(boundary.operator_line)?;
            self.format_comment_run(
                boundary.operator_line,
                CommentRunLayout::SourceAfter(boundary.operator_line_start),
                writer,
            )?;
            let after_colon = boundary
                .operator_line
                .last()
                .map_or(boundary.colon.end, |comment| comment.span.end);
            let constraint = self.line_constraint(after_colon, prefix_start);
            let write_value = |writer: &mut Writer<'ast>| {
                self.comments.claim(boundary.moved)?;
                self.format_comment_run(boundary.moved, CommentRunLayout::Source, writer)?;
                if let Some(last) = boundary.moved.last() {
                    let next = boundary
                        .leading
                        .first()
                        .map_or(boundary.value_start, |comment| comment.span.start);
                    let before = self.line_breaks(last.span.end, boundary.colon.start);
                    let after = self.line_breaks(boundary.colon.end, next);
                    // Moving the colon does not introduce an empty source line.
                    match before.max(after) {
                        0 => writer.space(),
                        1 => writer.hard_line(),
                        _ => {
                            writer.hard_line();
                            writer.hard_line();
                        }
                    }
                }
                self.comments.claim(boundary.leading)?;
                let mut context = ExpressionContext::at(ExpressionPosition::PropertyValue)
                    .with_content_mode(boundary.suppression.value);
                if context.is_formatted_conditional_rhs(content.data) {
                    context = context.with_operand_indent(layout.operand_indent());
                }
                self.format_prepared_expression_with_leading_comments(
                    value,
                    content,
                    NodeHead {
                        comments: boundary.leading,
                        content_start: boundary.value_start,
                        layout: CommentRunLayout::GroupedLeading(boundary.value_start),
                        gap_end: None,
                    },
                    context,
                    writer,
                )
            };
            let separator = |writer: &mut Writer<'ast>| {
                constraint.write(writer, FlatSeparator::Space, |writer| {
                    if boundary.line_after_colon {
                        writer.hard_line();
                    } else if matches!(layout, RhsLayout::NeverBreakAfterOperator) {
                        writer.space();
                    } else {
                        writer.line();
                    }
                });
            };
            layout.write(writer, separator, write_value)
        })
    }

    fn object_property_boundary(
        &self,
        property: ObjectProperty,
        key_end: u32,
    ) -> Result<ObjectPropertyBoundary<'ast>, FormatError> {
        let value_start = property.value(self.ast).span(self.ast).start;
        let gap = Span::new(key_end, value_start);
        let comments = self.comments.in_range(gap);
        let Some(colon_start) = self.find_token(gap, comments, ":") else {
            return Err(self.unsupported("object property colon", property));
        };
        let colon = Span::new(colon_start, colon_start + 1);
        let count = comments.partition_point(|comment| comment.span.end <= colon.start);
        let (before, after) = comments.split_at(count);
        let key_trailing = if self.has_suppression && property.computed(self.ast) {
            self.comments.in_range(Span::new(
                property.key(self.ast).span(self.ast).end,
                colon.start,
            ))
        } else {
            before
        };
        let key_leading = (self.has_suppression && property.computed(self.ast)).then(|| {
            Span::new(
                property.span(self.ast).start + 1,
                property.key(self.ast).span(self.ast).start,
            )
        });
        let suppression =
            self.property_suppression(key_leading, key_trailing, after, colon, value_start);
        let ordinary_before = self
            .comment_run_binding(before.iter().copied(), |comment| {
                if self.comment_semantics(comment).is_ordinary() {
                    CommentBinding::Unbound
                } else {
                    CommentBinding::SourceBound
                }
            })
            .allows_relocation(None);
        let promoted = ordinary_before
            && before.first().is_some_and(|first| {
                self.line_breaks(key_end, first.span.start) != 0
                    || before.last().is_some_and(|last| {
                        self.line_breaks(last.span.start, last.span.end) != 0
                            && self.line_breaks(last.span.end, colon.start) != 0
                    })
            });
        let (before, moved) = if promoted {
            (&[][..], before)
        } else {
            (before, &[][..])
        };
        let line_before_colon = before
            .last()
            .is_some_and(|comment| comment.kind == CommentKind::Line);
        let mut count = 0;
        let mut previous = colon.end;
        for (index, comment) in after.iter().enumerate() {
            if promoted || self.line_breaks(previous, comment.span.start) != 0 {
                break;
            }
            if comment.kind == CommentKind::Line {
                count = index + 1;
                break;
            }
            previous = comment.span.end;
        }
        let (operator_line, leading) = after.split_at(count);
        let (before, operator_line, operator_line_start, leading) =
            if ordinary_before && line_before_colon {
                (&[][..], before, key_end, after)
            } else {
                (before, operator_line, colon.end, leading)
            };
        let line_before_colon = line_before_colon && !ordinary_before;
        let line_after_colon = promoted
            || !operator_line.is_empty()
            || leading.first().is_some_and(|comment| {
                comment.kind == CommentKind::Line
                    || self.line_breaks(colon.end, comment.span.start) != 0
            });
        Ok(ObjectPropertyBoundary {
            suppression,
            before,
            moved,
            operator_line,
            operator_line_start,
            leading,
            key_end,
            colon,
            value_start,
            line_before_colon,
            line_after_colon,
        })
    }

    pub(in crate::formatter) fn property_suppression(
        &self,
        key_leading: Option<Span>,
        before: &[Comment],
        after: &[Comment],
        colon: Span,
        value_start: u32,
    ) -> PropertySuppression {
        let mut suppression = PropertySuppression {
            key: ContentMode::Formatted,
            value: ContentMode::Formatted,
            enclosing: ContentMode::Formatted,
        };
        if let Some(span) = key_leading {
            let leading = self.comments.in_range(span);
            if self
                .suppression_positions(leading, span.end)
                .any(|position| matches!(position, SuppressionPosition::EndOfLine))
            {
                suppression.enclosing = ContentMode::Verbatim;
            }
        }
        for position in self.suppression_positions(before, colon.start) {
            match position {
                SuppressionPosition::OwnLine => suppression.value = ContentMode::Verbatim,
                SuppressionPosition::EndOfLine => suppression.enclosing = ContentMode::Verbatim,
                SuppressionPosition::Inline => suppression.key = ContentMode::Verbatim,
            }
        }
        for position in self.suppression_positions(after, value_start) {
            match position {
                SuppressionPosition::EndOfLine => suppression.enclosing = ContentMode::Verbatim,
                SuppressionPosition::OwnLine | SuppressionPosition::Inline => {
                    suppression.value = ContentMode::Verbatim;
                }
            }
        }
        suppression
    }

    fn object_key_end(&self, property: ObjectProperty) -> Result<u32, FormatError> {
        let key_end = property.key(self.ast).span(self.ast).end;
        if !property.computed(self.ast) {
            return Ok(key_end);
        }
        let gap = Span::new(key_end, property.value(self.ast).span(self.ast).start);
        let comments = self.comments.in_range(gap);
        self.find_token(gap, comments, "]")
            .map(|closing| closing + 1)
            .ok_or_else(|| self.unsupported("computed object key boundary", property))
    }

    fn format_object_key(
        &mut self,
        property: ObjectProperty,
        key_end: u32,
        mode: ContentMode,
        quoting: PropertyKeyQuoting,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let computed = property
            .computed(self.ast)
            .then_some(Span::new(property.span(self.ast).start, key_end));
        self.format_property_key(property.key(self.ast), computed, mode, quoting, writer)
    }

    pub(in crate::formatter) fn unquoted_string_key(
        &self,
        literal: swc_next_ecma_ast::StringLiteral,
        quoting: PropertyKeyQuoting,
    ) -> Option<&'ast str> {
        if quoting != PropertyKeyQuoting::AsNeeded {
            return None;
        }
        self.unquoted_string_key_content(literal)
    }

    fn unquoted_string_key_content(&self, literal: StringLiteral) -> Option<&'ast str> {
        let raw = self.ast.get_utf8(literal.raw(self.ast));
        let content = &raw[1..raw.len() - 1];
        (swc_next_ecma_ast::is_identifier_es5_and_esnext(content.as_bytes())
            || !self.ast.lang().is_typescript() && numeric_property_name(content))
        .then_some(content)
    }

    pub(in crate::formatter) fn format_property_key(
        &mut self,
        key: PropertyKey,
        computed: Option<Span>,
        mode: ContentMode,
        quoting: PropertyKeyQuoting,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
        let key_span = key.span(ast);
        if let Some(computed) = computed {
            let PropertyKeyData::Expr(expression) = ast.property_key_data(key) else {
                return Err(self.unsupported("computed object key", key));
            };
            self.prepare_computed_property_key(expression, computed, mode, writer)?
                .write(writer);
            return Ok(());
        }
        if mode == ContentMode::Verbatim {
            return self.format_verbatim(key_span, writer);
        }
        match ast.property_key_data(key) {
            PropertyKeyData::IdentifierName(identifier) => {
                writer.text(ast.get_utf8(identifier.name(ast)));
            }
            PropertyKeyData::Expr(expression) => match ast.expr_data(expression) {
                ExprData::StringLiteral(literal) => {
                    if let Some(content) = self.unquoted_string_key(literal, quoting) {
                        writer.text(content);
                    } else {
                        self.format_string_literal(literal, StringContext::Expression, writer);
                    }
                }
                ExprData::NumericLiteral(literal) => self.format_numeric_literal(literal, writer),
                ExprData::BigIntLiteral(literal) => self.format_bigint_literal(literal, writer),
                _ => return Err(self.unsupported("object property key", key)),
            },
        }
        Ok(())
    }

    fn prepare_computed_property_key(
        &mut self,
        expression: Expr,
        computed: Span,
        mode: ContentMode,
        writer: &mut Writer<'ast>,
    ) -> Result<PreparedComputedKey<'ast>, FormatError> {
        let key_span = expression.span(self.ast);
        let closing = computed.end - 1;
        let content = Span::new(computed.start + 1, closing);
        let opening_constraint = self.opening_line_constraint(content);
        let closing_constraint = if self.has_suppression {
            let comments = self.comments.in_range(Span::new(key_span.end, closing));
            if self.content_mode(comments) == ContentMode::Verbatim {
                comments
                    .last()
                    .map(|comment| self.source_line_constraint(comment.span.end, closing))
            } else {
                None
            }
        } else {
            None
        };
        let scope = if mode != ContentMode::Formatted {
            ComputedKeyScope::Expression
        } else if self.has_suppression
            || self.has_target_annotations
            || self.has_file_directives
            || !self.diagnostic_lines.is_empty()
        {
            let comments = self.ast.comments();
            let first = comments.partition_point(|comment| comment.span.start < content.start);
            let last = comments.partition_point(|comment| comment.span.end <= content.end);
            if self
                .comment_binding(&comments[first..last], content, None)
                .allows_relocation(None)
            {
                self.computed_key_scope(expression)
            } else {
                ComputedKeyScope::Expression
            }
        } else {
            self.computed_key_scope(expression)
        };
        let (inner, end) = writer.record_with_value(|writer| {
            let leading = self
                .comments
                .between(Span::new(computed.start + 1, key_span.start))?;
            let content = self.expression_content(expression);
            let binding = self.expression_head_binding(leading, expression);
            let binding = content
                .parentheses
                .map_or(binding, |removed| binding.combine(removed.head.binding));
            let mode = if mode == ContentMode::Verbatim {
                mode
            } else {
                self.content_mode(leading)
            };
            let operand_indent = if mode == ContentMode::Formatted
                && matches!(content.data, ExprData::ConditionalExpression(_))
            {
                OperandIndent::Parent
            } else {
                OperandIndent::Expression
            };
            let leading = self.comments.take(leading.len());
            let tail = self.format_prepared_expression_with_resolved_head(
                expression,
                content,
                ResolvedExpressionHead {
                    head: NodeHead {
                        comments: leading,
                        content_start: key_span.start,
                        layout: CommentRunLayout::GroupedLeading(key_span.start),
                        gap_end: None,
                    },
                    binding,
                },
                ExpressionContext::at(ExpressionPosition::Nested)
                    .with_operand_indent(operand_indent)
                    .with_content_mode(mode),
                writer,
            )?;
            let tail = self.capture_expression_tail(Span::new(key_span.end, closing), tail)?;
            let end = tail
                .as_ref()
                .and_then(|tail| tail.comments.last())
                .map_or(key_span.end, |comment| comment.span.end);
            self.format_expression_tail(tail, writer)?;
            Ok::<_, FormatError>(end)
        })?;
        let closing_constraint =
            closing_constraint.unwrap_or_else(|| self.line_constraint(end, closing));
        Ok(PreparedComputedKey {
            inner,
            scope,
            opening: opening_constraint,
            closing: closing_constraint,
        })
    }

    fn computed_key_scope(&self, expression: Expr) -> ComputedKeyScope {
        let mut data = self.expression_content(expression).data;
        loop {
            data = match data {
                ExprData::UnaryExpression(unary) => {
                    self.expression_content(unary.argument(self.ast)).data
                }
                ExprData::AwaitExpression(node) => {
                    self.expression_content(node.argument(self.ast)).data
                }
                ExprData::YieldExpression(node) => {
                    let Some(argument) = node.argument(self.ast) else {
                        return ComputedKeyScope::Text;
                    };
                    self.expression_content(argument).data
                }
                ExprData::UpdateExpression(update) => {
                    match self
                        .ast
                        .simple_assignment_target_data(update.argument(self.ast))
                    {
                        SimpleAssignmentTargetData::IdentifierReference(identifier) => {
                            ExprData::IdentifierReference(identifier)
                        }
                        _ => return ComputedKeyScope::Expression,
                    }
                }
                ExprData::IdentifierReference(_)
                | ExprData::ThisExpression(_)
                | ExprData::MetaProperty(_)
                | ExprData::StringLiteral(_)
                | ExprData::NumericLiteral(_)
                | ExprData::BigIntLiteral(_)
                | ExprData::BooleanLiteral(_)
                | ExprData::NullLiteral(_)
                | ExprData::RegExpLiteral(_) => return ComputedKeyScope::Text,
                // Postfix, template, and compound expressions own their layout scope.
                _ => return ComputedKeyScope::Expression,
            };
        }
    }
}

fn numeric_property_name(content: &str) -> bool {
    let digits = |part: &str| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit());
    let (integer, fraction) = content
        .split_once('.')
        .map_or((content, None), |(integer, fraction)| {
            (integer, Some(fraction))
        });
    if !digits(integer) || fraction.is_some_and(|fraction| !digits(fraction)) {
        return false;
    }
    // Numeric keys must round-trip through ECMAScript's number-to-string conversion.
    content
        .parse::<f64>()
        .is_ok_and(|number| zmij_ecma::Buffer::new().format(number) == content)
}
