//! Basic statement bodies and their enclosing source boundaries.

mod tails;

use swc_next_ecma_ast::{
    Ast, BlockStatement, Comment, CommentKind, DeclData, Directive, EmptyStatement, ExprData,
    FunctionBody, GetSpan, Span, Stmt, StmtData,
};

use self::tails::{StatementContent, StatementTailModes, StatementTailRelation};
use super::Formatter;
use super::comments::{CommentBinding, CommentRunLayout, ExpressionTail, NodeHead};
use super::diagnostics::{FlatSeparator, LineConstraint};
use super::expressions::{ExpressionContext, ExpressionPosition, ExpressionStart};
use super::literals::StringContext;
use super::source::is_source_whitespace;
use super::suppression::{ContentMode, is_suppression_comment};
use crate::{FormatError, Semicolons, document::Writer};

#[derive(Clone, Copy)]
pub(super) enum StatementSequenceKind<'ast> {
    Program,
    Block,
    Function {
        incoming: Option<IncomingStatement>,
        leading: StatementBodyLeading<'ast>,
    },
}

/// A claimed ordinary packet transferred to its actual statement body.
#[derive(Clone, Copy)]
pub(super) struct StatementBodyLeading<'ast> {
    comments: &'ast [Comment],
    target: Option<Stmt>,
}

impl<'ast> StatementBodyLeading<'ast> {
    pub(super) fn new(comments: &'ast [Comment], target: Option<Stmt>) -> Self {
        Self { comments, target }
    }

    pub(super) fn empty() -> Self {
        Self {
            comments: &[],
            target: None,
        }
    }

