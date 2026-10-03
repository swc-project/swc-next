//! Loop conditions and body boundaries use their real adjacent syntax owners.

use swc_next_ecma_ast::{DoWhileStatement, GetSpan, Span, WhileStatement};

use super::super::{
    EmptyBlock, Formatter, PrintedStatement, StatementBoundary, StatementPunctuation,
};
use super::{ConditionPrefix, ControlledBodyEnd, ControlledBodyRole};
use crate::formatter::diagnostics::LineConstraint;
use crate::formatter::suppression::{ContentMode, SuppressionPosition};
use crate::{FormatError, Semicolons, document::Writer};

/// The physical gap is retained only for an active ignore. Its two modes refer
/// to actual AST owners, independently of the keywords and parentheses in it.
pub(super) struct LoopBoundary {
    pub(super) preceding: ContentMode,
    pub(super) following: ContentMode,
    pub(super) source: Option<Span>,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(in crate::formatter::statements) fn format_while_statement(
        &mut self,
        node: WhileStatement,
        punctuation: StatementPunctuation,
        writer: &mut Writer<'ast>,
    ) -> Result<PrintedStatement<'ast>, FormatError> {
        let span = node.span(self.ast);
        let statement = node.body(self.ast);
        let condition = self.prepare_condition(
            node.test(self.ast),
            span.start + "while".len() as u32,
            statement.span(self.ast).start,
        )?;
        let body = self
            .prepare_controlled_body(
                statement,
                condition.closing + 1,
                ControlledBodyEnd::Enclosing(span.end),
                ControlledBodyRole::Body,
                ContentMode::Formatted,
            )
            .with_empty_block(EmptyBlock::Compact)
            .with_enclosing_punctuation(punctuation);
        writer.group(false, |writer| {
            writer.text("while");
            self.format_condition(condition, writer)?;
            self.format_controlled_body(body, writer)
        })
    }

    pub(in crate::formatter::statements) fn format_do_while_statement(
        &mut self,
        node: DoWhileStatement,
        punctuation: StatementPunctuation,
        writer: &mut Writer<'ast>,
    ) -> Result<PrintedStatement<'ast>, FormatError> {
        let span = node.span(self.ast);
        let statement = node.body(self.ast);
        let body_span = statement.span(self.ast);
        let expression = node.test(self.ast);
        let test_span = expression.span(self.ast);
        let head = Span::new(body_span.end, test_span.start);
        let comments = self.comments.in_range(head);
        let keyword_start = self
            .find_token(head, comments, "while")
            .ok_or_else(|| self.unsupported("do-while keyword boundary", node))?;
        let keyword = Span::new(keyword_start, keyword_start + "while".len() as u32);
        let boundary = self.prepare_loop_boundary(head, keyword);
        let prefix = boundary
            .source
            .map_or(ConditionPrefix::Canonical, |source| {
                ConditionPrefix::Source {
                    source,
                    mode: boundary.following,
                }
            });
        let condition =
            self.prepare_condition_with_prefix(expression, keyword.end, span.end, prefix)?;
        let closing = condition.closing;
        if self.trailing_content_mode(Span::new(closing + 1, span.end), true)
            == ContentMode::Verbatim
        {
            self.format_verbatim(span, writer)?;
            return Ok(StatementBoundary::SourceEnd {
                end: span.end,
                has_terminator: self.statement_source_terminator(span).is_some(),
            }
            .into());
        }
        let end = if boundary.source.is_some() {
            ControlledBodyEnd::Enclosing(body_span.end)
        } else {
            ControlledBodyEnd::LoopCondition(keyword)
        };
        let mut body = self
            .prepare_controlled_body(
                statement,
                span.start + "do".len() as u32,
                end,
                ControlledBodyRole::Body,
                boundary.preceding,
            )
            .with_empty_block(EmptyBlock::Compact);
        if boundary.source.is_some()
            && self.source_line_constraint(body_span.end, keyword.start) == LineConstraint::SameLine
        {
            // The retained gap cannot add an ASI newline before its `while`.
            body = body
                .with_enclosing_punctuation(StatementPunctuation::Canonical(Semicolons::Always));
        }
        let keyword_boundary = self.prepare_body_keyword(&body, keyword);
        let printed = writer.group(false, |writer| {
            writer.text("do");
            let printed = self.format_controlled_body(body, writer)?;
            self.flush_statement_tail(printed, writer)
                .map(PrintedStatement::from)
        })?;
        writer.group(false, |writer| {
            if boundary.source.is_none() {
                self.format_body_keyword(keyword_boundary, "while", printed, writer)?;
            }
            self.format_condition(condition, writer)
        })?;
        self.finish_statement(closing + 1, None, punctuation, writer)
    }

    pub(super) fn prepare_loop_boundary(&self, gap: Span, separator: Span) -> LoopBoundary {
        let mut boundary = LoopBoundary {
            preceding: ContentMode::Formatted,
            following: ContentMode::Formatted,
            source: None,
        };
        if !self.has_suppression {
            return boundary;
        }
        let comments = self.comments.in_range(gap);
        if self.content_mode(comments) != ContentMode::Verbatim {
            return boundary;
        }
        boundary.source = Some(gap);
        for (run, position) in self.suppressed_comment_runs(comments, gap.end) {
            let preceding = match position {
                SuppressionPosition::OwnLine => false,
                SuppressionPosition::EndOfLine => true,
                SuppressionPosition::Inline => {
                    let Some(last) = run.last() else {
                        continue;
                    };
                    last.span.end <= separator.start
                }
            };
            if preceding {
                boundary.preceding = ContentMode::Verbatim;
            } else {
                boundary.following = ContentMode::Verbatim;
            }
        }
        boundary
    }
}
