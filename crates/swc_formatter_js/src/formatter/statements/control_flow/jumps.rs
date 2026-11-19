//! Restricted jump labels and the real owner of a labeled statement's header.

use swc_next_ecma_ast::{
    BreakStatement, Comment, ContinueStatement, DeclData, Expr, ExprData, GetSpan, LabelIdentifier,
    LabeledStatement, Span, Stmt, StmtData,
};

use super::super::{
    Formatter, PrintedStatement, StatementContent, StatementItem, StatementPunctuation,
};
use super::{ControlledBodyEnd, ControlledBodyRole};
use crate::formatter::comments::{CommentBinding, CommentCursor, CommentRunLayout};
use crate::formatter::diagnostics::FlatSeparator;
use crate::formatter::expressions::{ExpressionContext, ExpressionPosition};
use crate::formatter::suppression::{
    CommentDirective, ContentMode, LintLineDirective, SuppressionPosition,
};
use crate::{FormatError, document::Writer};

#[derive(Clone, Copy)]
pub(in crate::formatter::statements) enum JumpStatement {
    Break(BreakStatement),
    Continue(ContinueStatement),
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(in crate::formatter::statements) fn format_jump_statement(
        &mut self,
        jump: JumpStatement,
        punctuation: StatementPunctuation,
        writer: &mut Writer<'ast>,
    ) -> Result<PrintedStatement<'ast>, FormatError> {
        let ast = self.ast;
        let (keyword, label, span) = match jump {
            JumpStatement::Break(node) => ("break", node.label(ast), node.span(ast)),
            JumpStatement::Continue(node) => ("continue", node.label(ast), node.span(ast)),
        };
        let keyword_end = span.start + keyword.len() as u32;
        let content_end = label.map_or(keyword_end, |label| label.span(ast).end);
        if self.trailing_content_mode(Span::new(content_end, span.end), true)
            == ContentMode::Verbatim
        {
            return self.format_verbatim_statement(
                span,
                StatementContent::Semicolon(content_end).into(),
                punctuation,
                writer,
            );
        }
        let mode = if self.has_suppression {
            label.map_or(ContentMode::Formatted, |label| {
                self.content_mode(
                    self.comments
                        .in_range(Span::new(keyword_end, label.span(ast).start)),
                )
            })
        } else {
            ContentMode::Formatted
        };
        writer.text(keyword);
        if let Some(label) = label {
            self.format_source_token_gap(keyword_end, label.span(ast).start, true, writer)?;
            self.format_jump_label(label, mode, writer)?;
        }
        self.finish_statement(content_end, None, punctuation, writer)
    }

    fn format_jump_label(
        &mut self,
        label: LabelIdentifier,
        mode: ContentMode,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        if mode == ContentMode::Verbatim {
            self.format_verbatim(label.span(self.ast), writer)
        } else {
            writer.text(self.ast.get_utf8(label.name(self.ast)));
            Ok(())
        }
    }

    pub(in crate::formatter::statements) fn format_labeled_statement(
        &mut self,
        node: LabeledStatement,
        punctuation: StatementPunctuation,
        source_candidate: crate::formatter::statements::ContinuationBoundary,
        controlled: bool,
        writer: &mut Writer<'ast>,
    ) -> Result<PrintedStatement<'ast>, FormatError> {
        let span = node.span(self.ast);
        let prepared = self
            .label_chain_head
            .is_some_and(|head| head.start <= span.start && span.start < head.end);
        if prepared
            || !matches!(
                self.ast.stmt_data(node.body(self.ast)),
                StmtData::LabeledStatement(_)
            )
            || self
                .comments
                .first()
                .is_none_or(|comment| comment.span.start >= span.end)
        {
            return self.format_labeled_contents(
                node,
                punctuation,
                source_candidate,
                controlled,
                writer,
            );
        }
        let (head, prefix) = self.prepare_label_chain_head(node)?;
        let outer = self.label_chain_head.replace(head);
        let result = (|| {
            self.comments.claim(prefix)?;
            if !prefix.is_empty() {
                if controlled
                    || prefix.iter().any(|&comment| {
                        matches!(
                            self.comment_directive(comment),
                            Some(CommentDirective::LintLine(LintLineDirective::SameLine))
                        )
                    })
                {
                    self.format_comment_run(prefix, CommentRunLayout::Leading(head.end), writer)?;
                } else {
                    self.format_comment_run(prefix, CommentRunLayout::Source, writer)?;
                    writer.hard_line();
                }
            }
            self.format_labeled_contents(node, punctuation, source_candidate, controlled, writer)
        })();
        self.label_chain_head = outer;
        result
    }

    fn prepare_label_chain_head(
        &self,
        node: LabeledStatement,
    ) -> Result<(Span, &'ast [Comment]), FormatError> {
        let ast = self.ast;
        let start = node.span(ast).start;
        let mut current = node;
        let mut prefix_end = start;
        let end = loop {
            let label = current.label(ast);
            let statement = current.body(ast);
            let head = Span::new(label.span(ast).end, statement.span(ast).start);
            let comments = self.comments.in_range(head);
            let colon = self
                .find_token(head, comments, ":")
                .ok_or_else(|| self.unsupported("label colon boundary", current))?;
            let split = comments.partition_point(|comment| comment.span.end <= colon);
            let (before, after) = comments.split_at(split);
            let (expression, declaration) = match ast.stmt_data(statement) {
                StmtData::ExpressionStatement(node) => (Some(node.expression(ast)), false),
                StmtData::Declaration(node) => {
                    (None, matches!(ast.decl_data(node), DeclData::Function(_)))
                }
                _ => (None, false),
            };
            if !self
                .control_flow_comment_binding(before, Span::new(head.start, colon), None, false)?
                .allows_relocation(None)
            {
                break head.end;
            }
            let move_before = self.label_comments_lead_statement(before, colon);
            if !before.is_empty() && !move_before {
                break head.end;
            }
            if move_before {
                prefix_end = before.last().map_or(prefix_end, |comment| comment.span.end);
            }
            if !self
                .control_flow_comment_binding(
                    after,
                    Span::new(colon + 1, head.end),
                    expression,
                    declaration,
                )?
                .allows_relocation(None)
            {
                break head.end;
            }
            if (before.is_empty() || move_before)
                && self.label_comments_lead_statement(after, head.end)
            {
                prefix_end = after.last().map_or(prefix_end, |comment| comment.span.end);
            }
            let StmtData::LabeledStatement(next) = ast.stmt_data(statement) else {
                break head.end;
            };
            current = next;
        };
        Ok((
            Span::new(start, end),
            self.comments.in_range(Span::new(start, prefix_end)),
        ))
    }

    fn format_labeled_contents(
        &mut self,
        node: LabeledStatement,
        punctuation: StatementPunctuation,
        source_candidate: crate::formatter::statements::ContinuationBoundary,
        controlled: bool,
        writer: &mut Writer<'ast>,
    ) -> Result<PrintedStatement<'ast>, FormatError> {
        let ast = self.ast;
        let label = node.label(ast);
        let statement = node.body(ast);
        let head = Span::new(label.span(ast).end, statement.span(ast).start);
        let comments = self.comments.in_range(head);
        let token_comments = if self.label_chain_head.is_some_and(|chain| {
            chain.start <= node.span(ast).start && node.span(ast).start < chain.end
        }) {
            CommentCursor::new(ast.comments()).in_range(head)
        } else {
            comments
        };
        let colon = self
            .find_token(head, token_comments, ":")
            .ok_or_else(|| self.unsupported("label colon boundary", node))?;
        let mut label_mode = ContentMode::Formatted;
        let mut body_mode = ContentMode::Formatted;
        let mut enclosing_mode = ContentMode::Formatted;
        if self.has_suppression {
            for (run, position) in self.suppressed_comment_runs(comments, head.end) {
                if !matches!(position, SuppressionPosition::Inline) {
                    enclosing_mode = ContentMode::Verbatim;
                    continue;
                }
                if run.last().is_some_and(|comment| comment.span.end <= colon) {
                    label_mode = ContentMode::Verbatim;
                } else {
                    body_mode = ContentMode::Verbatim;
                }
            }
        }
        if enclosing_mode == ContentMode::Verbatim {
            let span = node.span(ast);
            let content = self.statement_content(StatementItem::Statement(statement));
            let source_fence = Some((
                StatementItem::Statement(Stmt::LabeledStatement(node)),
                source_candidate,
            ));
            return self.format_verbatim_statement_with_source_fence(
                span,
                content.into(),
                punctuation,
                source_fence,
                writer,
            );
        }
        let mut body = self
            .prepare_controlled_body(
                statement,
                colon + 1,
                ControlledBodyEnd::Enclosing(node.span(ast).end),
                ControlledBodyRole::Label,
                body_mode,
            )
            .with_enclosing_boundary(punctuation, source_candidate);
        let split = comments.partition_point(|comment| comment.span.end <= colon);
        let (before, after) = comments.split_at(split);
        let (expression, declaration) = match ast.stmt_data(statement) {
            StmtData::ExpressionStatement(node) => (Some(node.expression(ast)), false),
            StmtData::Declaration(node) => {
                (None, matches!(ast.decl_data(node), DeclData::Function(_)))
            }
            _ => (None, false),
        };
        let move_before = self.label_comments_lead_statement(before, colon)
            && self
                .control_flow_comment_binding(before, Span::new(head.start, colon), None, false)?
                .allows_relocation(None);
        // A body packet cannot pass an earlier packet retained in the label header.
        let move_after = (before.is_empty() || move_before)
            && self.label_comments_lead_statement(after, head.end)
            && self
                .control_flow_comment_binding(
                    after,
                    Span::new(colon + 1, head.end),
                    expression,
                    declaration,
                )?
                .allows_relocation(None);
        let before_boundary = self.prepare_comment_boundary(before, head.start, colon);
        // Claim the complete movable prefix once; both owners retain their borrowed views.
        if move_after {
            self.comments.claim(comments)?;
            body.leading = &[];
        } else if move_before {
            self.comments.claim(before)?;
        }
        if move_before {
            self.format_comment_run(before, CommentRunLayout::Leading(colon), writer)?;
        }
        if move_after {
            if controlled
                || after.iter().any(|&comment| {
                    matches!(
                        self.comment_directive(comment),
                        Some(CommentDirective::LintLine(LintLineDirective::SameLine))
                    )
                })
            {
                self.format_comment_run(after, CommentRunLayout::Leading(head.end), writer)?;
            } else {
                self.format_comment_run(after, CommentRunLayout::Source, writer)?;
                if after
                    .last()
                    .is_some_and(|comment| self.comment_run_ends_line(*comment, head.end))
                {
                    writer.hard_line();
                } else if !after.is_empty() {
                    writer.space();
                }
            }
        }
        self.format_jump_label(label, label_mode, writer)?;
        if !move_before {
            if !move_after {
                self.comments.claim(before)?;
            }
            self.format_comment_run(before, CommentRunLayout::SourceAfter(head.start), writer)?;
            before_boundary
                .gap()
                .write(writer, FlatSeparator::Empty, |_| {});
        }
        writer.text(":");
        self.format_controlled_body(body, writer)
    }

    fn label_comments_lead_statement(&self, comments: &[Comment], end: u32) -> bool {
        let Some(first) = comments.first() else {
            return false;
        };
        self.layout_line_starts_line(first.span.start)
            || comments
                .last()
                .is_some_and(|last| self.comment_run_ends_line(*last, end))
            || comments
                .windows(2)
                .any(|pair| self.line_breaks(pair[0].span.end, pair[1].span.start) != 0)
    }

    pub(in crate::formatter::statements::control_flow) fn control_flow_comment_binding(
        &self,
        comments: &[Comment],
        span: Span,
        expression: Option<Expr>,
        declaration: bool,
    ) -> Result<CommentBinding, FormatError> {
        if self.diagnostic_range_is_sensitive(span)
            || comments.iter().any(|&comment| {
                matches!(
                    self.comment_directive(comment),
                    Some(CommentDirective::LintRange | CommentDirective::Coverage)
                )
            })
        {
            return Ok(CommentBinding::SourceBound);
        }
        let cast = expression
            .is_some_and(|node| self.parenthesis_has_type_cast(node.span(self.ast).start));
        let mut invocation = false;
        let mut function = declaration;
        if self.has_target_annotations
            && let Some(mut expression) = expression
        {
            let mut context = ExpressionContext::at(ExpressionPosition::Statement);
            loop {
                if context.is_verbatim() {
                    return Ok(self.comment_binding(comments, span, None));
                }
                let data = self.ast.expr_data(expression);
                match data {
                    ExprData::CallExpression(_) | ExprData::NewExpression(_) => {
                        invocation = true;
                        break;
                    }
                    ExprData::Function(_) | ExprData::ArrowFunctionExpression(_) => {
                        function = true;
                        break;
                    }
                    ExprData::ParenthesizedExpression(node) => {
                        expression = node.expression(self.ast);
                    }
                    ExprData::SequenceExpression(node) => {
                        let Some(first) = node.expressions(self.ast).iter().next() else {
                            break;
                        };
                        expression = self.ast.get_node_in_sub_range(first);
                    }
                    _ => {
                        let Some((operand, next)) = self.expression_head_operand(data, context)?
                        else {
                            break;
                        };
                        expression = operand;
                        context = next;
                    }
                }
            }
        }
        Ok(
            self.comment_run_binding(comments.iter().copied(), |comment| {
                let semantics = self.comment_semantics(comment);
                let optimizer = semantics.optimizer
                    && ((invocation && self.source(comment.span).contains("__PURE__"))
                        || (function && self.source(comment.span).contains("__NO_SIDE_EFFECTS__")));
                if semantics.suppression
                    || semantics.diagnostic
                    || semantics.file_directive
                    || optimizer
                    || semantics.type_cast && cast
                {
                    CommentBinding::SourceBound
                } else {
                    // A colon or statement keyword supplies no expression target.
                    CommentBinding::Unbound
                }
            }),
        )
    }
}
