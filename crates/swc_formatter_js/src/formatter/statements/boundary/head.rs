//! Guard placement at a statement's emitted head and its leading comments.

use swc_next_ecma_ast::{Comment, CommentKind, Expr, StmtData};

use super::super::{
    PreparedStatement, StatementBoundary, StatementItem, StatementSequenceKind, StatementSourceLine,
};
use crate::formatter::Formatter;
use crate::formatter::comments::{CommentBinding, HeadGapPlacement, NodeHead};
use crate::formatter::diagnostics::LineConstraint;
use crate::formatter::expressions::{
    ExpressionContext, ExpressionPosition, ExpressionStart, StringStatement,
};
use crate::formatter::suppression::ContentMode;
use crate::{FormatError, Semicolons, document::Writer};

pub(in crate::formatter::statements) struct StatementStart {
    pub(in crate::formatter::statements) expression: ExpressionStart,
    pub(in crate::formatter::statements) header_prefix: usize,
    pub(in crate::formatter::statements) terminate_previous: bool,
}

impl StatementStart {
    #[inline(always)]
    pub(in crate::formatter::statements) fn resolve(
        formatter: &Formatter<'_, '_>,
        preceding: Option<(StatementBoundary, LineConstraint)>,
        line: StatementSourceLine,
        first: Option<&StatementSequenceKind<'_>>,
        current: &PreparedStatement<'_>,
        mode: ContentMode,
        leading: &[Comment],
    ) -> Result<Self, FormatError> {
        let mut start = Self {
            expression: ExpressionStart::StatementGuarded,
            header_prefix: 0,
            terminate_previous: false,
        };
        if formatter.options.semicolons == Semicolons::AsNeeded {
            let mut guarded = line.incoming;
            if let Some((boundary, constraint)) = preceding {
                start.terminate_previous = constraint == LineConstraint::SameLine
                    && !boundary.has_terminator()
                    && current.item.is_nonempty(formatter.ast);
                guarded |= start.terminate_previous || boundary.guards_expression();
            }
            if !guarded {
                start.expression = ExpressionStart::StatementPending;
            }
            let first_program = matches!(first, Some(StatementSequenceKind::Program { .. }));
            let ignored_case = mode == ContentMode::Verbatim
                && matches!(first, Some(StatementSequenceKind::Case { .. }));
            if first_program
                || !leading.is_empty() && (ignored_case || formatter.has_file_directives)
            {
                start.place_before_comments(
                    formatter,
                    first_program,
                    ignored_case,
                    current.item,
                    leading,
                )?;
            }
        }
        if current
            .prologue
            .needs_barrier(current.item, mode, formatter.ast)
        {
            start.expression = ExpressionStart::StatementBarrier {
                terminate_previous: first.is_none()
                    && !start.terminate_previous
                    && preceding.is_some_and(|(boundary, _)| !boundary.has_terminator()),
            };
        }
        Ok(start)
    }

    fn place_before_comments(
        &mut self,
        formatter: &Formatter<'_, '_>,
        first_program: bool,
        ignored_case: bool,
        current: StatementItem,
        leading: &[Comment],
    ) -> Result<(), FormatError> {
        if ignored_case
            && leading.iter().any(|&comment| {
                comment.kind == CommentKind::Line
                    && formatter.comment_semantics(comment).suppression
            })
        {
            // The case colon already separates this ignored expression.
            self.expression = ExpressionStart::StatementGuarded;
        }
        if first_program && formatter.program_starts_with_jsdoc_enclosure(current, leading)? {
            self.expression = ExpressionStart::StatementGuarded;
        } else if formatter.has_file_directives
            && leading
                .iter()
                .any(|&comment| formatter.comment_is_file_directive(comment))
            && leading
                .iter()
                .any(|&comment| formatter.comment_has_target_annotation(comment))
        {
            if let Some(prefix) = formatter.program_header_guard_prefix(leading) {
                // Place the guard after the file header and before its target annotations.
                self.header_prefix = prefix;
            } else if first_program {
                self.expression = ExpressionStart::StatementGuarded;
            }
        }
        Ok(())
    }
}

impl Formatter<'_, '_> {
    fn program_starts_with_jsdoc_enclosure(
        &self,
        item: StatementItem,
        leading: &[Comment],
    ) -> Result<bool, FormatError> {
        if leading
            .iter()
            .any(|&comment| self.comment_needs_jsdoc_enclosure(comment))
        {
            return Ok(true);
        }
        let StatementItem::Statement(statement) = item else {
            return Ok(false);
        };
        let StmtData::ExpressionStatement(statement) = self.ast.stmt_data(statement) else {
            return Ok(false);
        };
        self.expression_has_jsdoc_enclosure_head(
            statement.expression(self.ast),
            ExpressionContext::at(ExpressionPosition::Statement(
                StringStatement::Parenthesized,
            )),
        )
    }

