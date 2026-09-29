//! Shared function-body source modes and transferred arrow block comments.
use super::Formatter;
use super::diagnostics::LineConstraint;
use super::statements::{IncomingStatement, StatementBodyLeading};
use super::suppression::{ContentMode, SuppressionPosition, is_suppression_comment};
use swc_next_ecma_ast::{Comment, CommentKind, FunctionBody, GetSpan, Span, Stmt, StmtData};
pub(super) struct ArrowBodyModes {
    pub(super) body: ContentMode,
    pub(super) first_statement: Option<IncomingStatement>,
    pub(super) constraint: LineConstraint,
}
impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn function_body_content_mode(
        &self,
        body: FunctionBody,
        mode: ContentMode,
    ) -> ContentMode {
        if mode == ContentMode::Verbatim
            || !self.has_suppression
            || self.function_body_requires_break(body)
        {
            return mode;
        }
        let span = body.span(self.ast);
        self.content_mode(
            self.comments
                .in_range(Span::new(span.start + 1, span.end - 1)),
        )
    }

    pub(super) fn prepare_arrow_block_leading(
        &self,
        comments: &'ast [Comment],
        body: FunctionBody,
    ) -> (&'ast [Comment], StatementBodyLeading<'ast>) {
        let Some(first_line) = comments
            .iter()
            .position(|comment| comment.kind == CommentKind::Line)
        else {
            return (comments, StatementBodyLeading::empty());
        };
        // A contiguous body suffix retains source order even when other packets
        // remain at the signature. Mixed attachment cannot reorder that stream.
        if comments[first_line..]
            .iter()
            .any(|comment| comment.kind != CommentKind::Line)
        {
            return (comments, StatementBodyLeading::empty());
        }
        let mut start = first_line;
        {
            // Arrow own-line comments lead the block itself; only an end-of-line
            // packet after a real parameter belongs to the first body item.
            if comments.len() != first_line + 1
                || self.source_position_starts_line(comments[first_line].span.start)
            {
                return (comments, StatementBodyLeading::empty());
            }
            while start != 0 && self.source_comment_gap_is_run(comments[start - 1], comments[start])
            {
                if self.source_position_starts_line(comments[start - 1].span.start) {
                    break;
                }
                start -= 1;
            }
        }
        let ast = self.ast;
        let target = body
            .body(ast)
            .iter()
            .map(|id| ast.get_node_in_sub_range(id))
            .find(|statement: &Stmt| {
                !matches!(ast.stmt_data(*statement), StmtData::EmptyStatement(_))
            });
        if !body.directives(ast).is_empty() {
            let span = body.span(ast);
            let limit = target.map_or(span.end - 1, |statement| statement.span(ast).start);
            // Moving past existing directive comments would reverse source order.
            // Keep the original boundary packet when those owners precede the target.
            if !self
                .comments
                .in_range(Span::new(span.start + 1, limit))
                .is_empty()
            {
                return (comments, StatementBodyLeading::empty());
            }
        }
        let (boundary, leading) = comments.split_at(start);
        (boundary, StatementBodyLeading::new(leading, target))
    }

    pub(super) fn arrow_signature_body_modes(
        &self,
        body: FunctionBody,
        comments: &[Comment],
        body_start: u32,
        source_end: u32,
        has_parameters: bool,
    ) -> ArrowBodyModes {
        let end = comments
            .last()
            .map_or(source_end, |comment| comment.span.end);
        let mut modes = ArrowBodyModes {
            body: ContentMode::Formatted,
            first_statement: None,
            constraint: self.line_constraint(end, body_start),
        };
        if !self.has_suppression {
            return modes;
        }
        let mut needs_first_statement = false;
        for (run, position) in self.suppressed_comment_runs(comments, body_start) {
            for &comment in run {
                if !is_suppression_comment(self.ast, comment) {
                    continue;
                }
                if has_parameters
                    && (comment.kind == CommentKind::Line
                        || matches!(position, SuppressionPosition::EndOfLine))
                {
                    needs_first_statement = true;
                } else {
                    modes.body = ContentMode::Verbatim;
                }
            }
        }
        if needs_first_statement {
            let ast = self.ast;
            modes.first_statement = body
                .body(ast)
                .iter()
                .map(|id| ast.get_node_in_sub_range(id))
                .find(|&statement| !matches!(ast.stmt_data(statement), StmtData::EmptyStatement(_)))
                .map(|statement| IncomingStatement {
                    span: statement.span(ast),
                    mode: ContentMode::Verbatim,
                });
        }
        if modes.body == ContentMode::Verbatim || modes.first_statement.is_some() {
            modes.constraint = self.source_line_constraint(end, body_start);
        }
        modes
    }
}
