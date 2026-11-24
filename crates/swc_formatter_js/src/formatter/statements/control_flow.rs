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
    ContinuationBoundary, EmptyBlock, FollowingSyntax, Formatter, PrintedStatement,
    StatementBoundary, StatementContent, StatementItem, StatementLayout, StatementPunctuation,
    StatementTailRelation, VerbatimContent,
};
use crate::formatter::comments::{
    CommentBinding, CommentGap, CommentRunLayout, ExpressionTail, NodeHead,
    StatementCommentBoundary,
};
use crate::formatter::diagnostics::{FlatSeparator, LineConstraint};
use crate::formatter::expressions::{
    ExpressionContext, ExpressionPosition, ExpressionStart, StringStatement, TailOwner,
};
use crate::formatter::source::is_source_whitespace;
use crate::formatter::suppression::ContentMode;
use crate::{
    FormatError, Semicolons,
    document::{ConditionTarget, Writer},
};

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

#[derive(Clone, Copy, PartialEq, Eq)]
enum ControlledBodyContent {
    Formatted,
    Verbatim,
    SourceNode,
}

impl From<ContentMode> for ControlledBodyContent {
    fn from(mode: ContentMode) -> Self {
        match mode {
            ContentMode::Formatted => Self::Formatted,
            ContentMode::Verbatim => Self::Verbatim,
        }
    }
}

#[derive(Clone, Copy)]
enum ControlledBodyEnd {
    Alternate(Span),
    LoopCondition(Span),
    Enclosing(u32),
}

impl ControlledBodyEnd {
    fn keyword(self) -> Option<Span> {
        match self {
            Self::Alternate(keyword) | Self::LoopCondition(keyword) => Some(keyword),
            Self::Enclosing(_) => None,
        }
    }

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
    mode: ControlledBodyContent,
    punctuation: StatementPunctuation,
    source_candidate: ContinuationBoundary,
    end: ControlledBodyEnd,
    keyword: Option<BodyKeyword>,
    annotation_tail: Option<ControlledBodyTail<'ast>>,
    operator_tail: Option<&'ast super::DeferredOperatorTail<'ast>>,
}

/// The statement may consume a prefix while preserving an ignored source terminator.
struct ControlledBodyTail<'ast> {
    tail: ExpressionTail<'ast>,
    binding: CommentBinding,
}

impl<'ast> ControlledBody<'ast> {
    fn with_operator_tail(mut self, tail: Option<&'ast super::DeferredOperatorTail<'ast>>) -> Self {
        self.operator_tail = tail;
        self
    }

    fn with_empty_block(mut self, empty: EmptyBlock) -> Self {
        self.empty_block = empty;
        self
    }

    fn with_enclosing_boundary(
        mut self,
        punctuation: StatementPunctuation,
        source: ContinuationBoundary,
    ) -> Self {
        self.source_candidate = source.for_child(self.span);
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
    },
    Source(Span),
}

#[derive(Clone, Copy)]
struct ConditionLineSuffix<'ast> {
    comments: &'ast [Comment],
}

impl<'ast> ConditionLineSuffix<'ast> {
    fn new(comments: &'ast [Comment]) -> Option<Self> {
        let (last, prefix) = comments.split_last()?;
        (last.kind == CommentKind::Line
            && prefix
                .iter()
                .all(|comment| comment.kind == CommentKind::Block))
        .then_some(Self { comments })
    }
}

struct Condition<'ast> {
    header: ConditionHeader<'ast>,
    expression: Expr,
    position: ExpressionPosition,
    closing: u32,
    leading: &'ast [Comment],
    mode: ContentMode,
    inline: bool,
}

/// The keyword retains its source separator after its body has consumed a tail prefix.
#[derive(Clone, Copy)]
struct BodyKeyword {
    span: Span,
    following: CommentGap,
    adjacent: bool,
    line: bool,
    detached: Option<u32>,
    layout: CommentRunLayout,
}

