//! Basic statement bodies and their enclosing source boundaries.

mod control_flow;
mod tails;

use self::tails::{StatementContent, StatementTailModes, StatementTailRelation};

use swc_next_ecma_ast::{
    Ast, BlockStatement, Comment, CommentKind, DeclData, Directive, EmptyStatement, FunctionBody,
    GetSpan, Span, StaticBlock, Stmt, StmtData, SubRange,
};

use super::Formatter;
use super::comments::{CommentBinding, CommentRunLayout, ExpressionTail, NodeHead};
use super::diagnostics::{FlatSeparator, LineConstraint};
use super::expressions::{ExpressionContext, ExpressionPosition, ExpressionStart};
use super::literals::StringContext;
use super::source::is_source_whitespace;
use super::suppression::ContentMode;
use crate::{FormatError, Semicolons, document::Writer};

#[derive(Clone, Copy)]
pub(super) enum StatementSequenceKind<'ast> {
    Program {
        has_source_items: bool,
    },
    Block {
        incoming: Option<IncomingStatement>,
        leading: StatementBodyLeading<'ast>,
    },
    Function {
        incoming: Option<IncomingStatement>,
        leading: StatementBodyLeading<'ast>,
    },
    Static(Option<IncomingStatement>),
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
pub(super) enum EmptyBlock {
    Compact,
    Expanded,
}

/// Layout supplied by the real parent without bypassing statement ownership.
struct StatementLayout {
    start: ExpressionStart,
    empty_block: EmptyBlock,
}

pub(super) struct StatementSequence {
    pub(super) has_content: bool,
    end: u32,
}

#[derive(Clone, Copy)]
pub(super) enum StatementItem {
    Directive(Directive),
    Statement(Stmt),
}

enum BodySource<'ast> {
    Block {
        node: BlockStatement,
        incoming: Option<IncomingStatement>,
        leading: StatementBodyLeading<'ast>,
        empty: EmptyBlock,
    },
    Function(
        FunctionBody,
        Option<IncomingStatement>,
        StatementBodyLeading<'ast>,
    ),
    Static {
        body: StaticBlock,
        span: Span,
        incoming: Option<IncomingStatement>,
    },
}

struct StatementAnnotationTail<'ast> {
    comments: &'ast [Comment],
    end: u32,
    suppression: StatementTailModes,
    following_gap: Option<LineConstraint>,
}

#[derive(Clone, Copy)]
struct OmittedStatementTail<'ast> {
    comments: &'ast [Comment],
    binding: CommentBinding,
    source_gap: Span,
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

fn statement_body_has_items(
    ast: &Ast<'_>,
    directives: Option<SubRange<Directive>>,
    body: SubRange<Stmt>,
) -> bool {
    directives.is_some_and(|directives| !directives.is_empty())
        || body
            .iter()
            .any(|id| StatementItem::Statement(ast.get_node_in_sub_range(id)).is_nonempty(ast))
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
    Semicolon {
        end: u32,
        terminator: u32,
    },
    /// A printed source separator has already consumed the following head's guard.
    SourceTerminator {
        end: u32,
        terminator: u32,
    },
    SourceEnd {
        end: u32,
        has_terminator: bool,
    },
}

/// A compound statement forwards a pending run beyond its own layout group.
/// Source terminators consume their bound run before returning this result.
struct PrintedStatement<'ast> {
    boundary: StatementBoundary,
    pending: Option<ExpressionTail<'ast>>,
}

impl From<StatementBoundary> for PrintedStatement<'_> {
    fn from(boundary: StatementBoundary) -> Self {
        Self {
            boundary,
            pending: None,
        }
    }
}

impl StatementBoundary {
    fn end(self) -> u32 {
        match self {
            Self::Semicolon { end, .. }
            | Self::SourceTerminator { end, .. }
            | Self::SourceEnd { end, .. } => end,
        }
    }

