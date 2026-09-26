//! Basic statement bodies and their enclosing source boundaries.

use swc_next_ecma_ast::{
    Ast, BlockStatement, Comment, Directive, EmptyStatement, ExprData, GetSpan, Span, Stmt,
    StmtData,
};

use super::Formatter;
use super::comments::CommentRunLayout;
use super::diagnostics::{FlatSeparator, LineConstraint};
use super::literals::StringContext;
use super::source::is_source_whitespace;
use crate::{FormatError, Semicolons, document::Writer};

#[derive(Clone, Copy)]
pub(super) enum StatementSequenceKind {
    Program,
    Block,
}

#[derive(Clone, Copy)]
pub(super) enum StatementItem {
    Directive(Directive),
    Statement(Stmt),
}

impl StatementItem {
    fn empty(self, ast: &Ast<'_>) -> Option<EmptyStatement> {
        match self {
            Self::Statement(node) => match ast.stmt_data(node) {
                StmtData::EmptyStatement(empty) => Some(empty),
                _ => None,
            },
            Self::Directive(_) => None,
        }
    }

    fn is_nonempty(self, ast: &Ast<'_>) -> bool {
        !matches!(self, Self::Statement(statement) if matches!(ast.stmt_data(statement), StmtData::EmptyStatement(_)))
    }

    fn span(self, ast: &Ast<'_>) -> Span {
        match self {
            Self::Directive(node) => node.span(ast),
            Self::Statement(node) => node.span(ast),
        }
    }
}

#[derive(Clone, Copy)]
struct StatementLayout {
    guarded: bool,
    source_parentheses: bool,
}

#[derive(Clone, Copy)]
enum StatementBoundary {
    SourceEnd { end: u32, has_terminator: bool },
}

impl StatementBoundary {
    fn end(self) -> u32 {
        match self {
            Self::SourceEnd { end, .. } => end,
        }
    }
    fn has_terminator(self) -> bool {
        match self {
            Self::SourceEnd { has_terminator, .. } => has_terminator,
        }
    }
    fn with_end(self, end: u32) -> Self {
        match self {
            Self::SourceEnd { has_terminator, .. } => Self::SourceEnd {
                end,
                has_terminator,
            },
        }
    }
}

