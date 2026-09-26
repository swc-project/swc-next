//! Formatting context and the program entry point.

mod comments;
mod diagnostics;
mod expressions;
mod literals;
mod source;
mod statements;

use swc_next_ecma_ast::{Ast, GetSpan};

use self::comments::CommentCursor;
use self::statements::{StatementItem, StatementSequenceKind};
use crate::{FormatError, FormatOptions, document::Writer};

pub(crate) struct Formatter<'ast, 'source> {
    ast: &'ast Ast<'source>,
    options: FormatOptions,
    comments: CommentCursor<'ast>,
    diagnostic_lines: diagnostics::DiagnosticLines,
    has_target_annotations: bool,
    has_type_casts: bool,
    has_file_directives: bool,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(crate) fn new(ast: &'ast Ast<'source>, options: FormatOptions) -> Self {
        let mut formatter = Self {
            ast,
            options,
            comments: CommentCursor::new(ast.comments()),
            diagnostic_lines: diagnostics::DiagnosticLines::new(ast.source(), ast.comments()),
            has_target_annotations: false,
            has_type_casts: false,
            has_file_directives: false,
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

    pub(crate) fn format_program(mut self, writer: &mut Writer<'ast>) -> Result<(), FormatError> {
        let program = self.ast.root_program();
        if self.ast.source().starts_with('\u{feff}') {
            writer.text("\u{feff}");
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
            StatementSequenceKind::Program,
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
