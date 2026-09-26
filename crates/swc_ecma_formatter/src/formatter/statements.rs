//! Basic statement bodies and their enclosing source boundaries.

mod tails;

use swc_next_ecma_ast::{
    Ast, BlockStatement, Comment, CommentKind, Directive, EmptyStatement, ExprData, GetSpan, Span,
    Stmt, StmtData,
};

use self::tails::{StatementContent, StatementTailModes, StatementTailRelation};
use super::Formatter;
use super::comments::{CommentBinding, CommentRunLayout};
use super::diagnostics::{FlatSeparator, LineConstraint};
use super::literals::StringContext;
use super::source::is_source_whitespace;
use super::suppression::{ContentMode, is_suppression_comment};
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
pub(super) struct StatementSourceTerminator {
    start: u32,
}

#[derive(Clone, Copy)]
enum StatementPunctuation {
    Canonical(Semicolons),
    Source(StatementSourceTerminator),
    SourcePacket,
}

#[derive(Clone, Copy)]
enum StatementBoundary {
    SourceEnd { end: u32, has_terminator: bool },
    SourceTerminator { end: u32, terminator: u32 },
}

impl StatementBoundary {
    fn end(self) -> u32 {
        match self {
            Self::SourceEnd { end, .. } | Self::SourceTerminator { end, .. } => end,
        }
    }
    fn has_terminator(self) -> bool {
        match self {
            Self::SourceEnd { has_terminator, .. } => has_terminator,
            Self::SourceTerminator { .. } => true,
        }
    }
    fn guards_expression(self) -> bool {
        self.has_terminator()
    }
    fn with_end(self, end: u32) -> Self {
        match self {
            Self::SourceEnd { has_terminator, .. } => Self::SourceEnd {
                end,
                has_terminator,
            },
            Self::SourceTerminator { terminator, .. } => Self::SourceTerminator { end, terminator },
        }
    }
    fn with_source_packet(self, end: u32) -> Self {
        Self::SourceEnd {
            end,
            has_terminator: true,
        }
    }
}

struct StatementAnnotationTail<'ast> {
    comments: &'ast [Comment],
    end: u32,
    suppression: StatementTailModes,
    following_gap: Option<LineConstraint>,
}

struct PreparedStatement<'ast> {
    item: StatementItem,
    span: Span,
    mode: ContentMode,
    source_gap: Option<LineConstraint>,
    annotation_tail: Option<StatementAnnotationTail<'ast>>,
}

