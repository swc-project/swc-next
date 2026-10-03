//! Try blocks, catch bindings, and finally boundaries retain distinct owners.
use super::{CompoundBlock, CompoundGap, Formatter};
use crate::formatter::comments::CommentRunLayout;
use crate::formatter::diagnostics::FlatSeparator;
use crate::formatter::statements::{EmptyBlock, StatementBoundary};
use crate::formatter::suppression::{ContentMode, SuppressionPosition};
use crate::{FormatError, document::Writer};
use swc_next_ecma_ast::{
    BindingPattern, CatchClause, Comment, CommentKind, GetSpan, Span, TryStatement,
};

struct PlainCatchBinding<'ast> {
    pattern: BindingPattern,
    mode: ContentMode,
    opening: u32,
    closing: u32,
    head: CompoundGap<'ast>,
    tail: CompoundGap<'ast>,
    before_opening: &'ast [Comment],
    leading: &'ast [Comment],
    expanded: bool,
}

struct PreparedCatch<'ast> {
    node: CatchClause,
    head: Option<CatchHead<'ast>>,
}

struct FinallyBoundary<'ast> {
    keyword: Span,
    before: CompoundGap<'ast>,
    after: CompoundGap<'ast>,
    packet: CompoundGap<'ast>,
}

struct CatchHead<'ast> {
    binding: Option<PlainCatchBinding<'ast>>,
    body_head: CompoundGap<'ast>,
    body: CompoundBlock<'ast>,
    leading_before_parameter: bool,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(in crate::formatter::statements) fn format_try_statement(
        &mut self,
        node: TryStatement,
        writer: &mut Writer<'ast>,
    ) -> Result<StatementBoundary, FormatError> {
        let ast = self.ast;
        let span = node.span(ast);
        let mut body = CompoundBlock::new(node.block(ast), EmptyBlock::Expanded);
        let mut head = self.compound_gap(Span::new(span.start + 3, body.node.span(ast).start));
        self.prepare_block_head(head, &mut body);
        self.transfer_block_leading(&mut head, &mut body);
        let handler = node.handler(ast);
        let mut finalizer = node
            .finalizer(ast)
            .map(|body| CompoundBlock::new(body, EmptyBlock::Expanded));
        let mut catch_gap = handler.map(|handler| {
            self.compound_gap(Span::new(body.node.span(ast).end, handler.span(ast).start))
        });
        let mut handler_mode = ContentMode::Formatted;
        let mut handler_first = ContentMode::Formatted;
        if let Some(gap) = catch_gap
            && gap.has_suppression()
        {
            for position in self.suppression_positions(gap.comments, gap.span.end) {
                match position {
                    SuppressionPosition::Inline => handler_mode = ContentMode::Verbatim,
                    SuppressionPosition::EndOfLine | SuppressionPosition::OwnLine => {
                        handler_first = ContentMode::Verbatim
                    }
                }
            }
        }
        let mut finally_head = if let Some(finalizer) = finalizer.as_ref() {
            let start = handler.map_or(body.node.span(ast).end, |handler| handler.span(ast).end);
            let limit = Span::new(start, finalizer.node.span(ast).start);
            let packet = self.compound_gap(limit);
            let comments = packet.comments;
            let Some(keyword) = self.find_token(limit, comments, "finally") else {
                return Err(FormatError::unsupported(
                    "finally keyword boundary",
                    None,
                    limit,
                ));
            };
            let keyword = Span::new(keyword, keyword + 7);
            Some(FinallyBoundary {
                keyword,
                before: self.compound_gap(Span::new(start, keyword.start)),
                after: self.compound_gap(Span::new(keyword.end, limit.end)),
                packet,
            })
        } else {
            None
        };
        if let (Some(boundary), Some(finalizer)) = (finally_head.as_mut(), finalizer.as_mut()) {
            if boundary.before.has_suppression() {
                for position in
                    self.suppression_positions(boundary.before.comments, boundary.before.span.end)
                {
                    match position {
                        SuppressionPosition::Inline => {
                            if handler.is_some() {
                                handler_mode = ContentMode::Verbatim;
                            } else {
                                body.mode = ContentMode::Verbatim;
                            }
                        }
                        SuppressionPosition::EndOfLine | SuppressionPosition::OwnLine => {
                            self.protect_block_first(finalizer)
                        }
                    }
                }
            }
            self.prepare_block_head(boundary.after, finalizer);
            if boundary.after.comments.is_empty() {
                self.transfer_block_leading(&mut boundary.before, finalizer);
            } else if self.block_leading_start(boundary.before) == Some(0)
                && self.block_leading_start(boundary.after) == Some(0)
                && self.transfer_block_leading(&mut boundary.packet, finalizer)
            {
                boundary.before.comments = &[];
                boundary.after.comments = &[];
            } else {
                // A later header packet cannot be overtaken by an earlier body packet.
                self.transfer_block_leading(&mut boundary.after, finalizer);
            }
        }
        let mut handler = handler
            .map(|handler| {
                self.prepare_catch(handler, handler_mode, handler_first, finalizer.is_some())
            })
            .transpose()?;
        if let (Some(gap), Some(handler)) = (catch_gap.as_mut(), handler.as_mut())
            && let Some(head) = handler.head.as_mut()
            // A header comment cannot be overtaken by the earlier body packet.
            && self.comments.in_range(Span::new(handler.node.span(ast).start, head.body.node.span(ast).start)).is_empty()
            && self.transfer_block_leading(gap, &mut head.body)
        {
            head.leading_before_parameter = true;
        }
        writer.text("try");
        self.format_compound_gap(head, FlatSeparator::Space, Writer::space, writer)?;
        self.comments.claim(body.leading.comments())?;
        self.format_compound_block_part(body, writer)?;
        if let (Some(gap), Some(handler)) = (catch_gap, handler) {
            self.format_compound_gap(gap, FlatSeparator::Space, Writer::space, writer)?;
            self.format_catch(handler, writer)?;
        }
        if let (Some(boundary), Some(finalizer)) = (finally_head, finalizer) {
            self.format_compound_gap(boundary.before, FlatSeparator::Space, Writer::space, writer)?;
            writer.text(self.source(boundary.keyword));
            self.format_compound_gap(boundary.after, FlatSeparator::Space, Writer::space, writer)?;
            self.comments.claim(finalizer.leading.comments())?;
            self.format_compound_block_part(finalizer, writer)?;
        }
        Ok(StatementBoundary::SourceEnd {
            end: span.end,
            has_terminator: false,
        })
    }

    fn prepare_catch(
        &self,
        node: CatchClause,
        mode: ContentMode,
        incoming_first: ContentMode,
        has_finalizer: bool,
    ) -> Result<PreparedCatch<'ast>, FormatError> {
        if mode == ContentMode::Verbatim {
            return Ok(PreparedCatch { node, head: None });
        }
        let ast = self.ast;
        let keyword_end = node.span(ast).start + 5;
        let mut body = CompoundBlock::new(
            node.body(ast),
            if has_finalizer {
                EmptyBlock::Expanded
            } else {
                EmptyBlock::Compact
            },
        );
        let body_start = body.node.span(ast).start;
        let binding = if let Some(pattern) = node.param(ast) {
            if node.type_annotation(ast).is_some() {
                return Err(self.unsupported("typed catch binding", node));
            }
            let span = pattern.span(ast);
            let head = self.compound_gap(Span::new(keyword_end, span.start));
            let tail = self.compound_gap(Span::new(span.end, body_start));
            let Some(opening) = self.find_token(head.span, head.comments, "(") else {
                return Err(FormatError::unsupported(
                    "catch opening boundary",
                    None,
                    head.span,
                ));
            };
            let Some(closing) = self.find_token(tail.span, tail.comments, ")") else {
                return Err(FormatError::unsupported(
                    "catch closing boundary",
                    None,
                    tail.span,
                ));
            };
            let mut mode = ContentMode::Formatted;
            if self.has_suppression {
                let before_close = self.comments.in_range(Span::new(span.end, closing));
                if head.has_suppression()
                    || self.content_mode(before_close) == ContentMode::Verbatim
                {
                    mode = ContentMode::Verbatim;
                }
                let after_close = self.comments.in_range(Span::new(closing + 1, body_start));
                for position in self.suppression_positions(after_close, body_start) {
                    match position {
                        SuppressionPosition::Inline => body.mode = ContentMode::Verbatim,
                        SuppressionPosition::EndOfLine | SuppressionPosition::OwnLine => {
                            mode = ContentMode::Verbatim
                        }
                    }
                }
            }
            let head_split = head
                .comments
                .partition_point(|comment| comment.span.end <= opening);
            let before_opening = &head.comments[..head_split];
            let leading = &head.comments[head_split..];
            let tail_split = tail
                .comments
                .partition_point(|comment| comment.span.end <= closing);
            let parameter_tail = &tail.comments[..tail_split];
            let expanded = !head.has_suppression()
                && !tail.has_suppression()
                && (leading.iter().any(|comment| {
                    comment.kind == CommentKind::Line
                        || self.line_breaks(comment.span.end, span.start) != 0
                }) || parameter_tail.iter().any(|comment| {
                    comment.kind == CommentKind::Line
                        || self.line_breaks(span.end, comment.span.start) != 0
                }));
            Some(PlainCatchBinding {
                pattern,
                mode,
                opening,
                closing,
                head,
                tail,
                before_opening,
                leading,
                expanded,
            })
        } else {
            None
        };
        let mut body_head = self.compound_gap(Span::new(
            binding
                .as_ref()
                .map_or(keyword_end, |binding| binding.closing + 1),
            body_start,
        ));
        if binding.is_none() {
            self.prepare_block_head(body_head, &mut body);
        }
        if binding.is_none() {
            self.transfer_block_leading(&mut body_head, &mut body);
        }
        if incoming_first == ContentMode::Verbatim {
            self.protect_block_first(&mut body);
        }
        Ok(PreparedCatch {
            node,
            head: Some(CatchHead {
                binding,
                body_head,
                body,
                leading_before_parameter: false,
            }),
        })
    }

    fn format_catch(
        &mut self,
        handler: PreparedCatch<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let Some(handler) = handler.head else {
            return self.format_verbatim(handler.node.span(self.ast), writer);
        };
        if handler.leading_before_parameter {
            self.comments.claim(handler.body.leading.comments())?;
        }
        writer.text("catch");
        if let Some(binding) = handler.binding {
            writer.group(false, |writer| {
                if binding.head.has_suppression() {
                    self.format_compound_gap(
                        binding.head,
                        FlatSeparator::Empty,
                        Writer::soft_line,
                        writer,
                    )?;
                } else {
                    self.line_constraint(
                        binding.head.span.start,
                        binding
                            .before_opening
                            .first()
                            .map_or(binding.opening, |comment| comment.span.start),
                    )
                    .write(writer, FlatSeparator::Space, Writer::space);
                    self.comments.claim(binding.before_opening)?;
                    self.format_comment_run(
                        binding.before_opening,
                        CommentRunLayout::FlatLeading(binding.opening),
                        writer,
                    )?;
                    writer.text("(");
                }
                let span = binding.pattern.span(self.ast);
                let mut parameter = |writer: &mut Writer<'ast>| {
                    if !binding.head.has_suppression() {
                        if binding.expanded {
                            writer.soft_line();
                        }
                        self.comments.claim(binding.leading)?;
                        self.format_comment_run(
                            binding.leading,
                            CommentRunLayout::GroupedLeading(binding.head.span.end),
                            writer,
                        )?;
                    }
                    let tail = self.format_catch_binding(binding.pattern, binding.mode, writer)?;
                    if binding.tail.has_suppression() {
                        self.format_expression_tail(tail, writer)?;
                        Ok((span.end, false))
                    } else {
                        let tail = self
                            .capture_expression_tail(Span::new(span.end, binding.closing), tail)?;
                        let last = tail.as_ref().and_then(|tail| tail.comments.last());
                        let end = last.map_or(span.end, |comment| comment.span.end);
                        let line = last.is_some_and(|comment| comment.kind == CommentKind::Line);
                        self.format_expression_tail(tail, writer)?;
                        Ok::<_, FormatError>((end, line))
                    }
                };
                let (end, line) = if binding.expanded {
                    writer.indent(parameter)?
                } else {
                    parameter(writer)?
                };
                if binding.tail.has_suppression() {
                    self.format_compound_gap(
                        binding.tail,
                        FlatSeparator::Space,
                        Writer::space,
                        writer,
                    )?;
                } else {
                    self.line_constraint(end, binding.closing).write(
                        writer,
                        FlatSeparator::Empty,
                        |writer| {
                            if line {
                                writer.hard_line();
                            } else if binding.expanded {
                                writer.soft_line();
                            }
                        },
                    );
                    writer.text(")");
                    self.format_compound_gap(
                        handler.body_head,
                        FlatSeparator::Space,
                        Writer::space,
                        writer,
                    )?;
                }
                Ok::<_, FormatError>(())
            })?;
        } else {
            self.format_compound_gap(
                handler.body_head,
                FlatSeparator::Space,
                Writer::space,
                writer,
            )?;
        }
        if !handler.leading_before_parameter {
            self.comments.claim(handler.body.leading.comments())?;
        }
        self.format_compound_block_part(handler.body, writer)
    }
}
