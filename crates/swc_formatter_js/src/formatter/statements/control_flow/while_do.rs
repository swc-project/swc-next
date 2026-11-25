//! Loop conditions and body boundaries use their real adjacent syntax owners.

use swc_next_ecma_ast::{Comment, DoWhileStatement, GetSpan, Span, Stmt, StmtData, WhileStatement};

use super::super::{
    EmptyBlock, Formatter, PrintedStatement, StatementBoundary, StatementContent, StatementItem,
    StatementPunctuation, StatementTailRelation,
};
use super::{
    Condition, ConditionHeader, ConditionLineSuffix, ConditionPrefix, ControlledBodyContent,
    ControlledBodyEnd, ControlledBodyRole, ControlledBodyShape,
};
use crate::formatter::diagnostics::LineConstraint;
use crate::formatter::suppression::{ContentMode, SuppressionPosition};
use crate::{FormatError, Semicolons, document::Writer};

/// The physical gap is retained only for an active ignore. Its two modes refer
/// to actual AST owners, independently of the keywords and parentheses in it.
struct LoopBoundary {
    preceding: ControlledBodyContent,
    following: ContentMode,
    source: Option<Span>,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(in crate::formatter::statements) fn format_while_statement(
        &mut self,
        node: WhileStatement,
        punctuation: StatementPunctuation,
        source_candidate: crate::formatter::statements::ContinuationBoundary,
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
                ControlledBodyContent::Formatted,
            )
            .with_empty_block(EmptyBlock::Compact)
            .with_enclosing_boundary(punctuation, source_candidate);
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
        let boundary = self.prepare_loop_boundary(statement, head, keyword, comments);
        // The condition opener is a grammar delimiter. Comments in the
        // keyword-to-opener gap belong to the header, outside the condition.
        let prefix = boundary
            .source
            .map_or(ConditionPrefix::Canonical, |source| {
                ConditionPrefix::Source {
                    source,
                    mode: boundary.following,
                }
            });
        let mut condition =
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
        let end = if let Some(source) = boundary.source {
            ControlledBodyEnd::RetainedGap(source.start)
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
        if boundary.source.is_some() {
            if let Some(terminator) = self.statement_source_terminator(body_span)
                && let StatementContent::Semicolon(content_end) =
                    self.statement_content(StatementItem::Statement(statement))
                && !self
                    .comments
                    .in_range(Span::new(content_end, terminator.start()))
                    .is_empty()
            {
                // The retained while gap starts after the body terminator.
                // Let the terminator claim its entire preceding comment run.
                body = body.with_enclosing_boundary(
                    StatementPunctuation::Source(terminator),
                    super::super::ContinuationBoundary::End,
                );
            } else if self.source_line_constraint(body_span.end, keyword.start)
                == LineConstraint::SameLine
            {
                // The retained gap cannot add an ASI newline before its `while`.
                body = body.with_enclosing_boundary(
                    StatementPunctuation::Canonical(Semicolons::Always),
                    super::super::ContinuationBoundary::End,
                );
            }
            body = self.with_controlled_body_keyword_fence(body, keyword);
        }
        let keyword_boundary = body.keyword;
        let condition_suffix = self.prepare_do_condition_suffix(&mut condition);
        let statement_suffix = boundary.source.is_none()
            && matches!(body.shape, ControlledBodyShape::Block)
            && self
                .prepare_controlled_body_tail(
                    span,
                    closing + 1,
                    ControlledBodyEnd::Enclosing(span.end),
                    ControlledBodyShape::Single,
                )
                .is_some_and(|tail| {
                    tail.binding.allows_relocation(None)
                        && ConditionLineSuffix::new(tail.tail.comments).is_some()
                });
        if let Some(keyword_boundary) = keyword_boundary
            && (condition_suffix.is_some() || statement_suffix)
        {
            condition.inline = false;
            writer.group(false, |writer| {
                writer.text("do");
                let printed = self.format_controlled_body(body, writer)?;
                self.format_body_keyword(keyword_boundary, "while", printed, writer)?;
                self.format_condition_with_suffix(condition, condition_suffix, writer)
            })?;
            return self.finish_statement(closing + 1, None, punctuation, writer);
        }
        let printed = writer.group(false, |writer| {
            writer.text("do");
            let mut printed = self.format_controlled_body(body, writer)?;
            if let Some(source) = boundary.source {
                printed.retain_pending_before(source.start);
            }
            self.flush_statement_tail(printed, writer)
                .map(PrintedStatement::from)
        })?;
        writer.group(false, |writer| {
            if let Some(keyword_boundary) = keyword_boundary {
                self.format_body_keyword(keyword_boundary, "while", printed, writer)?;
            }
            self.format_condition(condition, writer)
        })?;
        self.finish_statement(closing + 1, None, punctuation, writer)
    }