enum StatementMode {
    Formatted,
    Verbatim(StatementContent),
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
        let mut pending = statements
            .next()
            .map(|item| self.prepare_sequence_statement(item, statements.peek().copied()));
        while let Some(prepared) = pending {
            pending = statements
                .next()
                .map(|item| self.prepare_sequence_statement(item, statements.peek().copied()));
            let PreparedStatement {
                item: statement,
                span: statement_span,
                mode: source_mode,
                source_gap,
                annotation_tail,
            } = prepared;
            // Match Prettier's empty-statement removal. Comment protection covers
            // existing effects being lost, not inert annotations becoming active.
            // ECMAScript directives such as "use strict" must retain their semantics
            // in both directions.
            if statement.empty(ast).is_some() {
                continue;
            }
            let leading = self.comments.between(Span::new(
                previous.map_or(span.start, StatementBoundary::end),
                statement_span.start,
            ))?;
            let leading_mode = self.content_mode(leading);
            let mut omitted: Option<Span> = None;
            while let Some(next) = pending.as_ref() {
                let Some(empty) = next.item.empty(ast) else {
                    break;
                };
                let empty = empty.span(ast);
                omitted = Some(Span::new(
                    omitted.map_or(empty.start, |span| span.start),
                    empty.end,
                ));
                pending = statements
                    .next()
                    .map(|item| self.prepare_sequence_statement(item, statements.peek().copied()));
            }
            let following = pending.as_ref().map(|next| next.span.start);
            let omitted = omitted.map(|empty| {
                self.prepare_omitted_statement_tail(statement_span.end, empty, following, span.end)
            });
            let relation = if following.is_some() {
                StatementTailRelation::Statement
            } else {
                StatementTailRelation::Terminal
            };
            let suppression_tail = self.prepare_statement_suppression_tail_at(
                statement,
                statement_span,
                following.unwrap_or(span.end),
                relation,
            );
            if let Some(tail) = suppression_tail.as_ref()
                && tail.modes.following == ContentMode::Verbatim
                && let Some(next) = pending.as_mut()
            {
                next.mode = ContentMode::Verbatim;
                next.source_gap = tail.following_gap;
            }
            let source_terminator = if annotation_tail.is_some() {
                self.statement_source_terminator(statement_span)
            } else {
                suppression_tail.as_ref().and_then(|tail| tail.terminator)
            };
            let source_packet = suppression_tail
                .as_ref()
                .and_then(|tail| tail.source_packet);
            let punctuation = if source_packet.is_some() {
                StatementPunctuation::SourcePacket
            } else if let Some(terminator) = source_terminator {
                StatementPunctuation::Source(terminator)
            } else {
                StatementPunctuation::Canonical(self.options.semicolons)
            };
            let mode = if source_mode == ContentMode::Verbatim
                || leading_mode == ContentMode::Verbatim
                || annotation_tail
                    .as_ref()
                    .is_some_and(|tail| tail.suppression.current == ContentMode::Verbatim)
            {
                ContentMode::Verbatim
            } else {
                suppression_tail
                    .as_ref()
                    .map_or(ContentMode::Formatted, |tail| tail.modes.current)
            };
            let leading = self.comments.take(leading.len());
            let mut guarded = previous.is_some_and(StatementBoundary::guards_expression);
            if let Some(boundary) = previous {
                let end = boundary.end();
                let next_start = leading
                    .first()
                    .map_or(statement_span.start, |comment| comment.span.start);
                let constraint =
                    source_gap.unwrap_or_else(|| self.line_constraint(end, next_start));
                if constraint == LineConstraint::SameLine
                    && self.options.semicolons == Semicolons::AsNeeded
                    && !boundary.has_terminator()
                {
                    writer.text(";");
                    guarded = true;
                }
                let separator = if matches!(boundary, StatementBoundary::SourceTerminator { end, terminator } if end == terminator + 1)
                {
                    FlatSeparator::Empty
                } else {
                    FlatSeparator::Space
                };
                constraint.write(writer, separator, Writer::hard_line);
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
            let needs_guard = !guarded
                && self.options.semicolons == Semicolons::AsNeeded
                && expression.is_some_and(|expression| {
                    self.statement_expression_needs_guard(expression, mode)
                });
            let protected = leading.iter().any(|&comment| {
                self.comment_has_target_annotation(comment)
                    || mode == ContentMode::Verbatim && is_suppression_comment(ast, comment)
            });
            if needs_guard && protected {
                writer.text(";");
                guarded = true;
                if mode == ContentMode::Verbatim
                    && let Some(marker) = leading
                        .iter()
                        .find(|&&comment| is_suppression_comment(ast, comment))
                {
                    if self.source_position_starts_line(marker.span.start) {
                        writer.hard_line();
                    } else if marker.kind == CommentKind::Line {
                        writer.space();
                    }
                }
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
            let content = if mode == ContentMode::Verbatim {
                StatementMode::Verbatim(
                    suppression_tail
                        .as_ref()
                        .map_or_else(|| self.statement_content(statement), |tail| tail.content),
                )
            } else {
                StatementMode::Formatted
            };
            let mut boundary = match statement {
                StatementItem::Statement(node) => {
                    self.format_statement(node, layout, content, punctuation, writer)?
                }
                StatementItem::Directive(node) => {
                    let expression = node.expression(ast);
                    if mode == ContentMode::Verbatim {
                        self.format_verbatim(expression.span(ast), writer)?;
                    } else {
                        self.format_string_literal(expression, StringContext::Directive, writer);
                    }
                    self.finish_statement(expression.span(ast).end, punctuation, writer)?
                }
            };
            let trailing_start = if let Some(packet) = source_packet {
                let start = packet.span.start.max(boundary.end());
                self.format_verbatim(Span::new(start, packet.span.end), writer)?;
                if let Some(next) = pending.as_mut() {
                    next.source_gap = Some(packet.following);
                }
                boundary = boundary.with_source_packet(packet.span.end);
                packet.span.end
            } else if let Some(tail) = annotation_tail {
                if let Some(next) = pending.as_mut()
                    && tail.suppression.following == ContentMode::Verbatim
                {
                    next.mode = ContentMode::Verbatim;
                    next.source_gap = tail.following_gap;
                }
                self.comments.claim(tail.comments)?;
                self.format_comment_run(
                    tail.comments,
                    CommentRunLayout::SourceAfter(statement_span.end),
                    writer,
                )?;
                tail.end
            } else {
                boundary.end()
            };
            let end = if let Some((comments, _)) = omitted
                && source_packet.is_none()
            {
                self.format_omitted_statement_tail(comments, trailing_start, writer)?
            } else {
                self.format_trailing_comments(
                    Span::new(trailing_start, following.unwrap_or(span.end)),
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

    fn prepare_sequence_statement(
        &self,
        item: StatementItem,
        following: Option<StatementItem>,
    ) -> PreparedStatement<'ast> {
        let span = item.span(self.ast);
        let annotation_tail = if (self.has_target_annotations || self.has_suppression)
            && item.is_nonempty(self.ast)
        {
            self.prepare_statement_annotation_tail(span, following)
        } else {
            None
        };
        PreparedStatement {
            item,
            span,
            mode: ContentMode::Formatted,
            source_gap: None,
            annotation_tail,
        }
    }

    fn prepare_statement_annotation_tail(
        &self,
        statement_span: Span,
        following: Option<StatementItem>,
    ) -> Option<StatementAnnotationTail<'ast>> {
        let following = following?;
        let target = if following.is_nonempty(self.ast) {
            if self
                .ast
                .source()
                .as_bytes()
                .get(statement_span.end as usize - 1)
                != Some(&b';')
            {
                return None;
            }
            None
        } else {
            if !self.has_suppression {
                return None;
            }
            Some(statement_span)
        };
        let following_span = following.span(self.ast);
        let span = Span::new(statement_span.end, following_span.start);
        let comments = self.comments.in_range(span);
        let mut end = span.start;
        let mut count = 0;
        for &comment in comments {
            let gap = self.source(Span::new(end, comment.span.start));
            if target.is_some() && self.line_breaks(end, comment.span.start) != 0
                || !gap.chars().all(is_source_whitespace)
            {
                break;
            }
            end = comment.span.end;
            count += 1;
            if target.is_some() && comment.kind == CommentKind::Line {
                break;
            }
        }
        let comments = &comments[..count];
        let suppression = if target.is_some() {
            StatementTailModes {
                current: self.content_mode(comments),
                following: ContentMode::Formatted,
            }
        } else {
            self.statement_annotation_tail_modes(comments, span.end)
        };
        let previous_ignore = target.is_some() || suppression.current == ContentMode::Verbatim;
        if target.is_none() {
            // Block annotations belong to the following head, including its ASI guard.
            if comments
                .iter()
                .all(|comment| comment.kind == CommentKind::Block)
                && !previous_ignore
            {
                return None;
            }
            let optimizer = comments
                .iter()
                .any(|&comment| self.comment_is_optimizer_annotation(comment));
            if !optimizer && !previous_ignore {
                return None;
            }
        }

        let binding = self.comment_run_binding(comments.iter().copied(), |comment| {
            let semantics = self.comment_semantics(comment);
            if semantics.suppression || semantics.has_target_annotation() {
                CommentBinding::SourceBound
            } else {
                CommentBinding::Unbound
            }
        });
        if binding.allows_relocation() {
            return None;
        }
        let following_gap = if suppression.following == ContentMode::Verbatim {
            Some(
                match self.line_breaks(end, following.span(self.ast).start) {
                    0 => LineConstraint::SameLine,
                    count => LineConstraint::NewLine(count),
                },
            )
        } else {
            None
        };
        Some(StatementAnnotationTail {
            comments,
            end,
            suppression,
            following_gap,
        })
    }

    fn format_statement(
        &mut self,
        statement: Stmt,
        layout: StatementLayout,
        content: StatementMode,
        punctuation: StatementPunctuation,
        writer: &mut Writer<'ast>,
    ) -> Result<StatementBoundary, FormatError> {
        if let StatementMode::Verbatim(content) = content {
            if !matches!(
                self.ast.stmt_data(statement),
                StmtData::ExpressionStatement(_)
                    | StmtData::DebuggerStatement(_)
                    | StmtData::BlockStatement(_)
            ) {
                return Err(self.unsupported("statement", statement));
            }
            if let StmtData::ExpressionStatement(node) = self.ast.stmt_data(statement) {
                let expression = node.expression(self.ast);
                if !layout.guarded
                    && self.options.semicolons == Semicolons::AsNeeded
                    && self.statement_expression_needs_guard(expression, ContentMode::Verbatim)
                {
                    writer.text(";");
                }
            }
            let end = content.end();
            let parenthesize = matches!(self.ast.stmt_data(statement), StmtData::ExpressionStatement(node) if matches!(self.ast.expr_data(node.expression(self.ast)), ExprData::StringLiteral(_)));
            if parenthesize {
                writer.text("(");
            }
            self.format_verbatim(Span::new(statement.span(self.ast).start, end), writer)?;
            if parenthesize {
                writer.text(")");
            }
            return match content {
                StatementContent::Semicolon(_) => self.finish_statement(end, punctuation, writer),
                StatementContent::Complete(_) => Ok(StatementBoundary::SourceEnd {
                    end,
                    has_terminator: false,
                }),
            };
        }
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
        self.finish_statement(content_end, punctuation, writer)
    }

    fn finish_statement(
        &mut self,
        content_end: u32,
        punctuation: StatementPunctuation,
        writer: &mut Writer<'ast>,
    ) -> Result<StatementBoundary, FormatError> {
        if matches!(punctuation, StatementPunctuation::SourcePacket) {
            return Ok(StatementBoundary::SourceEnd {
                end: content_end,
                has_terminator: false,
            });
        }
        if let StatementPunctuation::Source(terminator) = punctuation {
            let source_end = terminator.start + 1;
            let comments = self.comments.between(Span::new(content_end, source_end))?;
            self.comments.claim(comments)?;
            self.format_comment_run(comments, CommentRunLayout::SourceAfter(content_end), writer)?;
            let end = comments
                .last()
                .map_or(content_end, |comment| comment.span.end);
            self.line_constraint(end, terminator.start).write(
                writer,
                FlatSeparator::Empty,
                |writer| {
                    if self.line_breaks(end, terminator.start) != 0
                        || comments
                            .last()
                            .is_some_and(|comment| comment.kind == CommentKind::Line)
                    {
                        writer.hard_line();
                    }
                },
            );
            writer.text(";");
            return Ok(if self.line_breaks(content_end, terminator.start) != 0 {
                StatementBoundary::SourceTerminator {
                    end: source_end,
                    terminator: terminator.start,
                }
            } else {
                StatementBoundary::SourceEnd {
                    end: source_end,
                    has_terminator: true,
                }
            });
        }
        let has_terminator = matches!(
            punctuation,
            StatementPunctuation::Canonical(Semicolons::Always)
        );
        if has_terminator {
            writer.text(";");
        }
        Ok(StatementBoundary::SourceEnd {
            end: content_end,
            has_terminator,
        })
    }

    fn statement_expression_needs_guard(
        &self,
        expression: swc_next_ecma_ast::Expr,
        mode: ContentMode,
    ) -> bool {
        if mode == ContentMode::Verbatim {
            let source = self.source(expression.span(self.ast));
            return matches!(self.ast.expr_data(expression), ExprData::StringLiteral(_))
                || source.starts_with(['(', '[', '/', '`', '<'])
                || source.starts_with(['+', '-'])
                    && !source.starts_with("++")
                    && !source.starts_with("--");
        }
        self.string_expression_literal(expression).is_some()
            || matches!(self.ast.expr_data(expression), ExprData::RegExpLiteral(_))
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
        if !has_items && self.content_mode(self.comments.in_range(content)) == ContentMode::Verbatim
        {
            return self.format_verbatim(span, writer);
        }
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
