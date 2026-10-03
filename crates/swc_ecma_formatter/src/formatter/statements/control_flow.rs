//! Parent-owned conditions, controlled bodies, and their following keywords.

mod compound;
mod for_loops;
pub(super) use for_loops::IterationStatement;
mod jumps;
mod while_do;
pub(super) use jumps::JumpStatement;

use swc_next_ecma_ast::{
    Comment, CommentKind, Expr, ExprData, GetSpan, IfStatement, Span, Stmt, StmtData, UnaryOperator,
};

use super::{
    EmptyBlock, Formatter, PrintedStatement, StatementBoundary, StatementItem, StatementLayout,
    StatementPunctuation, StatementTailRelation,
};
use crate::formatter::comments::{
    CommentBinding, CommentGap, CommentRunLayout, ExpressionTail, NodeHead,
};
use crate::formatter::diagnostics::{FlatSeparator, LineConstraint};
use crate::formatter::expressions::{
    ExpressionContext, ExpressionPosition, ExpressionStart, TailOwner,
};
use crate::formatter::source::is_source_whitespace;
use crate::formatter::suppression::ContentMode;
use crate::{FormatError, Semicolons, document::Writer};

#[derive(Clone, Copy)]
enum ControlledBodyRole {
    Body,
    Alternate,
    Label,
}

#[derive(Clone, Copy)]
enum ControlledBodyShape {
    Empty,
    Block,
    ElseIf,
    Label,
    Single,
}

#[derive(Clone, Copy)]
enum ControlledBodyEnd {
    Alternate(Span),
    LoopCondition(Span),
    Enclosing(u32),
}

impl ControlledBodyEnd {
    fn limit(self) -> u32 {
        match self {
            Self::Alternate(keyword) | Self::LoopCondition(keyword) => keyword.start,
            Self::Enclosing(end) => end,
        }
    }

    fn has_following(self) -> bool {
        !matches!(self, Self::Enclosing(_))
    }
}

struct ControlledBody<'ast> {
    statement: Stmt,
    span: Span,
    head_end: u32,
    leading: &'ast [Comment],
    shape: ControlledBodyShape,
    role: ControlledBodyRole,
    empty_block: EmptyBlock,
    mode: ContentMode,
    punctuation: StatementPunctuation,
    end: ControlledBodyEnd,
    annotation_tail: Option<ControlledBodyTail<'ast>>,
}

/// The statement may consume a prefix while preserving an ignored source terminator.
struct ControlledBodyTail<'ast> {
    tail: ExpressionTail<'ast>,
    binding: CommentBinding,
}

impl ControlledBody<'_> {
    fn with_empty_block(mut self, empty: EmptyBlock) -> Self {
        self.empty_block = empty;
        self
    }

    fn with_enclosing_punctuation(mut self, punctuation: StatementPunctuation) -> Self {
        if matches!(
            punctuation,
            StatementPunctuation::Canonical(Semicolons::Always)
                | StatementPunctuation::Source(_)
                | StatementPunctuation::SourcePacket
        ) {
            self.punctuation = punctuation;
        }
        self
    }
}

#[derive(Clone, Copy)]
enum ConditionPrefix {
    Canonical,
    Source { source: Span, mode: ContentMode },
}

enum ConditionHeader<'ast> {
    Canonical {
        keyword_end: u32,
        opening: u32,
        before_opening: &'ast [Comment],
        relocate_before_opening: bool,
    },
    Source(Span),
}

struct Condition<'ast> {
    header: ConditionHeader<'ast>,
    expression: Expr,
    position: ExpressionPosition,
    closing: u32,
    tail_end: u32,
    leading: &'ast [Comment],
    mode: ContentMode,
    inline: bool,
}

/// The keyword retains its source separator after its body has consumed a tail prefix.
struct BodyKeyword {
    span: Span,
    source_end: u32,
    binding: CommentBinding,
    following: CommentGap,
    adjacent: bool,
    line: bool,
}

