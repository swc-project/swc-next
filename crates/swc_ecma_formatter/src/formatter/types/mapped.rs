//! Mapped types own complete key headers and source-bound punctuation gaps.

use swc_next_ecma_ast::{Comment, GetSpan, Span, TsMappedType, TsMappedTypeModifier};

use super::{Formatter, PreparedType, TypeBoundaryLayout, TypeContext, TypeLayout};
use crate::formatter::comments::{
    CommentBinding, CommentBoundary, CommentGap, CommentRunLayout, ExpressionTail,
};
use crate::formatter::diagnostics::FlatSeparator;
use crate::formatter::suppression::{ContentMode, SuppressionPosition};
use crate::{FormatError, Semicolons, document::Writer};

struct MappedToken<'ast> {
    span: Span,
    text: &'static str,
    leading: CommentBoundary<'ast>,
}

struct MappedModifier<'ast> {
    sign: Option<MappedToken<'ast>>,
    keyword: MappedToken<'ast>,
}

struct MappedHead<'ast> {
    comments: &'ast [Comment],
    breaks_after_last: bool,
}

#[derive(Clone, Copy)]
enum MappedSeparator {
    In,
    As,
    Colon,
}

impl MappedSeparator {
    fn text(self) -> &'static str {
        match self {
            Self::In => "in",
            Self::As => "as",
            Self::Colon => ":",
        }
    }
}

struct MappedBoundary<'ast> {
    left_mode: ContentMode,
    layout: TypeBoundaryLayout<'ast>,
}

struct MappedBound<'ast> {
    node: PreparedType,
    boundary: MappedBoundary<'ast>,
}

enum MappedSuffix<'ast> {
    Source {
        closing: Span,
        remaining: Span,
    },
    Formatted {
        closing: MappedToken<'ast>,
        optional: Option<MappedModifier<'ast>>,
        annotation: Option<(MappedToken<'ast>, CommentBoundary<'ast>)>,
    },
}

