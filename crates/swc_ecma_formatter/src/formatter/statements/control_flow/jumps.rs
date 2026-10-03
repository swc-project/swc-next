//! Restricted jump labels and the real owner of a labeled statement's header.

use swc_next_ecma_ast::{
    BreakStatement, Comment, ContinueStatement, DeclData, Expr, ExprData, GetSpan, LabelIdentifier,
    LabeledStatement, Span, StmtData,
};

use super::super::{
    Formatter, PrintedStatement, StatementContent, StatementItem, StatementPunctuation,
};
use super::{ControlledBodyEnd, ControlledBodyRole};
use crate::formatter::comments::{CommentBinding, CommentRunLayout};
use crate::formatter::diagnostics::FlatSeparator;
use crate::formatter::expressions::{ExpressionContext, ExpressionPosition};
use crate::formatter::suppression::{ContentMode, SuppressionPosition};
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
                StatementContent::Semicolon(content_end),
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
        writer: &mut Writer<'ast>,
    ) -> Result<PrintedStatement<'ast>, FormatError> {
        let ast = self.ast;
        let label = node.label(ast);
        let statement = node.body(ast);
        let head = Span::new(label.span(ast).end, statement.span(ast).start);
        let comments = self.comments.in_range(head);
        let colon = self
            .find_token(head, comments, ":")
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
            return self.format_verbatim_statement(span, content, punctuation, writer);
        }
        let mut body = self
            .prepare_controlled_body(
                statement,
                colon + 1,
                ControlledBodyEnd::Enclosing(node.span(ast).end),
                ControlledBodyRole::Label,
                body_mode,
            )
            .with_enclosing_punctuation(punctuation);
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
            self.format_comment_run(after, CommentRunLayout::Leading(head.end), writer)?;
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
        self.source_line_starts_line(first.span.start)
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
        if self.diagnostic_range_is_sensitive(span) {
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
