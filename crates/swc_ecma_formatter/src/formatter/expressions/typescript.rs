//! Runtime angle suffixes shared by interface heritage and expression consumers.

use swc_next_ecma_ast::{CommentKind, Expr, GetSpan, Span, TsTypeParameterInstantiation};

use crate::formatter::Formatter;
use crate::formatter::comments::{
    CommentBinding, CommentBoundary, CommentRunLayout, ExpressionTail,
};
use crate::formatter::diagnostics::FlatSeparator;
use crate::formatter::source::is_source_whitespace;
use crate::formatter::suppression::{ContentMode, SuppressionPosition};
use crate::{FormatError, document::Writer};

pub(in crate::formatter) struct ExpressionTypeArguments<'ast> {
    node: TsTypeParameterInstantiation,
    pub(in crate::formatter) callee_mode: ContentMode,
    mode: ContentMode,
    prefix: ExpressionTypeArgumentPrefix<'ast>,
}

enum ExpressionTypeArgumentPrefix<'ast> {
    Source(Span),
    Formatted(CommentBoundary<'ast>),
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(in crate::formatter) fn prepare_expression_type_arguments(
        &self,
        expression: Expr,
        node: TsTypeParameterInstantiation,
    ) -> ExpressionTypeArguments<'ast> {
        let gap = Span::new(expression.span(self.ast).end, node.span(self.ast).start);
        let comments = self.comments.in_range(gap);
        let mut callee_mode = ContentMode::Formatted;
        let mut mode = ContentMode::Formatted;
        if self.has_suppression && self.content_mode(comments) == ContentMode::Verbatim {
            for (_, position) in self.suppressed_comment_runs(comments, gap.end) {
                if matches!(position, SuppressionPosition::EndOfLine) {
                    callee_mode = ContentMode::Verbatim;
                } else {
                    mode = ContentMode::Verbatim;
                }
            }
        }
        let prefix = if callee_mode == ContentMode::Verbatim || mode == ContentMode::Verbatim {
            ExpressionTypeArgumentPrefix::Source(gap)
        } else {
            ExpressionTypeArgumentPrefix::Formatted(
                self.prepare_comment_boundary(comments, gap.start, gap.end),
            )
        };
        ExpressionTypeArguments {
            node,
            callee_mode,
            mode,
            prefix,
        }
    }

    pub(in crate::formatter) fn format_expression_type_arguments(
        &mut self,
        arguments: ExpressionTypeArguments<'ast>,
        tail: Option<ExpressionTail<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        match arguments.prefix {
            ExpressionTypeArgumentPrefix::Source(gap) => {
                let line_tail = tail.as_ref().is_some_and(|tail| {
                    tail.comments
                        .last()
                        .is_some_and(|comment| comment.kind == CommentKind::Line)
                });
                let needs_line = line_tail
                    && !self
                        .source(gap)
                        .chars()
                        .take_while(|&character| is_source_whitespace(character))
                        .any(|character| matches!(character, '\r' | '\n'));
                self.format_expression_tail(tail, writer)?;
                if needs_line {
                    // The removed callee pair supplied the line before this
                    // protected gap. Only CR/LF in literal text flushes a suffix.
                    writer.hard_line();
                }
                self.format_verbatim(gap, writer)?;
            }
            ExpressionTypeArgumentPrefix::Formatted(before) => {
                let before = if let Some(tail) = tail {
                    // The real angle suffix owns one source-ordered packet,
                    // including comments exposed by removed callee pairs.
                    let end = arguments.node.span(self.ast).start;
                    let comments = self.comments.between(Span::new(tail.source_end, end))?;
                    self.prepare_comment_boundary(comments, tail.source_end, end)
                } else {
                    before
                };
                if !before.is_empty()
                    && matches!(
                        self.comment_boundary_binding(&before, arguments.node.span(self.ast)),
                        CommentBinding::Unbound
                    )
                {
                    // Keep one source-ordered packet. Its first comment selects
                    // a callee line tail or an angle-list block head.
                    if before.comments()[0].kind == CommentKind::Line {
                        writer.space();
                    }
                    self.format_comment_boundary_with_layout(
                        &before,
                        CommentRunLayout::FlatLeading(arguments.node.span(self.ast).start),
                        writer,
                    )?;
                } else {
                    self.format_comment_boundary(&before, writer)?;
                    before.gap().write(writer, FlatSeparator::Empty, |_| {});
                }
            }
        }
        self.format_type_arguments(arguments.node, arguments.mode, writer)
    }
}
