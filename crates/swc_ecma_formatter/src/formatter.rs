//! Formatting context and the program entry point.

mod bindings;
mod classes;
mod comments;
mod decorators;
mod diagnostics;
mod expressions;
mod functions;
mod heritage;
mod literals;
mod parameters;
mod restricted;
mod source;
mod statements;
mod suppression;
mod templates;
mod types;
mod variables;

use swc_next_ecma_ast::{Ast, AwaitExpression, GetSpan, Program, Span, StmtData};

use self::comments::CommentCursor;
use self::statements::{StatementItem, StatementSequenceKind};
use crate::{FormatError, FormatOptions, document::Writer};

pub(crate) struct Formatter<'ast, 'source> {
    ast: &'ast Ast<'source>,
    options: FormatOptions,
    comments: CommentCursor<'ast>,
    diagnostic_lines: diagnostics::DiagnosticLines,
    has_suppression: bool,
    has_target_annotations: bool,
    has_type_casts: bool,
    has_file_directives: bool,
    enclosing_await: Option<AwaitExpression>,
    for_initializer_span: Option<Span>,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(crate) fn new(ast: &'ast Ast<'source>, options: FormatOptions) -> Self {
        let mut formatter = Self {
            ast,
            options,
            comments: CommentCursor::new(ast.comments()),
            diagnostic_lines: diagnostics::DiagnosticLines::new(ast.source(), ast.comments()),
            has_suppression: ast
                .comments()
                .iter()
                .any(|&comment| suppression::is_suppression_comment(ast, comment)),
            has_target_annotations: false,
            has_type_casts: false,
            has_file_directives: false,
            enclosing_await: None,
            for_initializer_span: None,
        };
        for &comment in ast.comments() {
            if !formatter.has_type_casts {
                formatter.has_type_casts = formatter.comment_is_type_cast(comment);
            }
            if !formatter.has_target_annotations {
                formatter.has_target_annotations =
                    formatter.has_type_casts || formatter.comment_is_optimizer_annotation(comment);
            }
            if !formatter.has_file_directives {
                formatter.has_file_directives = formatter.comment_is_file_directive(comment);
            }
            if formatter.has_type_casts && formatter.has_file_directives {
                break;
            }
        }
        formatter
    }

    fn program_header_barrier(&self, program: Program) -> Option<Span> {
        let ast = self.ast;
        if !self.has_file_directives || !program.directives(ast).is_empty() {
            return None;
        }
        let mut statements = program.body(ast).iter();
        let first = ast.get_node_in_sub_range(statements.next()?);
        let StmtData::EmptyStatement(empty) = ast.stmt_data(first) else {
            return None;
        };
        let barrier = empty.span(ast);
        let end = statements
            .map(|id| ast.get_node_in_sub_range(id))
            .find(|&statement| !matches!(ast.stmt_data(statement), StmtData::EmptyStatement(_)))
            .map_or(ast.source().len() as u32, |statement| {
                statement.span(ast).start
            });
        let comments = self
            .comments
            .in_range(Span::new(program.span(ast).start, end));
        let before = comments.partition_point(|comment| comment.span.end <= barrier.start);
        let original = comments[..before]
            .iter()
            .rev()
            .find_map(|&comment| self.comment_checking_mode(comment))?;
        let formatted = comments[before..]
            .iter()
            .rev()
            .find_map(|&comment| self.comment_checking_mode(comment))
            .unwrap_or(original);
        // Newly active annotations may not override an existing header binding.
        (formatted != original).then_some(barrier)
    }

    pub(crate) fn format_program(mut self, writer: &mut Writer<'ast>) -> Result<(), FormatError> {
        let program = self.ast.root_program();
        // With no comment-owning child, an ignore directive belongs to the program.
        if self.has_suppression
            && program.directives(self.ast).is_empty()
            && program.body(self.ast).iter().all(|id| {
                matches!(
                    self.ast.stmt_data(self.ast.get_node_in_sub_range(id)),
                    StmtData::EmptyStatement(_)
                )
            })
        {
            let start = if self.ast.source().starts_with('\u{feff}') {
                3
            } else {
                0
            };
            return self.format_verbatim(Span::new(start, self.ast.source().len() as u32), writer);
        }
        let hashbang = program.hashbang(self.ast);
        if let Some(hashbang) = hashbang {
            writer.text("#!");
            writer.text(self.ast.get_utf8(hashbang.value(self.ast)).trim_end());
        }
        let ast = self.ast;
        let statements = program
            .body(ast)
            .iter()
            .map(|id| StatementItem::Statement(ast.get_node_in_sub_range(id)));
        let directives = program
            .directives(ast)
            .iter()
            .map(|id| StatementItem::Directive(ast.get_node_in_sub_range(id)));
        let has_content = self.format_statement_sequence(
            directives.chain(statements).peekable(),
            program.span(ast),
            hashbang.map(|node| node.span(ast).end),
            StatementSequenceKind::Program {
                header_barrier: self.program_header_barrier(program),
                has_source_items: !program.body(ast).is_empty()
                    || !program.directives(ast).is_empty(),
            },
            writer,
        )?;
        if let Some(comment) = self.comments.first() {
            return Err(FormatError::unsupported(
                "comment placement",
                None,
                comment.span,
            ));
        }
        if has_content.has_content {
            writer.hard_line();
        }
        Ok(())
    }

    fn unsupported<N: swc_next_ecma_ast::AstNode + GetSpan>(
        &self,
        construct: &'static str,
        node: N,
    ) -> FormatError {
        FormatError::unsupported(
            construct,
            Some(self.ast.node_kind(node.node_id())),
            node.span(self.ast),
        )
    }
}
