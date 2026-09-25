//! Basic statement bodies and their enclosing source boundaries.

use swc_next_ecma_ast::{
    BlockStatement, Comment, ExprData, GetSpan, Span, Stmt, StmtData, SubRange,
};

use super::Formatter;
use super::comments::CommentRunLayout;
use super::source::is_source_whitespace;
use crate::{FormatError, Semicolons, document::Writer};

#[derive(Clone, Copy)]
pub(super) enum StatementSequenceKind {
    Program,
    Block,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_statement_sequence(
        &mut self,
        statements: SubRange<Stmt>,
        span: Span,
        mut previous_end: Option<u32>,
        kind: StatementSequenceKind,
        writer: &mut Writer<'ast>,
    ) -> Result<bool, FormatError> {
        let ast = self.ast;
        let mut statements = statements
            .iter()
            .map(|id| ast.get_node_in_sub_range(id))
            .peekable();
        let mut previous_span_end = previous_end.unwrap_or(span.start);
        let mut previous_omitted: Option<Span> = None;
        while let Some(statement) = statements.next() {
            if matches!(ast.stmt_data(statement), StmtData::EmptyStatement(_)) {
                continue;
            }
            let statement_span = statement.span(ast);
            let leading = self.comments.between(Span::new(
                previous_end.unwrap_or(span.start),
                statement_span.start,
            ))?;
            let leading = self.comments.take(leading.len());
            let mut omitted: Option<Span> = None;
            while let Some(next) = statements.peek() {
                if !matches!(ast.stmt_data(*next), StmtData::EmptyStatement(_)) {
                    break;
                }
                let empty = statements.next().unwrap().span(ast);
                omitted = Some(Span::new(
                    omitted.map_or(empty.start, |span| span.start),
                    empty.end,
                ));
            }
            let following = statements.peek().map(|next| next.span(ast).start);
            let omitted = omitted.map(|empty| {
                self.prepare_omitted_statement_tail(statement_span.end, empty, following, span.end)
            });
            if let Some(end) = previous_end {
                let next_start = leading
                    .first()
                    .map_or(statement_span.start, |comment| comment.span.start);
                writer.hard_line();
                let empty_line = if let Some(gap) = previous_omitted {
                    self.next_line_is_empty(gap.start, gap.end)
                } else {
                    let gap_start = previous_span_end.max(end);
                    let before_separator =
                        end < gap_start && self.next_line_is_empty(end, gap_start.min(next_start));
                    let separator_end = leading
                        .get(leading.partition_point(|comment| comment.span.start < gap_start))
                        .map_or(statement_span.start, |comment| comment.span.start);
                    let after_separator = gap_start < separator_end
                        && self.next_line_is_empty(gap_start, separator_end);
                    before_separator || after_separator
                };
                if empty_line {
                    writer.hard_line();
                }
            }
            self.format_comment_run(leading, CommentRunLayout::Source, writer)?;
            if let Some(last) = leading.last() {
                self.comment_gap(last.span.end, statement_span.start, writer);
            }
            let content_end = self.format_statement(statement, writer)?;
            previous_end = Some(if let Some((comments, _)) = omitted {
                self.format_omitted_statement_tail(comments, content_end, writer)?
            } else {
                self.format_trailing_comments(
                    Span::new(content_end, following.unwrap_or(span.end)),
                    statement_span.end,
                    following.is_some(),
                    writer,
                )?
            });
            previous_span_end = statement_span.end;
            previous_omitted = omitted.map(|(_, gap)| gap);
        }
        let dangling = self
            .comments
            .between(Span::new(previous_end.unwrap_or(span.start), span.end))?;
        let dangling = self.comments.take(dangling.len());
        if let Some(first) = dangling.first()
            && let Some(end) = previous_end
        {
            self.comment_gap(end, first.span.start, writer);
        }
        self.format_comment_run(
            dangling,
            if previous_end.is_none() && matches!(kind, StatementSequenceKind::Block) {
                CommentRunLayout::CanonicalDangling
            } else {
                CommentRunLayout::Source
            },
            writer,
        )?;
        Ok(previous_end.is_some() || !dangling.is_empty())
    }

    fn prepare_omitted_statement_tail(
        &self,
        start: u32,
        statements: Span,
        following: Option<u32>,
        end: u32,
    ) -> (&'ast [Comment], Span) {
        let limit = following.unwrap_or(end);
        let comments = self.comments.in_range(Span::new(start, limit));
        let mut gap_start = start;
        for comment in comments {
            if self.line_breaks(gap_start, comment.span.start) != 0 {
                break;
            }
            gap_start = comment.span.end;
        }
        let before = comments.partition_point(|comment| comment.span.end <= statements.end);
        let after = self.trailing_comment_prefix(
            &comments[before..],
            Span::new(statements.end, limit),
            statements.end,
            following.is_some(),
        );
        let mut count = 0;
        let mut previous = start;
        for comment in &comments[..before + after.len()] {
            let gap = self.source(Span::new(previous, comment.span.start));
            if gap
                .rsplit_once(['\n', '\r', '\u{2028}', '\u{2029}'])
                .is_some_and(|(_, line)| line.chars().all(is_source_whitespace))
            {
                break;
            }
            previous = comment.span.end;
            count += 1;
        }
        (&comments[..count], Span::new(gap_start, limit))
    }

    fn format_omitted_statement_tail(
        &mut self,
        comments: &[Comment],
        start: u32,
        writer: &mut Writer<'ast>,
    ) -> Result<u32, FormatError> {
        let end = comments
            .last()
            .map_or(start, |comment| start.max(comment.span.end));
        let comments = self.comments.between(Span::new(start, end))?;
        let comments = self.comments.take(comments.len());
        self.format_trailing_comment_run(comments, writer)?;
        Ok(end)
    }

    fn format_statement(
        &mut self,
        statement: Stmt,
        writer: &mut Writer<'ast>,
    ) -> Result<u32, FormatError> {
        let content_end = match self.ast.stmt_data(statement) {
            StmtData::ExpressionStatement(node) => {
                let expression = node.expression(self.ast);
                match self.ast.expr_data(expression) {
                    ExprData::IdentifierReference(identifier) => {
                        writer.text(self.ast.get_utf8(identifier.name(self.ast)))
                    }
                    _ => return Err(self.unsupported("expression", expression)),
                }
                expression.span(self.ast).end
            }
            StmtData::DebuggerStatement(node) => {
                writer.text("debugger");
                node.span(self.ast).start + "debugger".len() as u32
            }
            StmtData::BlockStatement(block) => {
                self.format_block(block, writer)?;
                return Ok(block.span(self.ast).end);
            }
            _ => return Err(self.unsupported("statement", statement)),
        };
        if self.options.semicolons == Semicolons::Always {
            writer.text(";");
        }
        Ok(content_end)
    }

    fn format_block(
        &mut self,
        block: BlockStatement,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let block_span = block.span(self.ast);
        let body_span = Span::new(block_span.start + 1, block_span.end - 1);
        let body = block.body(self.ast);
        writer.text("{");
        if body.iter().all(|id| {
            matches!(
                self.ast.stmt_data(self.ast.get_node_in_sub_range(id)),
                StmtData::EmptyStatement(_)
            )
        }) && self.comments.between(body_span)?.is_empty()
        {
            writer.hard_line();
        } else {
            writer.indent(|writer| {
                writer.hard_line();
                self.format_statement_sequence(
                    body,
                    body_span,
                    None,
                    StatementSequenceKind::Block,
                    writer,
                )
            })?;
            writer.hard_line();
        }
        writer.text("}");
        Ok(())
    }
}