    pub(super) fn comments(&self) -> &'ast [Comment] {
        self.comments
    }

    pub(super) fn take_for(&mut self, statement: Stmt) -> &'ast [Comment] {
        if self.target == Some(statement) {
            std::mem::take(&mut self.comments)
        } else {
            &[]
        }
    }

    pub(super) fn take_dangling(&mut self) -> &'ast [Comment] {
        if self.target.is_none() {
            std::mem::take(&mut self.comments)
        } else {
            &[]
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct IncomingStatement {
    pub(super) span: Span,
    pub(super) mode: ContentMode,
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
        kind: StatementSequenceKind<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<StatementSequence, FormatError> {
        let ast = self.ast;
        let (incoming, mut body_leading) = match kind {
            StatementSequenceKind::Function { incoming, leading } => (incoming, leading),
            _ => (None, StatementBodyLeading::empty()),
        };
        let mut first_statement = true;
        let mut previous = previous_end.map(|end| StatementBoundary::SourceEnd {
            end,
            has_terminator: false,
        });
        let mut previous_span_end = previous_end.unwrap_or(span.start);
        let mut previous_omitted: Option<Span> = None;
        let mut pending = statements.next().map(|item| {
            self.prepare_sequence_statement(item, statements.peek().copied(), incoming)
        });
        while let Some(prepared) = pending {
            pending = statements.next().map(|item| {
                self.prepare_sequence_statement(item, statements.peek().copied(), incoming)
            });
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
                pending = statements.next().map(|item| {
                    self.prepare_sequence_statement(item, statements.peek().copied(), incoming)
                });
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
            if first_statement
                && matches!(kind, StatementSequenceKind::Program)
                && self.has_file_directives
                && leading
                    .iter()
                    .any(|&comment| self.comment_is_file_directive(comment))
                && leading
                    .iter()
                    .any(|&comment| self.comment_has_target_annotation(comment))
            {
                guarded = true;
            }
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
            let needs_guard = mode == ContentMode::Verbatim
                && !guarded
                && self.options.semicolons == Semicolons::AsNeeded
                && expression
                    .map(|expression| self.statement_expression_needs_guard(expression, mode))
                    .transpose()?
                    .unwrap_or(false);
            let protected = leading.iter().any(|&comment| {
                self.comment_has_target_annotation(comment)
                    || mode == ContentMode::Verbatim && is_suppression_comment(ast, comment)
            });
            if needs_guard && protected && mode == ContentMode::Verbatim {
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
            let layout = StatementLayout { guarded };
            let expression_head = expression.is_some() && mode == ContentMode::Formatted;
            let transferred = match statement {
                StatementItem::Statement(node) => body_leading.take_for(node),
                StatementItem::Directive(_) => &[],
            };
            if !transferred.is_empty() {
                self.format_comment_run(
                    transferred,
                    CommentRunLayout::Leading(statement_span.start),
                    writer,
                )?;
            }
            let head = NodeHead {
                comments: if expression_head { leading } else { &[] },
                content_start: statement_span.start,
                layout: CommentRunLayout::Source,
                gap_end: Some(statement_span.start),
            };
            if !expression_head {
                self.format_comment_run(leading, CommentRunLayout::Source, writer)?;
                if let Some(last) = leading.last() {
                    self.comment_gap(last.span.end, statement_span.start, writer);
                }
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
            let (mut boundary, mut expression_tail) = match statement {
                StatementItem::Statement(node) => {
                    self.format_statement(node, head, layout, content, punctuation, writer)?
                }
                StatementItem::Directive(node) => {
                    let expression = node.expression(ast);
                    if mode == ContentMode::Verbatim {
                        self.format_verbatim(expression.span(ast), writer)?;
                    } else {
                        self.format_string_literal(expression, StringContext::Directive, writer);
                    }
                    (
                        self.finish_statement(expression.span(ast).end, punctuation, writer)?,
                        None,
                    )
                }
            };
            if source_packet.is_some() || annotation_tail.is_some() || omitted.is_some() {
                self.format_expression_tail(expression_tail.take(), writer)?;
            }
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
                    expression_tail,
                    writer,
                )?
            };
            first_statement = false;
            previous = Some(boundary.with_end(end));
            previous_span_end = statement_span.end;
            previous_omitted = omitted.map(|(_, gap)| gap);
        }
        let transferred = body_leading.take_dangling();
        if !transferred.is_empty() {
            if previous.is_some() {
                writer.hard_line();
            }
            self.format_comment_run(transferred, CommentRunLayout::Lines, writer)?;
        }
        let dangling = self.comments.between(Span::new(
            previous.map_or(span.start, StatementBoundary::end),
            span.end,
        ))?;
        let dangling = self.comments.take(dangling.len());
        if !transferred.is_empty() && !dangling.is_empty() {
            writer.hard_line();
        }
        if let Some(first) = dangling.first()
            && let Some(boundary) = previous
        {
            self.comment_gap(boundary.end(), first.span.start, writer);
        }
        self.format_comment_run(
            dangling,
            if previous.is_none()
                && matches!(
                    kind,
                    StatementSequenceKind::Block | StatementSequenceKind::Function { .. }
                )
            {
                CommentRunLayout::CanonicalDangling
            } else {
                CommentRunLayout::Source
            },
            writer,
        )?;
        Ok(StatementSequence {
            has_content: previous.is_some() || !dangling.is_empty() || !transferred.is_empty(),
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
            .comment_binding(comments, Span::new(start, end), None)
            .allows_relocation(None)
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
        incoming: Option<IncomingStatement>,
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
            mode: incoming
                .filter(|target| target.span == span)
                .map_or(ContentMode::Formatted, |target| target.mode),
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
        if binding.allows_relocation(None) {
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
        head: NodeHead<'ast>,
        layout: StatementLayout,
        content: StatementMode,
        punctuation: StatementPunctuation,
        writer: &mut Writer<'ast>,
    ) -> Result<(StatementBoundary, Option<ExpressionTail<'ast>>), FormatError> {
        if let StatementMode::Verbatim(content) = content {
            if !matches!(
                self.ast.stmt_data(statement),
                StmtData::ExpressionStatement(_)
                    | StmtData::DebuggerStatement(_)
                    | StmtData::BlockStatement(_)
                    | StmtData::ReturnStatement(_)
                    | StmtData::ThrowStatement(_)
                    | StmtData::Declaration(_)
            ) {
                return Err(self.unsupported("statement", statement));
            }
            if let StmtData::ExpressionStatement(node) = self.ast.stmt_data(statement) {
                let expression = node.expression(self.ast);
                if !layout.guarded
                    && self.options.semicolons == Semicolons::AsNeeded
                    && self.statement_expression_needs_guard(expression, ContentMode::Verbatim)?
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
            let boundary = match content {
                StatementContent::Semicolon(_) => self.finish_statement(end, punctuation, writer),
                StatementContent::Complete(_) => Ok(StatementBoundary::SourceEnd {
                    end,
                    has_terminator: false,
                }),
            }?;
            return Ok((boundary, None));
        }
        let mut tail = None;
        let content_end = match self.ast.stmt_data(statement) {
            StmtData::ExpressionStatement(node) => {
                let expression = node.expression(self.ast);
                let context = ExpressionContext::at(ExpressionPosition::Statement)
                    .with_expression_start(if layout.guarded {
                        ExpressionStart::StatementGuarded
                    } else {
                        ExpressionStart::StatementPending
                    });
                tail = self
                    .format_expression_with_leading_comments(expression, head, context, writer)?;
                expression.span(self.ast).end
            }
            StmtData::DebuggerStatement(node) => {
                writer.text("debugger");
                node.span(self.ast).start + "debugger".len() as u32
            }
            StmtData::ReturnStatement(node) => {
                let end = node.span(self.ast).start + 6;
                if let Some(argument) = node.argument(self.ast) {
                    tail = self.format_restricted_argument(
                        end,
                        argument,
                        |writer| writer.text("return"),
                        writer,
                    )?;
                    argument.span(self.ast).end
                } else {
                    writer.text("return");
                    end
                }
            }
            StmtData::ThrowStatement(node) => {
                let argument = node.argument(self.ast);
                tail = self.format_restricted_argument(
                    node.span(self.ast).start + 5,
                    argument,
                    |writer| writer.text("throw"),
                    writer,
                )?;
                argument.span(self.ast).end
            }
            StmtData::Declaration(declaration) => {
                let DeclData::Function(function) = self.ast.decl_data(declaration) else {
                    return Err(self.unsupported("declaration", declaration));
                };
                self.format_function(function, writer)?;
                return Ok((
                    StatementBoundary::SourceEnd {
                        end: function.span(self.ast).end,
                        has_terminator: false,
                    },
                    None,
                ));
            }
            StmtData::BlockStatement(block) => {
                self.format_block(block, writer)?;
                return Ok((
                    StatementBoundary::SourceEnd {
                        end: block.span(self.ast).end,
                        has_terminator: false,
                    },
                    None,
                ));
            }
            _ => return Err(self.unsupported("statement", statement)),
        };
        let boundary = self.finish_statement(content_end, punctuation, writer)?;
        Ok((boundary, tail))
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
    ) -> Result<bool, FormatError> {
        self.expression_needs_asi_guard(
            expression,
            ExpressionContext::at(ExpressionPosition::Statement).with_content_mode(mode),
        )
    }

    pub(in crate::formatter) fn function_body_requires_break(&self, body: FunctionBody) -> bool {
        !body.directives(self.ast).is_empty()
            || body.body(self.ast).iter().any(|id| {
                StatementItem::Statement(self.ast.get_node_in_sub_range(id)).is_nonempty(self.ast)
            })
    }
    pub(super) fn format_function_body(
        &mut self,
        body: FunctionBody,
        mode: ContentMode,
        incoming: Option<IncomingStatement>,
        leading: StatementBodyLeading<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        self.comments.claim(leading.comments())?;
        let ast = self.ast;
        let span = body.span(ast);
        if mode == ContentMode::Verbatim {
            return self.format_verbatim(span, writer);
        }
        let content = Span::new(span.start + 1, span.end - 1);
        let has_items = self.function_body_requires_break(body);
        let source_empty = if has_items {
            false
        } else {
            let dangling = self.comments.in_range(content);
            if self.content_mode(dangling) == ContentMode::Verbatim {
                return self.format_verbatim(span, writer);
            }
            dangling.is_empty()
        };
        let empty = source_empty && leading.comments().is_empty();
        let opening = self.opening_line_constraint(content);
        writer.text("{");
        if empty {
            opening.write(writer, FlatSeparator::Empty, |_| {});
        } else {
            let sequence = writer.indent(|writer| {
                opening.write(writer, FlatSeparator::Space, Writer::hard_line);
                self.format_statement_sequence(
                    body.directives(ast)
                        .iter()
                        .map(|id| StatementItem::Directive(ast.get_node_in_sub_range(id)))
                        .chain(
                            body.body(ast)
                                .iter()
                                .map(|id| StatementItem::Statement(ast.get_node_in_sub_range(id))),
                        )
                        .peekable(),
                    content,
                    None,
                    StatementSequenceKind::Function { incoming, leading },
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