struct ElseBoundary {
    keyword: Span,
    preceding_mode: ControlledBodyContent,
    enclosing_mode: ContentMode,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_if_statement(
        &mut self,
        node: IfStatement,
        punctuation: StatementPunctuation,
        source_candidate: ContinuationBoundary,
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
            return self.format_verbatim_if_statement(node, punctuation, source_candidate, writer);
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
                .map_or(ControlledBodyContent::Formatted, |boundary| {
                    boundary.preceding_mode
                }),
        );
        let body = if boundary.is_none() {
            body.with_enclosing_boundary(punctuation, source_candidate)
        } else {
            body
        };
        let keyword = body.keyword;
        let printed = writer.group(false, |writer| {
            writer.text("if");
            self.format_condition(condition, writer)?;
            self.format_controlled_body(body, writer)
        })?;
        let (Some(alternate), Some(keyword)) = (alternate, keyword) else {
            return Ok(printed);
        };
        self.format_body_keyword(keyword, "else", printed, writer)?;
        let body = self
            .prepare_controlled_body(
                alternate,
                keyword.span.end,
                ControlledBodyEnd::Enclosing(span.end),
                ControlledBodyRole::Alternate,
                ControlledBodyContent::Formatted,
            )
            .with_enclosing_boundary(punctuation, source_candidate);
        writer.group(false, |writer| self.format_controlled_body(body, writer))
    }

    fn format_verbatim_if_statement(
        &mut self,
        node: IfStatement,
        punctuation: StatementPunctuation,
        following: ContinuationBoundary,
        writer: &mut Writer<'ast>,
    ) -> Result<PrintedStatement<'ast>, FormatError> {
        let span = node.span(self.ast);
        let statement = StatementItem::Statement(Stmt::IfStatement(node));
        let has_terminator = self.statement_source_terminator(span).is_some();
        let unterminated_body = !has_terminator
            && matches!(
                self.statement_content(statement),
                StatementContent::Semicolon(_)
            );
        let punctuation = match (unterminated_body, punctuation) {
            (true, StatementPunctuation::Canonical(Semicolons::AsNeeded))
                if following.is_following() =>
            {
                let following_needs_guard = match following {
                    ContinuationBoundary::Following {
                        kind:
                            FollowingSyntax::ExpressionStatement {
                                expression,
                                string_statement,
                                mode,
                            },
                        ..
                    } => self.expression_needs_asi_guard(
                        expression,
                        ExpressionContext::at(ExpressionPosition::Statement(string_statement))
                            .with_content_mode(mode),
                    )?,
                    _ => false,
                };
                if following_needs_guard
                    || following.needs_generated(statement.raw_type_terminal(self.ast))
                {
                    // A leading guard would become part of the copied branch on the next pass.
                    StatementPunctuation::Canonical(Semicolons::Always)
                } else {
                    punctuation
                }
            }
            _ => punctuation,
        };
        self.format_verbatim(span, writer)?;
        if unterminated_body {
            self.finish_statement(span.end, None, punctuation, writer)
        } else {
            Ok(StatementBoundary::SourceEnd {
                end: span.end,
                has_terminator,
            }
            .into())
        }
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
                let (before_opening, leading) = leading.split_at(split);
                let mut mode = self.content_mode(leading);
                if self.content_mode(before_opening) == ContentMode::Verbatim
                    && self
                        .comment_binding(before_opening, head, Some(span))
                        .allows_relocation(Some(span))
                {
                    // The ignore's content owner is independent of the opener's comments.
                    mode = ContentMode::Verbatim;
                }
                let inline = before_opening.is_empty()
                    && leading.is_empty()
                    && trailing.is_empty()
                    && self.condition_is_inline(expression);
                let header = ConditionHeader::Canonical {
                    keyword_end,
                    opening,
                    before_opening,
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
        self.format_condition_with_suffix(condition, None, writer)
    }

    fn format_condition_line_suffix(
        &mut self,
        suffix: ConditionLineSuffix<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        self.comments.claim(suffix.comments)?;
        let Some((line, prefix)) = suffix.comments.split_last() else {
            return Ok(());
        };
        if !prefix.is_empty() {
            self.format_comment_run(prefix, CommentRunLayout::Source, writer)?;
        }
        writer.line_suffix(|writer| {
            writer.space();
            writer.text(self.source(line.span).trim_end());
            Ok(())
        })
    }

    fn format_control_header_gap(
        &mut self,
        comments: &'ast [Comment],
        start: u32,
        next: u32,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        if let Some(&last) = comments.last() {
            self.comments.claim(comments)?;
            self.format_comment_run(comments, CommentRunLayout::SourceAfter(start), writer)?;
            self.comment_gap_after(last, next, writer);
        } else {
            self.line_constraint(start, next)
                .write(writer, FlatSeparator::Space, Writer::space);
        }
        Ok(())
    }

    fn format_condition_with_suffix(
        &mut self,
        condition: Condition<'ast>,
        suffix: Option<ConditionLineSuffix<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let Condition {
            header,
            expression,
            position,
            closing,
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
            } => {
                self.format_control_header_gap(before_opening, keyword_end, opening, writer)?;
                writer.text("(");
                let first = leading
                    .first()
                    .map_or(span.start, |comment| comment.span.start.max(opening + 1));
                self.line_constraint(opening + 1, first)
            }
        };
        if let Some(suffix) = suffix {
            self.format_condition_line_suffix(suffix, writer)?;
        }
        writer.group(suffix.is_some(), |writer| {
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
                let tail = self.capture_expression_tail(Span::new(span.end, closing), tail)?;
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
        trailing_mode: ControlledBodyContent,
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
        let leading_mode = self.content_mode(leading);
        let mode = match (trailing_mode, leading_mode) {
            (ControlledBodyContent::SourceNode, _) => ControlledBodyContent::SourceNode,
            (_, ContentMode::Verbatim) => ControlledBodyContent::Verbatim,
            _ => trailing_mode,
        };
        let annotation_tail = if needs_comment_tail || self.has_suppression {
            self.prepare_controlled_body_tail(span, content_end, end, shape)
        } else {
            None
        };
        let mut body = ControlledBody {
            statement,
            span,
            head_end,
            leading,
            shape,
            role,
            empty_block: EmptyBlock::Expanded,
            mode,
            punctuation: StatementPunctuation::Canonical(self.options.semicolons),
            source_candidate: ContinuationBoundary::End,
            end,
            keyword: None,
            annotation_tail,
            operator_tail: None,
        };
        if let Some(span) = end.keyword() {
            let detached = self.detach_alternate_comment_run(&mut body);
            let keyword = self.prepare_body_keyword(&body, span, detached);
            if self.options.semicolons == Semicolons::AsNeeded
                && keyword.detached.is_none()
                && matches!(
                    keyword.following,
                    CommentGap::Source(LineConstraint::SameLine)
                )
                && self.line_breaks(content_end, span.start) == 0
            {
                // The constrained keyword gap cannot supply a line break for ASI.
                body.punctuation = StatementPunctuation::Canonical(Semicolons::Always);
            }
            body.keyword = Some(keyword);
            body = self.with_controlled_body_keyword_fence(body, span);
        }
        body
    }

    fn detach_alternate_comment_run(&self, body: &mut ControlledBody<'ast>) -> Option<u32> {
        if !matches!(body.end, ControlledBodyEnd::Alternate(_))
            || !matches!(body.shape, ControlledBodyShape::Single)
            || body.mode != ControlledBodyContent::Formatted
        {
            return None;
        }
        let tail = body.annotation_tail.as_mut()?;
        if !tail.binding.allows_relocation(None) {
            return None;
        }
        let index = tail.tail.comments.iter().position(|comment| {
            comment.kind == CommentKind::Block
                && self.line_breaks(comment.span.start, comment.span.end) != 0
        })?;
        let start = tail.tail.comments[index].span.start;
        if self.lint_line_at(start).is_some() {
            return None;
        }
        // The multiline suffix belongs to the keyword join outside the body group.
        tail.tail.comments = &tail.tail.comments[..index];
        if index == 0 {
            body.annotation_tail = None;
        }
        Some(start)
    }

    fn with_controlled_body_keyword_fence(
        &self,
        mut body: ControlledBody<'ast>,
        keyword: Span,
    ) -> ControlledBody<'ast> {
        if body.span.end <= keyword.start {
            body.source_candidate = ContinuationBoundary::Following {
                kind: FollowingSyntax::Keyword,
                source: self.statement_source_terminator(body.span),
            };
        }
        body
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
            source_candidate,
            end,
            ref annotation_tail,
            operator_tail,
            ..
        } = body;
        let trailing_limit = body
            .keyword
            .and_then(|keyword| keyword.detached)
            .unwrap_or(end.limit());
        let first = leading
            .first()
            .map_or(span.start, |comment| comment.span.start);
        let constraint = self.line_constraint(head_end, first);
        let constraint = if constraint == LineConstraint::Unconstrained
            && matches!(role, ControlledBodyRole::Alternate)
            && self.has_lint_line_directives()
        {
            self.lint_opening_constraint(head_end, first)
                .unwrap_or(constraint)
        } else {
            constraint
        };
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
        let formatted = mode == ControlledBodyContent::Formatted;
        let compact_head = operator_tail.is_none()
            && matches!(
                (shape, empty_block),
                (ControlledBodyShape::Block, EmptyBlock::Compact)
            )
            && formatted
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
        let inline_head = empty && formatted && !matches!(constraint, LineConstraint::NewLine(_));
        let write_gap = |constraint: LineConstraint,
                         leading_line: bool,
                         has_operator_head: bool,
                         writer: &mut Writer<'ast>| {
            let flat = if empty && leading.is_empty() && !has_operator_head {
                FlatSeparator::Empty
            } else {
                FlatSeparator::Space
            };
            constraint.write(writer, flat, |writer| {
                if empty && formatted {
                    if !leading.is_empty() || has_operator_head {
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
        };
        let mut write_body = |writer: &mut Writer<'ast>| {
            if let Some(tail) = operator_tail {
                debug_assert!(tail.destination() == super::OperatorTailDestination::ControlledBody);
                let (headed, ()) = writer.record_with_value(|writer| {
                    write_gap(LineConstraint::Unconstrained, false, true, writer);
                    self.format_deferred_operator_head(tail, first, writer);
                    Ok::<_, FormatError>(())
                })?;
                if let Some(group) = tail.flat_group() {
                    let (ordinary, ()) = writer.record_with_value(|writer| {
                        write_gap(constraint, leading_line, false, writer);
                        Ok::<_, FormatError>(())
                    })?;
                    writer.if_break_doc(ConditionTarget::Group(group), ordinary, headed);
                } else {
                    writer.write_fragment(headed);
                }
            } else {
                write_gap(constraint, leading_line, false, writer);
            }
            self.comments.claim(leading)?;
            let layout = StatementLayout {
                operator_tail_destination: super::OperatorTailDestination::Local,
                start: ExpressionStart::StatementBody,
                string_statement: StringStatement::Bare,
                empty_block,
                source_candidate: if matches!(
                    punctuation,
                    StatementPunctuation::Canonical(Semicolons::AsNeeded)
                ) {
                    source_candidate
                } else {
                    ContinuationBoundary::End
                },
            };
            let head = NodeHead {
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
            };
            let mut printed = if mode == ControlledBodyContent::SourceNode {
                self.format_node_head(head, layout.start, None, writer)?;
                self.format_verbatim_statement(
                    span,
                    VerbatimContent::SourceNode,
                    punctuation,
                    writer,
                )?
            } else {
                let mode = if formatted {
                    ContentMode::Formatted
                } else {
                    ContentMode::Verbatim
                };
                self.format_statement(statement, layout, mode, head, punctuation, writer)?
            };
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
                    writer.break_parent();
                } else {
                    printed = self.flush_statement_tail(printed, writer)?.into();
                }
            }
            if printed.pending.is_some() {
                return Ok(printed);
            }
            let (end, _) = self.format_trailing_comments(
                Span::new(
                    printed.boundary.end(),
                    trailing_limit.max(printed.boundary.end()),
                ),
                StatementCommentBoundary {
                    source_end: span.end,
                    terminator: printed.boundary.comment_terminator_placement(),
                },
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
        let mut boundary = ElseBoundary {
            keyword: Span::new(start, start + "else".len() as u32),
            preceding_mode: ControlledBodyContent::Formatted,
            enclosing_mode: ContentMode::Formatted,
        };
        let Some(tail) = self.prepare_statement_suppression_tail(
            StatementItem::Statement(consequent),
            start,
            StatementTailRelation::Keyword,
        ) else {
            return Ok(boundary);
        };
        boundary.enclosing_mode = tail.modes.enclosing;
        if tail.modes.current == ContentMode::Verbatim {
            boundary.preceding_mode = ControlledBodyContent::SourceNode;
            let comments = self.comments.in_range(Span::new(gap.start, start));
            let block = matches!(self.ast.stmt_data(consequent), StmtData::BlockStatement(_));
            if self
                .suppressed_comment_runs(comments, start)
                .any(|(run, _)| {
                    block
                        || run.last().is_some_and(|last| {
                            self.source_line_breaks(run[0].span.start, last.span.end) != 0
                        })
                })
            {
                boundary.enclosing_mode = ContentMode::Verbatim;
            }
        }
        Ok(boundary)
    }

    fn prepare_body_keyword(
        &self,
        body: &ControlledBody<'ast>,
        span: Span,
        detached: Option<u32>,
    ) -> BodyKeyword {
        let tail = body.annotation_tail.as_ref();
        let last = tail.and_then(|tail| tail.tail.comments.last());
        let source_end = last.map_or(body.span.end, |comment| comment.span.end);
        let gap = Span::new(
            detached.unwrap_or(body.span.end.max(source_end)),
            span.start,
        );
        let comments = self.comments.in_range(gap);
        let lint_lines = self.has_lint_line_directives()
            && (self.has_line_directive_in(gap)
                || self.lint_line_intersects(Span::new(source_end, span.end)));
        let tail_line = match (tail, last) {
            (Some(tail), Some(comment)) => {
                comment.kind == CommentKind::Line
                    && (tail.binding.allows_relocation(None) || comment.span.end > body.span.end)
            }
            _ => false,
        };
        let following = if lint_lines {
            let end = comments
                .last()
                .map_or(source_end, |comment| comment.span.end);
            CommentGap::Source(self.source_line_constraint(end, span.start))
        } else if comments.is_empty() && tail_line {
            CommentGap::LineComment
        } else {
            // Canonical punctuation is emitted before this tail, so the last
            // comment supplies the keyword's line constraint.
            self.prepare_comment_boundary(comments, source_end, span.start)
                .gap()
        };
        let adjacent = matches!(body.shape, ControlledBodyShape::Block);
        let line = if let Some(last) = comments.last() {
            self.line_breaks(last.span.end, span.start) != 0
                || detached.is_some() && last.span.end < body.span.end
        } else {
            !adjacent || last.is_some_and(|last| self.line_breaks(last.span.end, span.start) != 0)
        };
        BodyKeyword {
            span,
            following,
            adjacent,
            line,
            detached,
            layout: if detached.is_some()
                && !lint_lines
                && self
                    .comment_binding(comments, gap, None)
                    .allows_relocation(None)
            {
                CommentRunLayout::Lines
            } else {
                CommentRunLayout::Source
            },
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
        let start = keyword
            .detached
            .map_or(printed.end(), |start| printed.end().min(start));
        let span = Span::new(start, keyword.span.start);
        let comments = self.comments.between(span)?;
        if let Some(first) = comments.first() {
            let first = first.span.start;
            let source_end = span.start;
            let binding = self.comment_binding(comments, span, None);
            if keyword.detached.is_some() {
                writer.hard_line();
            } else {
                self.line_constraint(source_end, first).write(
                    writer,
                    FlatSeparator::Space,
                    |writer| {
                        if !binding.allows_relocation(None) {
                            self.comment_gap(source_end, first, writer);
                        } else {
                            let breaks = self.line_breaks(source_end, first);
                            if keyword.adjacent && breaks == 0 {
                                writer.space();
                            } else {
                                writer.hard_line();
                                if breaks == 2 {
                                    writer.hard_line();
                                }
                            }
                        }
                    },
                );
            }
            self.comments.claim(comments)?;
            self.format_comment_run(comments, keyword.layout, writer)?;
        }
        let following = if !keyword.following.requires_line()
            && comments
                .last()
                .is_some_and(|comment| comment.kind == CommentKind::Line)
        {
            CommentGap::LineComment
        } else {
            keyword.following
        };
        following.write(writer, FlatSeparator::Space, |writer| {
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