    fn prepare_do_condition_suffix(
        &self,
        condition: &mut Condition<'ast>,
    ) -> Option<ConditionLineSuffix<'ast>> {
        let ConditionHeader::Canonical {
            keyword_end,
            opening,
            ..
        } = &condition.header
        else {
            return None;
        };
        if condition.mode != ContentMode::Formatted {
            return None;
        }
        let mut split = 0;
        let mut previous = *opening + 1;
        for comment in condition.leading {
            if self.line_breaks(previous, comment.span.start) != 0 {
                break;
            }
            previous = comment.span.start;
            split += 1;
        }
        let comments = &condition.leading[..split];
        let suffix = ConditionLineSuffix::new(comments)?;
        if !self
            .comment_binding(
                comments,
                Span::new(*keyword_end, condition.expression.span(self.ast).start),
                None,
            )
            .allows_relocation(None)
        {
            return None;
        }
        condition.leading = &condition.leading[split..];
        Some(suffix)
    }

    fn prepare_loop_boundary(
        &self,
        statement: Stmt,
        gap: Span,
        keyword: Span,
        comments: &[Comment],
    ) -> LoopBoundary {
        let mut boundary = LoopBoundary {
            preceding: ControlledBodyContent::Formatted,
            following: ContentMode::Formatted,
            source: None,
        };
        if !self.has_suppression {
            return boundary;
        }
        if let Some(tail) = self.prepare_statement_suppression_tail(
            StatementItem::Statement(statement),
            keyword.start,
            StatementTailRelation::Keyword,
        ) {
            if tail.modes.current == ContentMode::Verbatim {
                boundary.preceding = match self.ast.stmt_data(statement) {
                    StmtData::BlockStatement(_) => ControlledBodyContent::Verbatim,
                    _ => ControlledBodyContent::SourceNode,
                };
            }
            boundary.following = tail.modes.enclosing;
        }
        let split = comments.partition_point(|comment| comment.span.end <= keyword.start);
        let (before, after) = comments.split_at(split);
        for position in self.suppression_positions(after, gap.end) {
            boundary.source = Some(gap);
            match position {
                SuppressionPosition::EndOfLine => {
                    if boundary.preceding == ControlledBodyContent::Formatted {
                        boundary.preceding = ControlledBodyContent::Verbatim;
                    }
                }
                _ => boundary.following = ContentMode::Verbatim,
            }
        }
        if self.content_mode(before) == ContentMode::Verbatim {
            let before_keyword = Span::new(gap.start, keyword.start);
            let body = statement.span(self.ast);
            // The body keeps its raw syntax, while an unbound keyword join can
            // use the same layout as an ordinary controlled-body tail.
            if boundary.preceding != ControlledBodyContent::SourceNode
                || boundary.following == ContentMode::Verbatim
                || self.has_line_directive_in(before_keyword)
                || self.lint_line_intersects(Span::new(gap.start, keyword.end))
                || !self
                    .comment_binding(before, before_keyword, Some(body))
                    .allows_relocation(Some(body))
            {
                boundary.source = Some(gap);
            }
        }
        boundary
    }
}