    fn has_terminator(self) -> bool {
        match self {
            Self::Semicolon { .. } | Self::SourceTerminator { .. } => true,
            Self::SourceEnd { has_terminator, .. } => has_terminator,
        }
    }

    fn guards_expression(self) -> bool {
        // An empty body's semicolon does not consume the next printed head's guard.
        // Source packets retain their own grammar closure.
        matches!(
            self,
            Self::SourceTerminator { .. }
                | Self::SourceEnd {
                    has_terminator: true,
                    ..
                }
        )
    }

    fn with_end(self, end: u32) -> Self {
        match self {
            Self::Semicolon { terminator, .. } => Self::Semicolon { end, terminator },
            Self::SourceTerminator { terminator, .. } => Self::SourceTerminator { end, terminator },
            Self::SourceEnd { has_terminator, .. } => Self::SourceEnd {
                end,
                has_terminator,
            },
        }
    }

    fn with_source_packet(self, end: u32) -> Self {
        Self::SourceEnd {
            end,
            has_terminator: true,
        }
    }
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
        let incoming = match kind {
            StatementSequenceKind::Block { incoming, .. }
            | StatementSequenceKind::Function { incoming, .. }
            | StatementSequenceKind::Static(incoming) => incoming,
            _ => None,
        };
        let mut body_leading = match kind {
            StatementSequenceKind::Block { leading, .. }
            | StatementSequenceKind::Function { leading, .. } => leading,
            _ => StatementBodyLeading::empty(),
        };
        let mut previous = previous_end.map(|end| StatementBoundary::SourceEnd {
            end,
            has_terminator: false,
        });
        let mut first_statement = true;
        let mut previous_span_end = previous_end.unwrap_or(span.start);
        let mut previous_omitted: Option<OmittedStatementTail<'ast>> = None;
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
            let omitted = omitted.map(|statements| {
                self.prepare_omitted_statement_tail(
                    statement_span.end,
                    statements,
                    following,
                    span.end,
                )
            });
            let suppression_tail = if self.has_suppression {
                let limit = following.unwrap_or(span.end);
                let relation = if following.is_some() {
                    StatementTailRelation::Statement
                } else {
                    StatementTailRelation::Terminal
                };
                self.prepare_statement_suppression_tail_at(
                    statement,
                    statement_span,
                    limit,
                    relation,
                )
            } else {
                None
            };
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
            } else {
                match source_terminator {
                    Some(terminator) => StatementPunctuation::Source(terminator),
                    None => StatementPunctuation::Canonical(self.options.semicolons),
                }
            };
            let mode = if source_mode == ContentMode::Verbatim
                || leading_mode == ContentMode::Verbatim
                || annotation_tail
                    .as_ref()
                    .is_some_and(|tail| tail.suppression.current == ContentMode::Verbatim)
            {
                ContentMode::Verbatim
            } else if !self.has_suppression {
                ContentMode::Formatted
            } else {
                suppression_tail
                    .as_ref()
                    .map_or(ContentMode::Formatted, |tail| tail.modes.current)
            };
            let leading = self.comments.take(leading.len());
            let mut expression_start = ExpressionStart::StatementPending;
            if first_statement
                && matches!(kind, StatementSequenceKind::Program { .. })
                && self.has_file_directives
                && leading
                    .iter()
                    .any(|&comment| self.comment_is_file_directive(comment))
                && leading
                    .iter()
                    .any(|&comment| self.comment_has_target_annotation(comment))
            {
                // Program start already separates the first statement. Keep its header intact.
                expression_start = ExpressionStart::StatementGuarded;
            }
            if let Some(boundary) = previous {
                if boundary.guards_expression() {
                    expression_start = ExpressionStart::StatementGuarded;
                }
                let next_start = leading
                    .first()
                    .map_or(statement_span.start, |comment| comment.span.start);
                let constraint =
                    source_gap.unwrap_or_else(|| self.line_constraint(boundary.end(), next_start));
                if constraint == LineConstraint::SameLine
                    && self.options.semicolons == Semicolons::AsNeeded
                    && !boundary.has_terminator()
                    && statement.is_nonempty(ast)
                {
                    writer.text(";");
                    expression_start = ExpressionStart::StatementGuarded;
                }
                let separator = if matches!(boundary, StatementBoundary::Semicolon { end, terminator } | StatementBoundary::SourceTerminator { end, terminator } if end == terminator + 1)
                {
                    FlatSeparator::Empty
                } else {
                    FlatSeparator::Space
                };
                constraint.write(writer, separator, Writer::hard_line);
                if constraint == LineConstraint::Unconstrained {
                    let empty_line = if let Some(tail) = previous_omitted {
                        self.next_line_is_empty(tail.source_gap.start, tail.source_gap.end)
                    } else {
                        let gap_start = previous_span_end.max(boundary.end());
                        let before_separator = boundary.end() < gap_start
                            && self.next_line_is_empty(boundary.end(), gap_start.min(next_start));
                        // Comments inside the previous statement do not delimit its following gap.
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
                comments: leading,
                content_start: statement_span.start,
                layout: CommentRunLayout::Source,
                gap_end: leading.last().map(|_| statement_span.start),
            };
            let printed = match statement {
                StatementItem::Directive(directive) => {
                    self.format_node_head(head, expression_start, None, writer)?;
                    let expression = directive.expression(ast);
                    if mode == ContentMode::Verbatim {
                        self.format_verbatim(expression.span(ast), writer)?;
                    } else {
                        self.format_string_literal(expression, StringContext::Directive, writer);
                    }
                    self.finish_statement(expression.span(ast).end, None, punctuation, writer)?
                }
                StatementItem::Statement(statement) => {
                    let content = if mode == ContentMode::Verbatim {
                        let content = suppression_tail.as_ref().map_or_else(
                            || self.statement_content(StatementItem::Statement(statement)),
                            |tail| tail.content,
                        );
                        StatementMode::Verbatim(content)
                    } else {
                        StatementMode::Formatted
                    };
                    self.format_prepared_statement(
                        statement,
                        statement_span,
                        StatementLayout {
                            start: expression_start,
                            empty_block: EmptyBlock::Expanded,
                        },
                        content,
                        head,
                        punctuation,
                        writer,
                    )?
                }
            };
            let (mut boundary, pending_tail) =
                if source_packet.is_some() || annotation_tail.is_some() {
                    (self.flush_statement_tail(printed, writer)?, None)
                } else {
                    (printed.boundary, printed.pending)
                };
            first_statement = false;
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
            let end = if let Some(tail) = omitted
                && source_packet.is_none()
            {
                self.format_omitted_statement_tail(tail, trailing_start, pending_tail, writer)?
            } else {
                self.format_trailing_comments(
                    Span::new(trailing_start, following.unwrap_or(span.end)),
                    statement_span.end,
                    following.is_some(),
                    pending_tail,
                    writer,
                )?
            };
            previous_span_end = statement_span.end;
            previous = Some(boundary.with_end(end));
            previous_omitted = omitted;
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
        let layout = match (previous, kind) {
            (
                None,
                StatementSequenceKind::Block { .. }
                | StatementSequenceKind::Function { .. }
                | StatementSequenceKind::Static(_),
            ) => CommentRunLayout::CanonicalDangling,
            (
                None,
                StatementSequenceKind::Program {
                    has_source_items: true,
                },
            ) => CommentRunLayout::Lines,
            _ => CommentRunLayout::Source,
        };
        self.format_comment_run(dangling, layout, writer)?;
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
    ) -> OmittedStatementTail<'ast> {
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
        let comments = &comments[..count];
        let end = comments.last().map_or(start, |comment| comment.span.end);
        OmittedStatementTail {
            comments,
            binding: self.comment_binding(comments, Span::new(start, end), None),
            source_gap: Span::new(gap_start, limit),
        }
    }

    fn format_omitted_statement_tail(
        &mut self,
        tail: OmittedStatementTail<'ast>,
        start: u32,
        pending: Option<ExpressionTail<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<u32, FormatError> {
        let source_end = pending.as_ref().map_or(start, |pending| pending.source_end);
        let end = tail
            .comments
            .last()
            .map_or(start, |comment| start.max(comment.span.end));
        let end = pending
            .as_ref()
            .and_then(|pending| pending.comments.last())
            .map_or(end, |comment| end.max(comment.span.end));
        let comments = self.comments.between(Span::new(source_end, end))?;
        self.comments.claim(comments)?;
        let binding =
            tail.binding
                .combine(self.comment_binding(comments, Span::new(source_end, end), None));
        if !binding.allows_relocation(None) {
            self.format_comment_run(comments, CommentRunLayout::SourceAfter(source_end), writer)?;
            return Ok(end);
        }
        let inherited = pending.as_ref().map_or(0, |pending| pending.comments.len());
        let (inherited, comments) = comments.split_at(inherited);
        self.format_comment_run(inherited, CommentRunLayout::SourceAfter(source_end), writer)?;
        self.format_trailing_comment_run(comments, inherited.last().copied(), None, writer)?;
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
                enclosing: ContentMode::Formatted,
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
        layout: StatementLayout,
        mode: ContentMode,
        head: NodeHead<'ast>,
        punctuation: StatementPunctuation,
        writer: &mut Writer<'ast>,
    ) -> Result<PrintedStatement<'ast>, FormatError> {
        let span = statement.span(self.ast);
        let content = if mode == ContentMode::Verbatim {
            StatementMode::Verbatim(self.statement_content(StatementItem::Statement(statement)))
        } else {
            StatementMode::Formatted
        };
        self.format_prepared_statement(statement, span, layout, content, head, punctuation, writer)
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "Carry the prepared statement envelope instead of rediscovering it during printing."
    )]
    fn format_prepared_statement(
        &mut self,
        statement: Stmt,
        span: Span,
        layout: StatementLayout,
        content: StatementMode,
        head: NodeHead<'ast>,
        punctuation: StatementPunctuation,
        writer: &mut Writer<'ast>,
    ) -> Result<PrintedStatement<'ast>, FormatError> {
        let mode = match content {
            StatementMode::Formatted => ContentMode::Formatted,
            StatementMode::Verbatim(_) => ContentMode::Verbatim,
        };
        let data = self.ast.stmt_data(statement);
        if let StmtData::ExpressionStatement(node) = data {
            let expression = node.expression(self.ast);
            let tail = self.format_expression_with_leading_comments(
                expression,
                head,
                ExpressionContext::at(ExpressionPosition::Statement)
                    .with_expression_start(layout.start)
                    .with_content_mode(mode),
                None,
                writer,
            )?;
            let content_end = expression.span(self.ast).end;
            return self.finish_statement(content_end, tail, punctuation, writer);
        }
        self.format_node_head(head, layout.start, None, writer)?;
        match data {
            StmtData::EmptyStatement(node) => {
                writer.text(";");
                Ok(StatementBoundary::Semicolon {
                    end: node.span(self.ast).end,
                    terminator: node.span(self.ast).end - 1,
                }
                .into())
            }
            StmtData::DebuggerStatement(node) => {
                writer.text("debugger");
                self.finish_statement(
                    node.span(self.ast).start + "debugger".len() as u32,
                    None,
                    punctuation,
                    writer,
                )
            }
            StmtData::ReturnStatement(node) if mode == ContentMode::Formatted => {
                let span = node.span(self.ast);
                let keyword_end = span.start + "return".len() as u32;
                let Some(argument) = node.argument(self.ast) else {
                    writer.text("return");
                    return self.finish_statement(keyword_end, None, punctuation, writer);
                };
                let tail = self.format_restricted_argument(
                    keyword_end,
                    argument,
                    |writer| writer.text("return"),
                    writer,
                )?;
                self.finish_statement(argument.span(self.ast).end, tail, punctuation, writer)
            }
            StmtData::ThrowStatement(node) if mode == ContentMode::Formatted => {
                let span = node.span(self.ast);
                let argument = node.argument(self.ast);
                let tail = self.format_restricted_argument(
                    span.start + "throw".len() as u32,
                    argument,
                    |writer| writer.text("throw"),
                    writer,
                )?;
                self.finish_statement(argument.span(self.ast).end, tail, punctuation, writer)
            }
            StmtData::Declaration(declaration) if mode == ContentMode::Formatted => {
                match self.ast.decl_data(declaration) {
                    DeclData::Function(function) => {
                        self.format_function(function, writer)?;
                        Ok(StatementBoundary::SourceEnd {
                            end: function.span(self.ast).end,
                            has_terminator: false,
                        }
                        .into())
                    }
                    DeclData::Class(class) => {
                        self.format_class(class, ExpressionPosition::Statement, writer)?;
                        Ok(StatementBoundary::SourceEnd {
                            end: class.span(self.ast).end,
                            has_terminator: false,
                        }
                        .into())
                    }
                    DeclData::VariableDeclaration(variable) => {
                        let (end, tail) = self.format_variable_declaration(variable, writer)?;
                        self.finish_statement(end, tail, punctuation, writer)
                    }
                    _ => Err(self.unsupported("declaration", declaration)),
                }
            }
            StmtData::BreakStatement(node) if mode == ContentMode::Formatted => self
                .format_jump_statement(
                    control_flow::JumpStatement::Break(node),
                    punctuation,
                    writer,
                ),
            StmtData::ContinueStatement(node) if mode == ContentMode::Formatted => self
                .format_jump_statement(
                    control_flow::JumpStatement::Continue(node),
                    punctuation,
                    writer,
                ),
            StmtData::LabeledStatement(node) if mode == ContentMode::Formatted => {
                self.format_labeled_statement(node, punctuation, writer)
            }
            StmtData::WhileStatement(node) if mode == ContentMode::Formatted => {
                self.format_while_statement(node, punctuation, writer)
            }
            StmtData::DoWhileStatement(node) if mode == ContentMode::Formatted => {
                self.format_do_while_statement(node, punctuation, writer)
            }
            StmtData::ForStatement(node) if mode == ContentMode::Formatted => {
                self.format_for_statement(node, punctuation, writer)
            }
            StmtData::ForInStatement(node) if mode == ContentMode::Formatted => self
                .format_iteration_statement(
                    control_flow::IterationStatement::In(node),
                    punctuation,
                    writer,
                ),
            StmtData::ForOfStatement(node) if mode == ContentMode::Formatted => self
                .format_iteration_statement(
                    control_flow::IterationStatement::Of(node),
                    punctuation,
                    writer,
                ),
            StmtData::IfStatement(node) if mode == ContentMode::Formatted => {
                self.format_if_statement(node, punctuation, writer)
            }
            StmtData::BlockStatement(block) => {
                if mode == ContentMode::Verbatim {
                    self.format_verbatim(block.span(self.ast), writer)?;
                } else {
                    self.format_block(block, layout.empty_block, writer)?;
                }
                Ok(StatementBoundary::SourceEnd {
                    end: block.span(self.ast).end,
                    has_terminator: false,
                }
                .into())
            }
            _ => {
                let StatementMode::Verbatim(content) = content else {
                    return Err(self.unsupported("statement", statement));
                };
                if !matches!(
                    data,
                    StmtData::ReturnStatement(_)
                        | StmtData::ThrowStatement(_)
                        | StmtData::Declaration(_)
                        | StmtData::IfStatement(_)
                        | StmtData::ForStatement(_)
                        | StmtData::ForInStatement(_)
                        | StmtData::ForOfStatement(_)
                        | StmtData::WhileStatement(_)
                        | StmtData::DoWhileStatement(_)
                        | StmtData::LabeledStatement(_)
                        | StmtData::BreakStatement(_)
                        | StmtData::ContinueStatement(_)
                ) {
                    return Err(self.unsupported("statement", statement));
                }
                self.format_verbatim_statement(span, content, punctuation, writer)
            }
        }
    }

    fn format_verbatim_statement(
        &mut self,
        span: Span,
        content: StatementContent,
        punctuation: StatementPunctuation,
        writer: &mut Writer<'ast>,
    ) -> Result<PrintedStatement<'ast>, FormatError> {
        match content {
            StatementContent::Semicolon(end) => {
                let punctuation = if self.has_suppression
                    && matches!(punctuation, StatementPunctuation::Canonical(_))
                    && let Some(terminator) = self.statement_source_terminator(span)
                    && self.content_mode(self.comments.in_range(Span::new(end, terminator.start)))
                        == ContentMode::Verbatim
                {
                    StatementPunctuation::Source(terminator)
                } else {
                    punctuation
                };
                self.format_verbatim(Span::new(span.start, end), writer)?;
                self.finish_statement(end, None, punctuation, writer)
            }
            StatementContent::Complete(_) => {
                self.format_verbatim(span, writer)?;
                Ok(StatementBoundary::SourceEnd {
                    end: span.end,
                    has_terminator: self.statement_source_terminator(span).is_some(),
                }
                .into())
            }
        }
    }

    fn finish_statement(
        &mut self,
        content_end: u32,
        tail: Option<ExpressionTail<'ast>>,
        punctuation: StatementPunctuation,
        writer: &mut Writer<'ast>,
    ) -> Result<PrintedStatement<'ast>, FormatError> {
        if matches!(punctuation, StatementPunctuation::SourcePacket) {
            let end = tail
                .as_ref()
                .and_then(|tail| tail.comments.last())
                .map_or(content_end, |comment| comment.span.end);
            self.format_expression_tail(tail, writer)?;
            return Ok(StatementBoundary::SourceEnd {
                end,
                has_terminator: false,
            }
            .into());
        }
        if let StatementPunctuation::Source(terminator) = punctuation {
            let source_end = terminator.start + 1;
            let tail_start = tail.as_ref().map_or(content_end, |tail| tail.source_end);
            let source_comments = self.comments.in_range(Span::new(tail_start, source_end));
            let printed_end = tail
                .as_ref()
                .and_then(|tail| tail.comments.last())
                .map_or(content_end, |comment| comment.span.end);
            self.format_expression_tail(tail, writer)?;
            let comments = self.comments.between(Span::new(content_end, source_end))?;
            self.comments.claim(comments)?;
            self.format_comment_run(comments, CommentRunLayout::SourceAfter(printed_end), writer)?;
            let last = comments.last().or_else(|| source_comments.last());
            let end = last.map_or(content_end, |comment| comment.span.end);
            self.line_constraint(end, terminator.start).write(
                writer,
                FlatSeparator::Empty,
                |writer| {
                    if self.line_breaks(end, terminator.start) != 0
                        || last.is_some_and(|comment| comment.kind == CommentKind::Line)
                    {
                        writer.hard_line();
                    }
                },
            );
            writer.text(";");
            let boundary = if self.line_breaks(content_end, terminator.start) != 0 {
                StatementBoundary::SourceTerminator {
                    end: source_end,
                    terminator: terminator.start,
                }
            } else {
                StatementBoundary::SourceEnd {
                    end: source_end,
                    has_terminator: true,
                }
            };
            return Ok(boundary.into());
        }
        if matches!(
            punctuation,
            StatementPunctuation::Canonical(Semicolons::Always)
        ) {
            writer.text(";");
        }
        Ok(PrintedStatement {
            boundary: StatementBoundary::SourceEnd {
                end: content_end,
                has_terminator: matches!(
                    punctuation,
                    StatementPunctuation::Canonical(Semicolons::Always)
                ),
            },
            pending: tail,
        })
    }

    fn flush_statement_tail(
        &mut self,
        printed: PrintedStatement<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<StatementBoundary, FormatError> {
        let end = printed
            .pending
            .as_ref()
            .and_then(|tail| tail.comments.last())
            .map_or(printed.boundary.end(), |comment| {
                printed.boundary.end().max(comment.span.end)
            });
        self.format_expression_tail(printed.pending, writer)?;
        Ok(printed.boundary.with_end(end))
    }

    fn format_block(
        &mut self,
        block: BlockStatement,
        empty: EmptyBlock,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        self.format_compound_block(
            block,
            ContentMode::Formatted,
            None,
            StatementBodyLeading::empty(),
            empty,
            writer,
        )
    }

    fn format_compound_block(
        &mut self,
        node: BlockStatement,
        mode: ContentMode,
        incoming: Option<IncomingStatement>,
        leading: StatementBodyLeading<'ast>,
        empty: EmptyBlock,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        self.format_braced_body(
            BodySource::Block {
                node,
                incoming,
                leading,
                empty,
            },
            mode,
            writer,
        )
    }

    pub(in crate::formatter) fn function_body_requires_break(&self, body: FunctionBody) -> bool {
        statement_body_has_items(
            self.ast,
            Some(body.directives(self.ast)),
            body.body(self.ast),
        )
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
        self.format_braced_body(BodySource::Function(body, incoming, leading), mode, writer)
    }

    pub(super) fn format_static_block_body(
        &mut self,
        body: StaticBlock,
        span: Span,
        incoming: Option<IncomingStatement>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        self.format_braced_body(
            BodySource::Static {
                body,
                span,
                incoming,
            },
            ContentMode::Formatted,
            writer,
        )
    }

    fn format_braced_body(
        &mut self,
        source: BodySource<'ast>,
        mode: ContentMode,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
        let (span, directives, body, kind, empty_layout, leading) = match source {
            BodySource::Static {
                body,
                span,
                incoming,
            } => (
                span,
                None,
                body.body(ast),
                StatementSequenceKind::Static(incoming),
                EmptyBlock::Compact,
                StatementBodyLeading::empty(),
            ),
            BodySource::Block {
                node,
                incoming,
                leading,
                empty,
            } => (
                node.span(ast),
                None,
                node.body(ast),
                StatementSequenceKind::Block { incoming, leading },
                empty,
                leading,
            ),
            BodySource::Function(function, incoming, leading) => (
                function.span(ast),
                Some(function.directives(ast)),
                function.body(ast),
                StatementSequenceKind::Function { incoming, leading },
                EmptyBlock::Compact,
                leading,
            ),
        };
        if mode == ContentMode::Verbatim {
            return self.format_verbatim(span, writer);
        }
        let content = Span::new(span.start + 1, span.end - 1);
        let has_items = statement_body_has_items(ast, directives, body);
        let directives = directives
            .into_iter()
            .flat_map(|directives| directives.iter())
            .map(|id| StatementItem::Directive(ast.get_node_in_sub_range(id)));
        let body = body
            .iter()
            .map(|id| StatementItem::Statement(ast.get_node_in_sub_range(id)));
        let items = directives.chain(body).peekable();
        let leading = leading.comments();
        let source_empty = if has_items {
            false
        } else {
            let dangling = self.comments.in_range(content);
            if self.content_mode(dangling) == ContentMode::Verbatim {
                return self.format_verbatim(span, writer);
            }
            dangling.is_empty()
        };
        let empty = source_empty && leading.is_empty();
        let opening = self.opening_line_constraint(content);
        writer.text("{");
        if empty {
            opening.write(writer, FlatSeparator::Empty, |writer| {
                if matches!(empty_layout, EmptyBlock::Expanded) {
                    writer.hard_line();
                }
            });
        } else {
            let sequence = writer.indent(|writer| {
                opening.write(writer, FlatSeparator::Space, Writer::hard_line);
                self.format_statement_sequence(items, content, None, kind, writer)
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