struct ElseBoundary {
    keyword: Span,
    preceding_mode: ContentMode,
    enclosing_mode: ContentMode,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_if_statement(
        &mut self,
        node: IfStatement,
        punctuation: StatementPunctuation,
        writer: &mut Writer<'ast>,
    ) -> Result<PrintedStatement<'ast>, FormatError> {
        let ast = self.ast;
        let span = node.span(ast);
        let consequent = node.consequent(ast);
        let alternate = node.alternate(ast);
        let boundary = alternate
            .map(|alternate| self.prepare_else_boundary(consequent, alternate))
            .transpose()?;
        if boundary
            .as_ref()
            .is_some_and(|boundary| boundary.enclosing_mode == ContentMode::Verbatim)
        {
            self.format_verbatim(span, writer)?;
            return Ok(StatementBoundary::SourceEnd {
                end: span.end,
                has_terminator: self.source(span).ends_with(';'),
            }
            .into());
        }
        let condition = self.prepare_condition(
            node.test(ast),
            span.start + "if".len() as u32,
            consequent.span(ast).start,
        )?;
        let body = self.prepare_controlled_body(
            consequent,
            condition.closing + 1,
            boundary
                .as_ref()
                .map_or(ControlledBodyEnd::Enclosing(span.end), |boundary| {
                    ControlledBodyEnd::Alternate(boundary.keyword)
                }),
            ControlledBodyRole::Body,
            boundary
                .as_ref()
                .map_or(ContentMode::Formatted, |boundary| boundary.preceding_mode),
        );
        let body = if boundary.is_none() {
            body.with_enclosing_punctuation(punctuation)
        } else {
            body
        };
        let keyword = boundary
            .as_ref()
            .map(|boundary| self.prepare_body_keyword(&body, boundary.keyword));
        let printed = writer.group(false, |writer| {
            writer.text("if");
            self.format_condition(condition, writer)?;
            self.format_controlled_body(body, writer)
        })?;
        let (Some(alternate), Some(boundary), Some(keyword)) = (alternate, boundary, keyword)
        else {
            return Ok(printed);
        };
        self.format_body_keyword(keyword, "else", printed, writer)?;
        let body = self
            .prepare_controlled_body(
                alternate,
                boundary.keyword.end,
                ControlledBodyEnd::Enclosing(span.end),
                ControlledBodyRole::Alternate,
                ContentMode::Formatted,
            )
            .with_enclosing_punctuation(punctuation);
        writer.group(false, |writer| self.format_controlled_body(body, writer))
    }

    fn prepare_condition(
        &self,
        expression: Expr,
        keyword_end: u32,
        body_start: u32,
    ) -> Result<Condition<'ast>, FormatError> {
        self.prepare_condition_with_prefix(
            expression,
            keyword_end,
            body_start,
            ConditionPrefix::Canonical,
        )
    }

    fn prepare_condition_with_prefix(
        &self,
        expression: Expr,
        keyword_end: u32,
        body_start: u32,
        prefix: ConditionPrefix,
    ) -> Result<Condition<'ast>, FormatError> {
        let span = expression.span(self.ast);
        let head = Span::new(keyword_end, span.start);
        let leading = self.comments.in_range(head);
        let Some(opening) = self.find_token(head, leading, "(") else {
            return Err(FormatError::unsupported(
                "condition opening boundary",
                None,
                head,
            ));
        };
        let tail = Span::new(span.end, body_start);
        let trailing = self.comments.in_range(tail);
        let Some(closing) = self.find_token(tail, trailing, ")") else {
            return Err(FormatError::unsupported(
                "condition closing boundary",
                None,
                tail,
            ));
        };
        let (header, leading, leading_mode, inline) = match prefix {
            ConditionPrefix::Canonical => {
                let split = leading.partition_point(|comment| comment.span.end <= opening);
                let before_opening = &leading[..split];
                let relocate_before_opening = self
                    .comment_binding(before_opening, head, Some(span))
                    .allows_relocation(Some(span));
                let leading = if relocate_before_opening {
                    leading
                } else {
                    &leading[split..]
                };
                let mode = self.content_mode(leading);
                let inline = before_opening.is_empty()
                    && leading.is_empty()
                    && trailing.is_empty()
                    && self.condition_is_inline(expression);
                let header = ConditionHeader::Canonical {
                    keyword_end,
                    opening,
                    before_opening,
                    relocate_before_opening,
                };
                (header, leading, mode, inline)
            }
            ConditionPrefix::Source { source, mode } => {
                (ConditionHeader::Source(source), &[][..], mode, false)
            }
        };
        let mode = if leading_mode == ContentMode::Verbatim {
            leading_mode
        } else {
            self.trailing_content_mode(Span::new(span.end, closing), true)
        };
        Ok(Condition {
            header,
            expression,
            position: ExpressionPosition::ControlCondition,
            closing,
            tail_end: closing,
            leading,
            mode,
            inline,
        })
    }

    fn condition_is_inline(&self, expression: Expr) -> bool {
        let content = self.expression_content(expression);
        if content.expression != expression {
            // Root trivia still belongs to the condition after its source pair is removed.
            let span = expression.span(self.ast);
            let inner = content.expression.span(self.ast);
            let comments = self.ast.comments();
            let first = comments.partition_point(|comment| comment.span.start < span.start);
            let comments = &comments[first..];
            let count = comments.partition_point(|comment| comment.span.end <= span.end);
            let comments = &comments[..count];
            if comments
                .first()
                .is_some_and(|comment| comment.span.end <= inner.start)
                || comments
                    .last()
                    .is_some_and(|comment| comment.span.start >= inner.end)
            {
                return false;
            }
        }
        let mut expression = content.expression;
        let mut negations = 0;
        while negations < 2 {
            let ExprData::UnaryExpression(unary) = self.ast.expr_data(expression) else {
                break;
            };
            if unary.operator(self.ast) != UnaryOperator::LogicalNot {
                break;
            }
            expression = self.expression_content(unary.argument(self.ast)).expression;
            negations += 1;
        }
        negations != 0
            && matches!(
                self.ast.expr_data(expression),
                ExprData::LogicalExpression(_)
            )
    }

    fn format_condition(
        &mut self,
        condition: Condition<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let Condition {
            header,
            expression,
            position,
            closing,
            tail_end,
            leading,
            mode,
            inline,
        } = condition;
        let span = expression.span(self.ast);
        let opening_constraint = match header {
            ConditionHeader::Source(source) => {
                self.format_delimited_source_gap(
                    source,
                    crate::formatter::suppression::DelimitedFollowing::Node,
                    writer,
                )?
                .following
            }
            ConditionHeader::Canonical {
                keyword_end,
                opening,
                before_opening,
                relocate_before_opening,
            } => {
                if relocate_before_opening {
                    self.line_constraint(keyword_end, opening).write(
                        writer,
                        FlatSeparator::Space,
                        Writer::space,
                    );
                } else {
                    let first = before_opening
                        .first()
                        .map_or(opening, |comment| comment.span.start);
                    self.line_constraint(keyword_end, first).write(
                        writer,
                        FlatSeparator::Space,
                        Writer::space,
                    );
                    self.comments.claim(before_opening)?;
                    self.format_comment_run(
                        before_opening,
                        CommentRunLayout::FlatLeading(opening),
                        writer,
                    )?;
                }
                writer.text("(");
                let first = leading
                    .first()
                    .map_or(span.start, |comment| comment.span.start.max(opening + 1));
                self.line_constraint(opening + 1, first)
            }
        };
        writer.group(false, |writer| {
            let mut write_contents = |writer: &mut Writer<'ast>| {
                if !inline {
                    opening_constraint.write(writer, FlatSeparator::Empty, Writer::soft_line);
                }
                self.comments.claim(leading)?;
                let tail = self.format_expression_with_leading_comments(
                    expression,
                    NodeHead {
                        comments: leading,
                        content_start: span.start,
                        layout: CommentRunLayout::GroupedLeading(span.start),
                        gap_end: None,
                    },
                    ExpressionContext::at(position)
                        .with_content_mode(mode)
                        .with_tail_owner(TailOwner::ParentOperation),
                    None,
                    writer,
                )?;
                let tail = self.capture_expression_tail(Span::new(span.end, tail_end), tail)?;
                let last = tail.as_ref().and_then(|tail| tail.comments.last());
                let end = last.map_or(span.end, |comment| comment.span.end);
                self.format_expression_tail(tail, writer)?;
                Ok::<_, FormatError>((
                    end,
                    last.is_some_and(|comment| comment.kind == CommentKind::Line),
                ))
            };
            let (end, line_comment) = if inline {
                write_contents(writer)?
            } else {
                writer.indent(write_contents)?
            };
            if !inline {
                self.line_constraint(end, closing)
                    .write(writer, FlatSeparator::Empty, |writer| {
                        if line_comment {
                            writer.hard_line();
                        } else {
                            writer.soft_line();
                        }
                    });
            }
            writer.text(")");
            Ok(())
        })
    }

    fn prepare_controlled_body(
        &self,
        statement: Stmt,
        head_end: u32,
        end: ControlledBodyEnd,
        role: ControlledBodyRole,
        trailing_mode: ContentMode,
    ) -> ControlledBody<'ast> {
        let span = statement.span(self.ast);
        let leading = self.comments.in_range(Span::new(head_end, span.start));
        let data = self.ast.stmt_data(statement);
        let shape = match data {
            StmtData::EmptyStatement(_) => ControlledBodyShape::Empty,
            StmtData::BlockStatement(_) => ControlledBodyShape::Block,
            StmtData::IfStatement(_) if matches!(role, ControlledBodyRole::Alternate) => {
                ControlledBodyShape::ElseIf
            }
            _ if matches!(role, ControlledBodyRole::Label) => ControlledBodyShape::Label,
            _ => ControlledBodyShape::Single,
        };
        let needs_source_tail = self.has_suppression
            || self.has_target_annotations
            || !self.diagnostic_lines.is_empty();
        let needs_comment_tail = (!matches!(shape, ControlledBodyShape::Block)
            || end.has_following())
            && !self.ast.comments().is_empty();
        let content_end = if needs_source_tail || needs_comment_tail {
            self.statement_content_end(StatementItem::Statement(statement))
        } else {
            span.end
        };
        let trailing_limit = end.limit();
        let leading_mode = self.content_mode(leading);
        let mode = if leading_mode == ContentMode::Verbatim {
            leading_mode
        } else {
            trailing_mode
        };
        let semicolons = if end.has_following()
            && self.options.semicolons == Semicolons::AsNeeded
            && self.line_constraint(content_end, trailing_limit) == LineConstraint::SameLine
        {
            // An expression body before `else` still needs a terminator on a covered source line.
            Semicolons::Always
        } else {
            self.options.semicolons
        };
        let punctuation = StatementPunctuation::Canonical(semicolons);
        let annotation_tail = if needs_comment_tail || self.has_suppression {
            self.prepare_controlled_body_tail(span, content_end, end, shape)
        } else {
            None
        };
        ControlledBody {
            statement,
            span,
            head_end,
            leading,
            shape,
            role,
            empty_block: EmptyBlock::Expanded,
            mode,
            punctuation,
            end,
            annotation_tail,
        }
    }

    /// A trailing source run keeps the same owner across a moved semicolon.
    /// Ordinary runs can follow the compound group; bound runs keep their
    /// original source boundary and the keyword consumes its own pending tail.
    fn prepare_controlled_body_tail(
        &self,
        statement: Span,
        content_end: u32,
        end: ControlledBodyEnd,
        shape: ControlledBodyShape,
    ) -> Option<ControlledBodyTail<'ast>> {
        let limit = match end {
            ControlledBodyEnd::Alternate(keyword) | ControlledBodyEnd::LoopCondition(keyword) => {
                keyword.start
            }
            ControlledBodyEnd::Enclosing(_) => self.ast.source().len() as u32,
        };
        let comments = self.comments.in_range(Span::new(content_end, limit));
        let mut previous = content_end;
        let mut count = 0;
        for &comment in comments {
            let gap = self.source(Span::new(previous, comment.span.start));
            if !gap.chars().all(|character| {
                character == ';'
                    || is_source_whitespace(character)
                        && !matches!(character, '\n' | '\r' | '\u{2028}' | '\u{2029}')
            }) {
                break;
            }
            previous = comment.span.end;
            count += 1;
            if comment.kind == CommentKind::Line {
                break;
            }
        }
        let comments = &comments[..count];
        let last = comments.last()?;
        let binding = self.comment_binding(comments, Span::new(content_end, last.span.end), None);
        let comments = if binding.allows_relocation(None) {
            self.ordinary_controlled_body_tail(comments, statement, end)
        } else {
            let count = comments.partition_point(|comment| comment.span.start < end.limit());
            &comments[..count]
        };
        if comments.is_empty()
            || matches!(shape, ControlledBodyShape::Block)
                && !end.has_following()
                && self.content_mode(comments) != ContentMode::Verbatim
        {
            return None;
        }
        Some(ControlledBodyTail {
            tail: ExpressionTail {
                source_end: content_end,
                comments,
            },
            binding,
        })
    }

    fn ordinary_controlled_body_tail(
        &self,
        comments: &'ast [Comment],
        statement: Span,
        end: ControlledBodyEnd,
    ) -> &'ast [Comment] {
        if end.has_following() {
            return comments;
        }
        let Some(terminator) = self.statement_source_terminator(statement) else {
            return comments;
        };
        let split = comments.partition_point(|comment| comment.span.end <= terminator.start);
        let after = &comments[split..];
        let Some(last) = after.last() else {
            return comments;
        };
        if last.kind != CommentKind::Block
            || !after.iter().any(|comment| {
                comment.kind == CommentKind::Block
                    && self.line_breaks(comment.span.start, comment.span.end) != 0
            })
        {
            return comments;
        }
        // An exterior multiline block leads the next statement when its
        // closing line continues into that statement. An internal packet
        // stays with this body across the canonical semicolon.
        let following = self.ast.source()[last.span.end as usize..]
            .char_indices()
            .find(|(_, character)| !is_source_whitespace(*character))
            .map_or(self.ast.source().len() as u32, |(offset, _)| {
                last.span.end + offset as u32
            });
        if self.line_breaks(last.span.end, following) != 0 {
            return comments;
        }
        &comments[..split]
    }

    fn format_controlled_body(
        &mut self,
        body: ControlledBody<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<PrintedStatement<'ast>, FormatError> {
        let ControlledBody {
            statement,
            span,
            head_end,
            leading,
            shape,
            role,
            empty_block,
            mode,
            punctuation,
            end,
            annotation_tail,
        } = body;
        let trailing_limit = end.limit();
        let first = leading
            .first()
            .map_or(span.start, |comment| comment.span.start);
        let constraint = self.line_constraint(head_end, first);
        let leading_line = leading.first().is_some_and(|comment| {
            self.line_breaks(head_end, comment.span.start) != 0
                || !matches!(role, ControlledBodyRole::Label)
                    && self.line_breaks(comment.span.start, comment.span.end) != 0
        });
        let adjacent = matches!(
            shape,
            ControlledBodyShape::Block | ControlledBodyShape::ElseIf | ControlledBodyShape::Label
        );
        let empty = matches!(shape, ControlledBodyShape::Empty);
        let compact_head = matches!(
            (shape, empty_block),
            (ControlledBodyShape::Block, EmptyBlock::Compact)
        ) && mode == ContentMode::Formatted
            && leading.first().is_some_and(|comment| {
                comment.kind == CommentKind::Block
                    && self.line_breaks(head_end, comment.span.start) == 0
                    && self.line_breaks(comment.span.start, comment.span.end) == 0
            })
            && leading
                .iter()
                .all(|comment| self.line_breaks(comment.span.start, comment.span.end) == 0)
            && leading
                .windows(2)
                .all(|pair| self.line_breaks(pair[0].span.end, pair[1].span.start) == 0)
            && self
                .comment_binding(leading, Span::new(head_end, span.start), None)
                .allows_relocation(None)
            && matches!(self.ast.stmt_data(statement), StmtData::BlockStatement(block)
                if !super::statement_body_has_items(self.ast, None, block.body(self.ast)));
        let inline_head = empty
            && mode == ContentMode::Formatted
            && !matches!(constraint, LineConstraint::NewLine(_));
        let write_body = |writer: &mut Writer<'ast>| {
            let flat = if empty && leading.is_empty() {
                FlatSeparator::Empty
            } else {
                FlatSeparator::Space
            };
            constraint.write(writer, flat, |writer| {
                if empty && mode == ContentMode::Formatted {
                    if !leading.is_empty() {
                        writer.space();
                    }
                } else if leading_line {
                    writer.hard_line();
                } else if empty && leading.is_empty() {
                } else if adjacent {
                    writer.space();
                } else {
                    writer.line();
                }
            });
            self.comments.claim(leading)?;
            let mut printed = self.format_statement(
                statement,
                StatementLayout {
                    start: ExpressionStart::StatementBody,
                    empty_block,
                },
                mode,
                NodeHead {
                    comments: leading,
                    content_start: span.start,
                    layout: if compact_head {
                        CommentRunLayout::InlineLeading(span.start)
                    } else if matches!(shape, ControlledBodyShape::Block) {
                        CommentRunLayout::Source
                    } else if inline_head {
                        CommentRunLayout::InlineLeading(span.start)
                    } else {
                        CommentRunLayout::GroupedLeading(span.start)
                    },
                    gap_end: if matches!(shape, ControlledBodyShape::Block) && !compact_head {
                        leading.last().map(|_| span.start)
                    } else {
                        None
                    },
                },
                punctuation,
                writer,
            )?;
            if let Some(tail) = annotation_tail
                && let Some(last) = tail.tail.comments.last()
                && last.span.end > printed.boundary.end()
            {
                printed.pending = self.capture_expression_tail(
                    Span::new(
                        tail.tail.source_end.max(printed.boundary.end()),
                        last.span.end,
                    ),
                    printed.pending,
                )?;
                if !tail.binding.allows_relocation(None) && !end.has_following() {
                    printed = self.flush_statement_tail(printed, writer)?.into();
                }
            }
            if end.has_following() {
                let line_tail = printed.pending.as_ref().is_some_and(|tail| {
                    tail.comments
                        .iter()
                        .any(|comment| comment.kind == CommentKind::Line)
                });
                if line_tail {
                    // The keyword owns the packet after this body's indent,
                    // while its pending line suffix still breaks the body group.
                    writer.break_parent();
                } else {
                    printed = self.flush_statement_tail(printed, writer)?.into();
                }
            }
            if printed.pending.is_some() {
                return Ok(printed);
            }
            let end = self.format_trailing_comments(
                Span::new(printed.boundary.end(), trailing_limit),
                span.end,
                end.has_following(),
                None,
                writer,
            )?;
            printed.boundary = printed.boundary.with_end(end);
            Ok(printed)
        };
        if matches!(shape, ControlledBodyShape::Block) || adjacent && !leading_line || empty {
            write_body(writer)
        } else {
            writer.indent(write_body)
        }
    }

    fn prepare_else_boundary(
        &self,
        consequent: Stmt,
        alternate: Stmt,
    ) -> Result<ElseBoundary, FormatError> {
        let gap = Span::new(
            consequent.span(self.ast).end,
            alternate.span(self.ast).start,
        );
        let comments = self.comments.in_range(gap);
        let Some(start) = self.find_token(gap, comments, "else") else {
            return Err(FormatError::unsupported("else boundary", None, gap));
        };
        let (preceding_mode, enclosing_mode) = match self.prepare_statement_suppression_tail(
            StatementItem::Statement(consequent),
            start,
            StatementTailRelation::Keyword,
        ) {
            Some(tail) => (tail.modes.current, tail.modes.enclosing),
            None => (ContentMode::Formatted, ContentMode::Formatted),
        };
        Ok(ElseBoundary {
            keyword: Span::new(start, start + "else".len() as u32),
            preceding_mode,
            enclosing_mode,
        })
    }

    fn prepare_body_keyword(&self, body: &ControlledBody<'ast>, span: Span) -> BodyKeyword {
        let tail = body.annotation_tail.as_ref();
        let last = tail.and_then(|tail| tail.tail.comments.last());
        let source_end = last.map_or(body.span.end, |comment| body.span.end.max(comment.span.end));
        let comments = self.comments.in_range(Span::new(source_end, span.start));
        let tail_line = match (tail, last) {
            (Some(tail), Some(comment)) => {
                comment.kind == CommentKind::Line
                    && (tail.binding.allows_relocation(None) || comment.span.end > body.span.end)
            }
            _ => false,
        };
        let condition_tail = matches!(body.end, ControlledBodyEnd::LoopCondition(_))
            && tail.is_some_and(|tail| tail.binding.allows_relocation(None));
        let following = if comments.is_empty() && tail_line && !condition_tail {
            CommentGap::LineComment
        } else {
            self.prepare_comment_boundary(comments, source_end, span.start)
                .gap()
        };
        let adjacent = matches!(body.shape, ControlledBodyShape::Block);
        let line = if let Some(last) = comments.last() {
            self.line_breaks(last.span.end, span.start) != 0
        } else {
            !adjacent
                || !condition_tail
                    && last.is_some_and(|last| self.line_breaks(last.span.end, span.start) != 0)
        };
        BodyKeyword {
            span,
            source_end,
            binding: self.comment_binding(comments, Span::new(source_end, span.start), None),
            following,
            adjacent,
            line,
        }
    }

    fn format_body_keyword(
        &mut self,
        keyword: BodyKeyword,
        text: &'static str,
        printed: PrintedStatement<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let printed = self.flush_statement_tail(printed, writer)?;
        let span = Span::new(printed.end(), keyword.span.start);
        let comments = self.comments.between(span)?;
        if let Some(first) = comments.first() {
            let first = first.span.start;
            self.line_constraint(keyword.source_end, first).write(
                writer,
                FlatSeparator::Space,
                |writer| {
                    if !keyword.binding.allows_relocation(None) {
                        self.comment_gap(keyword.source_end, first, writer);
                    } else if keyword.adjacent {
                        if self.line_breaks(keyword.source_end, first) != 0 {
                            writer.hard_line();
                            if self.line_breaks(keyword.source_end, first) == 2 {
                                writer.hard_line();
                            }
                        } else {
                            writer.space();
                        }
                    } else {
                        writer.hard_line();
                        if self.line_breaks(keyword.source_end, first) == 2 {
                            writer.hard_line();
                        }
                    }
                },
            );
            self.comments.claim(comments)?;
            self.format_comment_run(comments, CommentRunLayout::Source, writer)?;
        }
        keyword
            .following
            .write(writer, FlatSeparator::Space, |writer| {
                if keyword.line {
                    writer.hard_line();
                } else {
                    writer.space();
                }
            });
        writer.text(text);
        Ok(())
    }
}