    fn program_header_guard_prefix(&self, comments: &[Comment]) -> Option<usize> {
        let end = comments
            .iter()
            .rposition(|&comment| self.comment_is_file_directive(comment))?
            + 1;
        comments[..end]
            .iter()
            .all(|&comment| {
                let semantics = self.comment_semantics(comment);
                !semantics.suppression
                    && !semantics.diagnostic
                    && !semantics.has_target_annotation()
                    && !self.comment_needs_jsdoc_enclosure(comment)
            })
            .then_some(end)
    }
}

/// Leading comments that must stay between the guard and their expression.
pub(in crate::formatter) struct StatementCommentGuard {
    pub(in crate::formatter) prefix_len: usize,
    following: Option<HeadGapPlacement>,
}

impl StatementCommentGuard {
    pub(in crate::formatter) fn write(self, start: ExpressionStart, writer: &mut Writer<'_>) {
        start.write_statement_guard(writer);
        match self.following {
            Some(HeadGapPlacement::OwnLine) => writer.hard_line(),
            Some(HeadGapPlacement::Inline) => writer.space(),
            None => {}
        }
    }
}

impl ExpressionStart {
    pub(in crate::formatter) fn needs_statement_guard(self) -> bool {
        matches!(self, Self::StatementBarrier { .. }) || self.needs_asi_guard()
    }

    /// Terminating a prior directive does not itself create an empty statement.
    pub(in crate::formatter) fn write_statement_guard(self, writer: &mut Writer<'_>) {
        let guard = if matches!(
            self,
            Self::StatementBarrier {
                terminate_previous: true
            }
        ) {
            ";;"
        } else {
            ";"
        };
        writer.text(guard);
    }

    pub(in crate::formatter) fn needs_asi_guard(self) -> bool {
        matches!(self, Self::StatementPending | Self::StatementGuard)
    }

    /// Resolve a pending guard only when the actual formatted or copied head is known.
    #[inline]
    pub(in crate::formatter) fn guard_for_head(
        self,
        continues: impl FnOnce() -> Result<bool, FormatError>,
    ) -> Result<Option<Self>, FormatError> {
        let needed =
            matches!(self, Self::StatementBarrier { .. }) || self.needs_asi_guard() && continues()?;
        Ok(needed.then_some(self))
    }

    #[inline]
    pub(in crate::formatter) fn guard_at_comments<'ast>(
        self,
        formatter: &Formatter<'ast, '_>,
        expression: Expr,
        context: &ExpressionContext<'ast>,
        head: &mut NodeHead<'ast>,
        binding: CommentBinding,
        released: &[Comment],
    ) -> Result<Option<StatementCommentGuard>, FormatError> {
        if !self.needs_statement_guard()
            || head.comments.is_empty()
            || !formatter.has_target_annotations && !context.is_verbatim()
        {
            return Ok(None);
        }
        self.place_at_annotation(formatter, expression, context, head, binding, released)
    }

    fn place_at_annotation<'ast>(
        self,
        formatter: &Formatter<'ast, '_>,
        expression: Expr,
        context: &ExpressionContext<'ast>,
        head: &mut NodeHead<'ast>,
        binding: CommentBinding,
        released: &[Comment],
    ) -> Result<Option<StatementCommentGuard>, FormatError> {
        let keep_contact = context.is_verbatim()
            && head.layout.is_source()
            && head.comments.last().is_some_and(|comment| {
                comment.kind == CommentKind::Block
                    && formatter.line_breaks(comment.span.end, head.content_start) == 0
            });
        let source_bound = matches!(binding, CommentBinding::SourceBound);
        let protected = if formatter.has_target_annotations {
            head.comments
                .iter()
                .position(|&comment| formatter.comment_has_target_annotation(comment))
                .or_else(|| {
                    // Removing a source pair can join the exterior run to a bound inner head.
                    (source_bound
                        && !head.comments.is_empty()
                        && released
                            .iter()
                            .any(|&comment| formatter.comment_has_target_annotation(comment)))
                    .then_some(0)
                })
        } else {
            None
        };
        if !keep_contact && protected.is_none() {
            return Ok(None);
        }
        if self
            .guard_for_head(|| {
                Ok(self == Self::StatementGuard
                    || formatter.expression_needs_asi_guard(expression, *context)?)
            })?
            .is_none()
        {
            return Ok(None);
        }
        if keep_contact {
            head.gap_end = None;
        }
        let Some(index) = protected else {
            return Ok(None);
        };
        let prefix_len = if source_bound { 0 } else { index };
        let protected = &head.comments[prefix_len..];
        let following = if source_bound {
            Some(HeadGapPlacement::OwnLine)
        } else if formatter.comment_is_type_cast(protected[0]) {
            None
        } else if context.is_verbatim() && formatter.has_suppression {
            formatter
                .source_position_starts_line(protected[0].span.start)
                .then_some(HeadGapPlacement::OwnLine)
        } else {
            protected
                .iter()
                .any(|comment| comment.kind == CommentKind::Line)
                .then_some(HeadGapPlacement::Inline)
        };
        Ok(Some(StatementCommentGuard {
            prefix_len,
            following,
        }))
    }
}