pub(super) struct StatementSequence {
    pub(super) has_content: bool,
    pub(super) end: u32,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_statement_sequence(
        &mut self,
        mut statements: std::iter::Peekable<impl Iterator<Item = StatementItem>>,
        span: Span,
        previous_end: Option<u32>,
        kind: StatementSequenceKind,
        writer: &mut Writer<'ast>,
    ) -> Result<StatementSequence, FormatError> {
        let ast = self.ast;
        let mut previous = previous_end.map(|end| StatementBoundary::SourceEnd {
            end,
            has_terminator: false,
        });
        let mut previous_span_end = previous_end.unwrap_or(span.start);
        let mut previous_omitted: Option<Span> = None;
        while let Some(statement) = statements.next() {
            // Match Prettier's empty-statement removal. Comment protection covers
            // existing effects being lost, not inert annotations becoming active.
            // ECMAScript directives such as "use strict" must retain their semantics
            // in both directions.
            if statement.empty(ast).is_some() {
                continue;
            }
            let statement_span = statement.span(ast);
            let leading = self.comments.between(Span::new(
                previous.map_or(span.start, StatementBoundary::end),
                statement_span.start,
            ))?;
            let leading = self.comments.take(leading.len());
            let mut omitted: Option<Span> = None;
            while let Some(next) = statements.peek() {
                if next.empty(ast).is_none() {
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
            if let Some(boundary) = previous {
                let end = boundary.end();
                let next_start = leading
                    .first()
                    .map_or(statement_span.start, |comment| comment.span.start);
                let constraint = self.line_constraint(end, next_start);
                if constraint == LineConstraint::SameLine
                    && self.options.semicolons == Semicolons::AsNeeded
                    && !boundary.has_terminator()
                {
                    writer.text(";");
                }
                constraint.write(writer, FlatSeparator::Space, Writer::hard_line);
                if constraint == LineConstraint::Unconstrained {
                    let empty_line = if let Some(gap) = previous_omitted {
                        self.next_line_is_empty(gap.start, gap.end)
                    } else {
                        let gap_start = previous_span_end.max(end);
                        let before_separator = end < gap_start
                            && self.next_line_is_empty(end, gap_start.min(next_start));
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
            }
            let expression = match statement {
                StatementItem::Statement(node) => match ast.stmt_data(node) {
                    StmtData::ExpressionStatement(node) => Some(node.expression(ast)),
                    _ => None,
                },
                StatementItem::Directive(_) => None,
            };
            let needs_guard = self.options.semicolons == Semicolons::AsNeeded
                && expression.is_some_and(|expression| {
                    self.string_expression_literal(expression).is_some()
                        || matches!(ast.expr_data(expression), ExprData::RegExpLiteral(_))
                });
            let guarded = needs_guard
                && leading
                    .iter()
                    .any(|&comment| self.comment_has_target_annotation(comment));
            if guarded {
                writer.text(";");
            }
            let source_parentheses = expression.is_some_and(|expression| {
                matches!(
                    ast.expr_data(expression),
                    ExprData::ParenthesizedExpression(_)
                ) && (self.diagnostic_range_is_sensitive(expression.span(ast))
                    || leading
                        .iter()
                        .any(|&comment| self.comment_is_type_cast(comment)))
            });
            let layout = StatementLayout {
                guarded,
                source_parentheses,
            };
            self.format_comment_run(leading, CommentRunLayout::Source, writer)?;
            if let Some(last) = leading.last() {
                self.comment_gap(last.span.end, statement_span.start, writer);
            }
            let boundary = match statement {
                StatementItem::Statement(node) => self.format_statement(node, layout, writer)?,
                StatementItem::Directive(node) => {
                    let expression = node.expression(ast);
                    self.format_string_literal(expression, StringContext::Directive, writer);
                    if self.options.semicolons == Semicolons::Always {
                        writer.text(";");
                    }
                    StatementBoundary::SourceEnd {
                        end: expression.span(ast).end,
                        has_terminator: self.options.semicolons == Semicolons::Always,
                    }
                }
            };
            let content_end = boundary.end();
            let end = if let Some((comments, _)) = omitted {
                self.format_omitted_statement_tail(comments, content_end, writer)?
            } else {
                self.format_trailing_comments(
                    Span::new(content_end, following.unwrap_or(span.end)),
                    statement_span.end,
                    following.is_some(),
                    writer,
                )?
            };
            previous = Some(boundary.with_end(end));
            previous_span_end = statement_span.end;
            previous_omitted = omitted.map(|(_, gap)| gap);
        }
        let dangling = self.comments.between(Span::new(
            previous.map_or(span.start, StatementBoundary::end),
            span.end,
        ))?;
        let dangling = self.comments.take(dangling.len());
        if let Some(first) = dangling.first()
            && let Some(boundary) = previous
        {
            self.comment_gap(boundary.end(), first.span.start, writer);
        }
        self.format_comment_run(
            dangling,
            if previous.is_none() && matches!(kind, StatementSequenceKind::Block) {
                CommentRunLayout::CanonicalDangling
            } else {
                CommentRunLayout::Source
            },
            writer,
        )?;
        Ok(StatementSequence {
            has_content: previous.is_some() || !dangling.is_empty(),
            end: dangling.last().map_or_else(
                || previous.map_or(span.start, StatementBoundary::end),
                |comment| comment.span.end,
            ),
        })
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
            if following.is_some()
                && self.has_target_annotations
                && self.comment_has_target_annotation(*comment)
            {
                break;
            }
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
        if self
            .comment_binding(comments, Span::new(start, end))
            .allows_relocation()
        {
            self.format_trailing_comment_run(comments, writer)?;
        } else {
            self.format_comment_run(comments, CommentRunLayout::SourceAfter(start), writer)?;
        }
        Ok(end)
    }

    fn format_statement(
        &mut self,
        statement: Stmt,
        layout: StatementLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<StatementBoundary, FormatError> {
        let content_end = match self.ast.stmt_data(statement) {
            StmtData::ExpressionStatement(node) => {
                let expression = node.expression(self.ast);
                let parenthesize = self.string_expression_literal(expression).is_some();
                if !layout.guarded
                    && self.options.semicolons == Semicolons::AsNeeded
                    && (parenthesize
                        || matches!(self.ast.expr_data(expression), ExprData::RegExpLiteral(_)))
                {
                    writer.text(";");
                }
                if layout.source_parentheses
                    && let Some(literal) = self.string_expression_literal(expression)
                {
                    let outer = expression.span(self.ast);
                    let inner = literal.span(self.ast);
                    writer.group(false, |writer| {
                        writer.text("(");
                        writer.indent(|writer| {
                            self.line_constraint(outer.start + 1, inner.start).write(
                                writer,
                                FlatSeparator::Empty,
                                Writer::soft_line,
                            );
                            self.format_string_literal(literal, StringContext::Expression, writer);
                        });
                        self.line_constraint(inner.end, outer.end - 1).write(
                            writer,
                            FlatSeparator::Empty,
                            Writer::soft_line,
                        );
                        writer.text(")");
                    });
                } else {
                    if parenthesize {
                        writer.text("(");
                    }
                    self.format_expression(expression, writer)?;
                    if parenthesize {
                        writer.text(")");
                    }
                }
                expression.span(self.ast).end
            }
            StmtData::DebuggerStatement(node) => {
                writer.text("debugger");
                node.span(self.ast).start + "debugger".len() as u32
            }
            StmtData::BlockStatement(block) => {
                self.format_block(block, writer)?;
                return Ok(StatementBoundary::SourceEnd {
                    end: block.span(self.ast).end,
                    has_terminator: false,
                });
            }
            _ => return Err(self.unsupported("statement", statement)),
        };
        if self.options.semicolons == Semicolons::Always {
            writer.text(";");
        }
        Ok(StatementBoundary::SourceEnd {
            end: content_end,
            has_terminator: self.options.semicolons == Semicolons::Always,
        })
    }

    fn format_block(
        &mut self,
        block: BlockStatement,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
        let span = block.span(ast);
        let content = Span::new(span.start + 1, span.end - 1);
        let body = block.body(ast);
        let has_items = body
            .iter()
            .any(|id| StatementItem::Statement(ast.get_node_in_sub_range(id)).is_nonempty(ast));
        let empty = !has_items && self.comments.between(content)?.is_empty();
        let opening = self.opening_line_constraint(content);
        writer.text("{");
        if empty {
            opening.write(writer, FlatSeparator::Empty, Writer::hard_line);
        } else {
            let sequence = writer.indent(|writer| {
                opening.write(writer, FlatSeparator::Space, Writer::hard_line);
                self.format_statement_sequence(
                    body.iter()
                        .map(|id| StatementItem::Statement(ast.get_node_in_sub_range(id)))
                        .peekable(),
                    content,
                    None,
                    StatementSequenceKind::Block,
                    writer,
                )
            })?;
            self.line_constraint(sequence.end, content.end).write(
                writer,
                FlatSeparator::Space,
                Writer::hard_line,
            );
        }
        writer.text("}");
        Ok(())
    }
}