enum MappedTail<'ast> {
    Source(Span),
    Formatted {
        before: CommentBoundary<'ast>,
        after: Option<CommentBoundary<'ast>>,
    },
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_mapped_type(
        &mut self,
        node: TsMappedType,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let ast = self.ast;
        let span = node.span(ast);
        let content = Span::new(span.start + 1, span.end - 1);
        let key = node.key(ast);
        let key_span = key.span(ast);
        let prefix_span = Span::new(content.start, key_span.start);
        if self.has_suppression
            && self.content_mode(self.comments.in_range(prefix_span)) == ContentMode::Verbatim
        {
            // These leading runs select this real mapped node, including its body.
            self.format_verbatim(span, writer)?;
            return Ok(None);
        }
        let readonly = self.prepare_mapped_modifier(prefix_span, node.readonly(ast), "readonly")?;
        let opening_start = readonly
            .as_ref()
            .map_or(content.start, |modifier| modifier.keyword.span.end);
        let opening = self.prepare_mapped_token(Span::new(opening_start, key_span.start), "[")?;
        let after_open = self.prepare_comment_boundary(
            self.comments
                .in_range(Span::new(opening.span.end, key_span.start)),
            opening.span.end,
            key_span.start,
        );
        let first_token = readonly.as_ref().map_or(opening.span.start, |modifier| {
            modifier
                .sign
                .as_ref()
                .map_or(modifier.keyword.span.start, |sign| sign.span.start)
        });
        let head = self.prepare_mapped_head(prefix_span, readonly.as_ref(), opening.span);
        let key_head_binding = self.comment_boundary_binding(&after_open, key_span);
        let expand = self.type_body_expands(content.start, first_token);
        let mut constraint = self.prepare_type(node.constraint(ast), ContentMode::Formatted);
        let in_boundary = self.prepare_mapped_boundary(
            Span::new(key_span.end, constraint.node.span(ast).start),
            MappedSeparator::In,
            &mut constraint,
        )?;
        let key_mode = in_boundary.left_mode;
        let mut remap = if let Some(type_node) = node.name_type(ast) {
            let mut type_node = self.prepare_type(type_node, ContentMode::Formatted);
            let boundary = self.prepare_mapped_boundary(
                Span::new(
                    constraint.node.span(ast).end,
                    type_node.node.span(ast).start,
                ),
                MappedSeparator::As,
                &mut type_node,
            )?;
            if boundary.left_mode == ContentMode::Verbatim {
                constraint.source_owner = Some(constraint.node);
            }
            Some(MappedBound {
                node: type_node,
                boundary,
            })
        } else {
            None
        };
        let mut value = node
            .type_annotation(ast)
            .map(|type_node| self.prepare_type(type_node, ContentMode::Formatted));
        let last_header = remap
            .as_mut()
            .map_or(&mut constraint, |bound| &mut bound.node);
        let (suffix, tail) =
            self.prepare_mapped_suffix(node, last_header, &mut value, content.end)?;
        writer.group(expand, |writer| {
            writer.text("{");
            let closing_gap = writer.indent(|writer| {
                if let Some(head) = head {
                    self.mapped_body_line(writer);
                    self.format_mapped_head(head, writer)?;
                    if let Some(readonly) = readonly {
                        if let Some(sign) = readonly.sign {
                            writer.text(sign.text);
                        }
                        writer.text(readonly.keyword.text);
                        writer.space();
                    }
                } else {
                    if let Some(readonly) = readonly {
                        self.format_mapped_modifier(readonly, true, writer)?;
                    }
                    self.format_comment_boundary(&opening.leading, writer)?;
                    opening
                        .leading
                        .gap()
                        .write(writer, FlatSeparator::Space, |writer| {
                            if node.readonly(ast) == TsMappedTypeModifier::None {
                                self.mapped_body_line(writer);
                            } else {
                                writer.space();
                            }
                        });
                }
                writer.group(false, |writer| {
                    writer.text("[");
                    writer.indent(|writer| {
                        if matches!(key_head_binding, CommentBinding::Unbound) {
                            writer.soft_line();
                            self.format_comment_boundary_with_layout(
                                &after_open,
                                CommentRunLayout::FlatLeading(key_span.start),
                                writer,
                            )?;
                        } else {
                            self.format_comment_boundary(&after_open, writer)?;
                            after_open
                                .gap()
                                .write(writer, FlatSeparator::Empty, Writer::soft_line);
                        }
                        if key_mode == ContentMode::Verbatim {
                            self.format_verbatim(key_span, writer)?;
                        } else {
                            writer.text(ast.get_utf8(key.name(ast)));
                        }
                        self.format_mapped_boundary(&in_boundary, MappedSeparator::In, writer)?;
                        let tail =
                            self.format_prepared_type(constraint, TypeContext::Root, writer)?;
                        self.format_expression_tail(tail, writer)?;
                        if let Some(remap) = remap {
                            self.format_mapped_boundary(
                                &remap.boundary,
                                MappedSeparator::As,
                                writer,
                            )?;
                            let tail =
                                self.format_prepared_type(remap.node, TypeContext::Root, writer)?;
                            self.format_expression_tail(tail, writer)?;
                        }
                        if let MappedSuffix::Formatted { closing, .. } = &suffix {
                            self.format_comment_boundary(&closing.leading, writer)?;
                        }
                        Ok::<_, FormatError>(())
                    })?;
                    match &suffix {
                        MappedSuffix::Source { closing, .. } => {
                            self.format_verbatim(*closing, writer)
                        }
                        MappedSuffix::Formatted { closing, .. } => {
                            closing.leading.gap().write(
                                writer,
                                FlatSeparator::Empty,
                                Writer::soft_line,
                            );
                            writer.text("]");
                            Ok(())
                        }
                    }
                })?;
                let (value_layout, annotation_after) = match suffix {
                    MappedSuffix::Source { remaining, .. } => {
                        self.format_verbatim(remaining, writer)?;
                        (TypeLayout::OpeningProvided, None)
                    }
                    MappedSuffix::Formatted {
                        optional,
                        annotation,
                        ..
                    } => {
                        if let Some(optional) = optional {
                            self.format_mapped_modifier(optional, false, writer)?;
                        }
                        if let Some((colon, after)) = annotation {
                            self.format_mapped_token(colon, FlatSeparator::Empty, writer)?;
                            (TypeLayout::Root, Some(after))
                        } else {
                            (TypeLayout::Root, None)
                        }
                    }
                };
                if let Some(value) = value {
                    let tail = match annotation_after {
                        Some(after) => self.format_prepared_type_after_comments(
                            value,
                            TypeContext::Root,
                            &after,
                            value_layout,
                            writer,
                        )?,
                        None => self.format_prepared_type_with_layout(
                            value,
                            TypeContext::Root,
                            value_layout,
                            writer,
                        )?,
                    };
                    self.format_expression_tail(tail, writer)?;
                }
                if let Some(tail) = tail {
                    self.format_mapped_tail(tail, writer)
                } else {
                    Ok(None)
                }
            })?;
            if let Some(gap) = closing_gap {
                gap.write(writer, FlatSeparator::Space, |writer| {
                    self.mapped_body_line(writer)
                });
            }
            writer.text("}");
            Ok(None)
        })
    }

    fn prepare_mapped_token(
        &self,
        gap: Span,
        text: &'static str,
    ) -> Result<MappedToken<'ast>, FormatError> {
        let comments = self.comments.in_range(gap);
        let Some(start) = self.find_token(gap, comments, text) else {
            return Err(FormatError::unsupported(
                "mapped type punctuation",
                None,
                gap,
            ));
        };
        let span = Span::new(start, start + text.len() as u32);
        let leading = self.prepare_comment_boundary(
            self.comments.in_range(Span::new(gap.start, start)),
            gap.start,
            start,
        );
        Ok(MappedToken {
            span,
            text,
            leading,
        })
    }

    fn prepare_mapped_head(
        &self,
        span: Span,
        readonly: Option<&MappedModifier<'ast>>,
        opening: Span,
    ) -> Option<MappedHead<'ast>> {
        let span = Span::new(span.start, opening.start);
        let comments = self.comments.in_range(span);
        if !matches!(
            self.comment_binding(comments, span, None),
            CommentBinding::Unbound
        ) {
            return None;
        }
        let breaks_after_last = comments.last().is_some_and(|comment| {
            let next = readonly.map_or(opening.start, |modifier| {
                modifier
                    .sign
                    .as_ref()
                    .map(|sign| sign.span.start)
                    .into_iter()
                    .chain([modifier.keyword.span.start, opening.start])
                    .find(|&start| start >= comment.span.end)
                    .unwrap_or(opening.start)
            });
            comment.kind == swc_next_ecma_ast::CommentKind::Line
                || self.line_breaks(comment.span.end, next) != 0
        });
        Some(MappedHead {
            comments,
            breaks_after_last,
        })
    }

    fn format_mapped_head(
        &mut self,
        head: MappedHead<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        self.comments.claim(head.comments)?;
        let Some((last, rest)) = head.comments.split_last() else {
            return Ok(());
        };
        self.format_comment_run(rest, CommentRunLayout::CanonicalDangling, writer)?;
        if !rest.is_empty() {
            writer.hard_line();
        }
        let last = std::slice::from_ref(last);
        if head.breaks_after_last {
            self.format_comment_run(last, CommentRunLayout::Source, writer)?;
            writer.hard_line();
            Ok(())
        } else {
            writer.group(false, |writer| {
                self.format_comment_run(last, CommentRunLayout::Source, writer)?;
                writer.line();
                Ok(())
            })
        }
    }

    fn prepare_mapped_modifier(
        &self,
        gap: Span,
        modifier: TsMappedTypeModifier,
        keyword: &'static str,
    ) -> Result<Option<MappedModifier<'ast>>, FormatError> {
        let sign = match modifier {
            TsMappedTypeModifier::None => return Ok(None),
            TsMappedTypeModifier::True => None,
            TsMappedTypeModifier::Plus => Some(self.prepare_mapped_token(gap, "+")?),
            TsMappedTypeModifier::Minus => Some(self.prepare_mapped_token(gap, "-")?),
        };
        let start = sign.as_ref().map_or(gap.start, |sign| sign.span.end);
        let keyword = self.prepare_mapped_token(Span::new(start, gap.end), keyword)?;
        Ok(Some(MappedModifier { sign, keyword }))
    }

    fn prepare_mapped_boundary(
        &self,
        gap: Span,
        separator: MappedSeparator,
        right: &mut PreparedType,
    ) -> Result<MappedBoundary<'ast>, FormatError> {
        let comments = self.comments.in_range(gap);
        let text = separator.text();
        let Some(start) = self.find_token(gap, comments, text) else {
            return Err(FormatError::unsupported("mapped type boundary", None, gap));
        };
        let operator = Span::new(start, start + text.len() as u32);
        let split = comments.partition_point(|comment| comment.span.end <= start);
        let (before, after) = comments.split_at(split);
        let left_mode =
            if self.has_suppression && self.content_mode(comments) == ContentMode::Verbatim {
                self.mapped_suppression(gap, comments, operator, right)
            } else {
                ContentMode::Formatted
            };
        let layout = if left_mode == ContentMode::Verbatim || right.source_owner.is_some() {
            TypeBoundaryLayout::Source(gap)
        } else {
            TypeBoundaryLayout::Formatted {
                before: self.prepare_comment_boundary(before, gap.start, operator.start),
                after: self.prepare_comment_boundary(after, operator.end, gap.end),
            }
        };
        Ok(MappedBoundary { left_mode, layout })
    }

    fn mapped_suppression(
        &self,
        gap: Span,
        comments: &'ast [Comment],
        operator: Span,
        right: &mut PreparedType,
    ) -> ContentMode {
        let split = comments.partition_point(|comment| comment.span.end <= operator.start);
        let (before, after) = comments.split_at(split);
        let mut left_mode = ContentMode::Formatted;
        for (_, position) in self.suppressed_comment_runs(before, operator.start) {
            if matches!(position, SuppressionPosition::OwnLine) {
                self.select_type_head(right, position);
            } else {
                left_mode = ContentMode::Verbatim;
            }
        }
        for (_, position) in self.suppressed_comment_runs(after, gap.end) {
            // After the member separator, the following type owns the directive.
            self.select_type_head(right, position);
        }
        left_mode
    }

    fn prepare_mapped_suffix(
        &self,
        node: TsMappedType,
        header: &mut PreparedType,
        value: &mut Option<PreparedType>,
        content_end: u32,
    ) -> Result<(MappedSuffix<'ast>, Option<MappedTail<'ast>>), FormatError> {
        let ast = self.ast;
        let header_end = header.node.span(ast).end;
        let end = value
            .as_ref()
            .map_or(content_end, |value| value.node.span(ast).start);
        let span = Span::new(header_end, end);
        let comments = self.comments.in_range(span);
        let Some(closing_start) = self.find_token(span, comments, "]") else {
            return Err(FormatError::unsupported(
                "mapped type closing bracket",
                None,
                span,
            ));
        };
        let closing_span = Span::new(closing_start, closing_start + 1);
        let source = self.has_suppression && self.content_mode(comments) == ContentMode::Verbatim;
        let colon = if let Some(value) = value {
            let colon_gap = Span::new(closing_span.end, end);
            let Some(colon_start) =
                self.find_token(colon_gap, comments, MappedSeparator::Colon.text())
            else {
                return Err(FormatError::unsupported(
                    "mapped type annotation colon",
                    None,
                    colon_gap,
                ));
            };
            let colon = Span::new(colon_start, colon_start + 1);
            if source {
                if self.mapped_suppression(span, comments, colon, value) == ContentMode::Verbatim {
                    header.source_owner = Some(header.node);
                }
                let tail = self
                    .prepare_mapped_tail(Span::new(value.node.span(ast).end, content_end), value);
                return Ok((
                    MappedSuffix::Source {
                        closing: Span::new(header_end, closing_span.end),
                        remaining: Span::new(closing_span.end, end),
                    },
                    Some(tail),
                ));
            }
            Some(colon)
        } else {
            if source {
                header.source_owner = Some(header.node);
                return Ok((
                    MappedSuffix::Source {
                        closing: Span::new(header_end, closing_span.end),
                        remaining: Span::new(closing_span.end, end),
                    },
                    None,
                ));
            }
            None
        };
        let mut closing = MappedToken {
            span: closing_span,
            text: "]",
            leading: self.prepare_comment_boundary(
                self.comments
                    .in_range(Span::new(header_end, closing_span.start)),
                header_end,
                closing_span.start,
            ),
        };
        let modifier_end = colon.as_ref().map_or(end, |colon| colon.start);
        let mut optional = self.prepare_mapped_modifier(
            Span::new(closing_span.end, modifier_end),
            node.optional(ast),
            "?",
        )?;
        let suffix_end = optional
            .as_ref()
            .map_or(closing_span.end, |optional| optional.keyword.span.end);
        let key_tail_span = Span::new(header_end, modifier_end);
        let key_tail_comments = self.comments.in_range(key_tail_span);
        let relocate_key_tail = matches!(
            self.comment_binding(key_tail_comments, key_tail_span, None),
            CommentBinding::Unbound
        );
        if relocate_key_tail {
            let next = key_tail_comments
                .last()
                .map_or(closing_span.start, |comment| {
                    if comment.span.end <= closing_span.start {
                        closing_span.start
                    } else {
                        modifier_end
                    }
                });
            closing.leading = self.prepare_comment_boundary(key_tail_comments, header_end, next);
            if let Some(optional) = &mut optional {
                if let Some(sign) = &mut optional.sign {
                    sign.leading =
                        self.prepare_comment_boundary(&[], sign.span.start, sign.span.start);
                }
                optional.keyword.leading = self.prepare_comment_boundary(
                    &[],
                    optional.keyword.span.start,
                    optional.keyword.span.start,
                );
            }
        }
        let annotation = colon.map(|colon| {
            let leading_start = if relocate_key_tail {
                colon.start
            } else {
                suffix_end
            };
            let leading = self.prepare_comment_boundary(
                self.comments
                    .in_range(Span::new(leading_start, colon.start)),
                leading_start,
                colon.start,
            );
            let after = self.prepare_comment_boundary(
                self.comments.in_range(Span::new(colon.end, end)),
                colon.end,
                end,
            );
            (
                MappedToken {
                    span: colon,
                    text: ":",
                    leading,
                },
                after,
            )
        });
        let tail = if let Some(value) = value {
            self.prepare_mapped_tail(Span::new(value.node.span(ast).end, content_end), value)
        } else {
            let owned_end = if relocate_key_tail {
                key_tail_comments
                    .last()
                    .map_or(suffix_end, |comment| suffix_end.max(comment.span.end))
            } else {
                suffix_end
            };
            self.prepare_mapped_tail(Span::new(owned_end, content_end), header)
        };
        Ok((
            MappedSuffix::Formatted {
                closing,
                optional,
                annotation,
            },
            Some(tail),
        ))
    }

    fn prepare_mapped_tail(&self, span: Span, owner: &mut PreparedType) -> MappedTail<'ast> {
        let comments = self.comments.in_range(span);
        if self.has_suppression && self.content_mode(comments) == ContentMode::Verbatim {
            owner.source_owner = Some(owner.node);
            return MappedTail::Source(span);
        }
        let terminator = self
            .find_token(span, comments, ";")
            .or_else(|| self.find_token(span, comments, ","));
        let Some(start) = terminator else {
            return MappedTail::Formatted {
                before: self.prepare_comment_boundary(comments, span.start, span.end),
                after: None,
            };
        };
        let split = comments.partition_point(|comment| comment.span.end <= start);
        let (before, after) = comments.split_at(split);
        MappedTail::Formatted {
            before: self.prepare_comment_boundary(before, span.start, start),
            after: Some(self.prepare_comment_boundary(after, start + 1, span.end)),
        }
    }

    fn format_mapped_boundary(
        &mut self,
        boundary: &MappedBoundary<'ast>,
        separator: MappedSeparator,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        match &boundary.layout {
            TypeBoundaryLayout::Source(span) => self.format_verbatim(*span, writer),
            TypeBoundaryLayout::Formatted { before, after } => {
                self.format_type_comment_gap(before, FlatSeparator::Space, writer)?;
                writer.text(separator.text());
                self.format_type_comment_gap(after, FlatSeparator::Space, writer)
            }
        }
    }

    fn format_mapped_token(
        &mut self,
        token: MappedToken<'ast>,
        separator: FlatSeparator,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        self.format_type_comment_gap(&token.leading, separator, writer)?;
        writer.text(token.text);
        Ok(())
    }

    fn format_mapped_modifier(
        &mut self,
        modifier: MappedModifier<'ast>,
        body_start: bool,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let first = modifier
            .sign
            .as_ref()
            .map_or(&modifier.keyword, |sign| sign);
        self.format_comment_boundary(&first.leading, writer)?;
        first
            .leading
            .gap()
            .write(writer, FlatSeparator::Empty, |writer| {
                if body_start {
                    self.mapped_body_line(writer);
                }
            });
        if let Some(sign) = modifier.sign {
            writer.text(sign.text);
            self.format_mapped_token(modifier.keyword, FlatSeparator::Empty, writer)
        } else {
            writer.text(modifier.keyword.text);
            Ok(())
        }
    }

    fn format_mapped_tail(
        &mut self,
        tail: MappedTail<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<CommentGap>, FormatError> {
        let (before, after) = match tail {
            MappedTail::Source(span) => {
                self.format_verbatim(span, writer)?;
                return Ok(None);
            }
            MappedTail::Formatted { before, after } => (before, after),
        };
        self.format_comment_boundary(&before, writer)?;
        if after.is_some() {
            before.gap().write(writer, FlatSeparator::Empty, |_| {});
        }
        if self.options.semicolons == Semicolons::Always {
            writer.if_break(";");
        }
        if let Some(after) = after {
            self.format_comment_boundary(&after, writer)?;
            Ok(Some(after.gap()))
        } else {
            Ok(Some(before.gap()))
        }
    }

    fn mapped_body_line(&self, writer: &mut Writer<'ast>) {
        if self.options.bracket_spacing {
            writer.line();
        } else {
            writer.soft_line();
        }
    }
}
