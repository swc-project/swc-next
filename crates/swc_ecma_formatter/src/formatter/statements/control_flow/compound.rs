//! Shared source packets and actual block consumers for compound statements.

mod exceptions;
mod switch;

use super::{ControlledBodyEnd, ControlledBodyRole};
use crate::formatter::comments::CommentRunLayout;
use crate::formatter::diagnostics::FlatSeparator;
use crate::formatter::statements::{
    EmptyBlock, Formatter, IncomingStatement, PrintedStatement, StatementBodyLeading,
    StatementItem, StatementPunctuation,
};
use crate::formatter::suppression::{ContentMode, DelimitedFollowing, SuppressionPosition};
use crate::{FormatError, document::Writer};
use swc_next_ecma_ast::{BlockStatement, Comment, CommentKind, GetSpan, Span, WithStatement};

#[derive(Clone, Copy)]
enum GapLayout {
    Formatted,
    Source,
}

#[derive(Clone, Copy)]
struct CompoundGap<'ast> {
    span: Span,
    comments: &'ast [Comment],
    layout: GapLayout,
}

impl CompoundGap<'_> {
    fn has_suppression(self) -> bool {
        matches!(self.layout, GapLayout::Source)
    }
}

struct CompoundBlock<'ast> {
    node: BlockStatement,
    mode: ContentMode,
    incoming: Option<IncomingStatement>,
    empty: EmptyBlock,
    leading: StatementBodyLeading<'ast>,
}

impl CompoundBlock<'_> {
    fn new(node: BlockStatement, empty: EmptyBlock) -> Self {
        Self {
            node,
            mode: ContentMode::Formatted,
            incoming: None,
            empty,
            leading: StatementBodyLeading::empty(),
        }
    }
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(in crate::formatter::statements) fn format_with_statement(
        &mut self,
        node: WithStatement,
        punctuation: StatementPunctuation,
        writer: &mut Writer<'ast>,
    ) -> Result<PrintedStatement<'ast>, FormatError> {
        let span = node.span(self.ast);
        let body = node.body(self.ast);
        let mut condition = self.prepare_condition(
            node.object(self.ast),
            span.start + "with".len() as u32,
            body.span(self.ast).start,
        )?;
        condition.position = crate::formatter::expressions::ExpressionPosition::Nested;
        let body = self
            .prepare_controlled_body(
                body,
                condition.closing + 1,
                ControlledBodyEnd::Enclosing(span.end),
                ControlledBodyRole::Body,
                ContentMode::Formatted,
            )
            .with_enclosing_punctuation(punctuation);
        writer.group(false, |writer| {
            writer.text("with");
            self.format_condition(condition, writer)?;
            self.format_controlled_body(body, writer)
        })
    }

    fn compound_gap(&self, span: Span) -> CompoundGap<'ast> {
        let comments = self.comments.in_range(span);
        let layout = if self.content_mode(comments) == ContentMode::Verbatim {
            GapLayout::Source
        } else {
            GapLayout::Formatted
        };
        CompoundGap {
            span,
            comments,
            layout,
        }
    }

    fn format_compound_gap(
        &mut self,
        gap: CompoundGap<'ast>,
        flat: FlatSeparator,
        default: impl FnOnce(&mut Writer<'ast>),
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        if gap.has_suppression() {
            let printed =
                self.format_delimited_source_gap(gap.span, DelimitedFollowing::Node, writer)?;
            printed.following.write(writer, flat, default);
        } else {
            self.comments.claim(gap.comments)?;
            self.format_comment_run(
                gap.comments,
                CommentRunLayout::SourceAfter(gap.span.start),
                writer,
            )?;
            let end = gap
                .comments
                .last()
                .map_or(gap.span.start, |comment| comment.span.end);
            self.line_constraint(end, gap.span.end)
                .write(writer, flat, |writer| {
                    if gap
                        .comments
                        .last()
                        .is_some_and(|comment| comment.kind == CommentKind::Line)
                    {
                        writer.hard_line();
                    } else {
                        default(writer);
                    }
                });
        }
        Ok(())
    }

    fn protect_block_first(&self, body: &mut CompoundBlock<'ast>) {
        if body.mode == ContentMode::Verbatim || body.incoming.is_some() {
            return;
        }
        let first = body
            .node
            .body(self.ast)
            .iter()
            .map(|id| self.ast.get_node_in_sub_range(id))
            .find(|&node| StatementItem::Statement(node).is_nonempty(self.ast));
        if let Some(first) = first {
            body.incoming = Some(IncomingStatement {
                span: first.span(self.ast),
                mode: ContentMode::Verbatim,
            });
        } else {
            body.mode = ContentMode::Verbatim;
        }
    }

    fn prepare_block_head(&self, gap: CompoundGap<'ast>, body: &mut CompoundBlock<'ast>) {
        if !gap.has_suppression() {
            return;
        }
        for position in self.suppression_positions(gap.comments, gap.span.end) {
            match position {
                SuppressionPosition::Inline => body.mode = ContentMode::Verbatim,
                SuppressionPosition::EndOfLine | SuppressionPosition::OwnLine => {
                    self.protect_block_first(body)
                }
            }
        }
    }

    fn block_leading_start(&self, gap: CompoundGap<'ast>) -> Option<usize> {
        gap.comments
            .iter()
            .enumerate()
            .position(|(index, comment)| {
                let next = gap
                    .comments
                    .get(index + 1)
                    .map_or(gap.span.end, |next| next.span.start);
                comment.kind == CommentKind::Line
                    || self.source_position_starts_line(comment.span.start)
                    || self.line_breaks(comment.span.end, next) != 0
            })
    }

    fn transfer_block_leading(
        &self,
        gap: &mut CompoundGap<'ast>,
        body: &mut CompoundBlock<'ast>,
    ) -> bool {
        if gap.comments.is_empty()
            || body.mode == ContentMode::Verbatim
            || !body.leading.comments().is_empty()
            || !self
                .comment_binding(gap.comments, gap.span, None)
                .allows_relocation(None)
        {
            return false;
        }
        let Some(start) = self.block_leading_start(*gap) else {
            return false;
        };
        let target = body
            .node
            .body(self.ast)
            .iter()
            .map(|id| self.ast.get_node_in_sub_range(id))
            .find(|&node| StatementItem::Statement(node).is_nonempty(self.ast));
        if target.is_none() && self.has_suppression {
            let span = body.node.span(self.ast);
            if self.content_mode(
                self.comments
                    .in_range(Span::new(span.start + 1, span.end - 1)),
            ) == ContentMode::Verbatim
            {
                return false;
            }
        }
        body.leading = StatementBodyLeading::new(&gap.comments[start..], target);
        gap.comments = &gap.comments[..start];
        true
    }

    fn format_compound_block_part(
        &mut self,
        body: CompoundBlock<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        self.format_compound_block(
            body.node,
            body.mode,
            body.incoming,
            body.leading,
            body.empty,
            writer,
        )
    }
}
