//! TypeScript module leaves retain real names, references and expression owners.

use swc_next_ecma_ast::{
    Expr, GetSpan, ImportOrExportKind, Span, TsExportAssignment, TsImportEqualsDeclaration,
    TsModuleReference, TsModuleReferenceData, TsNamespaceExportDeclaration,
};

use super::{DeclarationKeyword, Formatter, PrintedStatement, StatementPunctuation};
use crate::formatter::comments::{CommentRunLayout, NodeHead};
use crate::formatter::diagnostics::FlatSeparator;
use crate::formatter::expressions::ExpressionContext;
use crate::formatter::expressions::imports::ImportArguments;
use crate::formatter::suppression::ContentMode;
use crate::{FormatError, document::Writer};

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_import_equals_declaration(
        &mut self,
        node: TsImportEqualsDeclaration,
        punctuation: StatementPunctuation,
        writer: &mut Writer<'ast>,
    ) -> Result<PrintedStatement<'ast>, FormatError> {
        let ast = self.ast;
        let span = node.span(ast);
        let identifier = node.id(ast);
        let name_span = identifier.span(ast);
        let reference = node.module_reference(ast);
        let reference_span = reference.span(ast);
        let import_end = span.start + "import".len() as u32;
        let type_keyword = if node.import_kind(ast) == ImportOrExportKind::Type {
            Some(self.module_operator(Span::new(import_end, name_span.start), "type")?)
        } else {
            None
        };
        let keyword_span = type_keyword.unwrap_or(Span::new(span.start, import_end));
        let boundary =
            self.prepare_module_reference_boundary(Span::new(name_span.end, reference_span.start))?;
        let mut name_mode = boundary.left_mode;
        let mut reference_mode = boundary.right_mode;
        if self.has_suppression {
            let prefix = self
                .comments
                .in_range(Span::new(import_end, name_span.start));
            if self.content_mode(prefix) == ContentMode::Verbatim {
                name_mode = ContentMode::Verbatim;
            }
            let trailing = self
                .comments
                .in_range(Span::new(reference_span.end, span.end));
            if self.content_mode(trailing) == ContentMode::Verbatim {
                reference_mode = ContentMode::Verbatim;
            }
        }
        let keyword = DeclarationKeyword::for_name(keyword_span, identifier, name_mode);
        writer.text("import");
        if let Some(token) = type_keyword {
            self.format_module_gap(import_end, token.start, true, writer)?;
            writer.text("type");
        }
        self.format_module_gap(keyword.span.end, name_span.start, true, writer)?;
        if keyword.suppresses_name(identifier) {
            self.format_verbatim(name_span, writer)?;
        } else {
            writer.text(ast.get_utf8(identifier.name(ast)));
        }
        self.format_type_boundary(&boundary, writer)?;
        self.format_ts_module_reference(reference, reference_mode, writer)?;
        let mut printed = self.finish_statement(reference_span.end, None, punctuation, writer)?;
        printed.boundary = printed.boundary.type_declaration();
        Ok(printed)
    }

    fn format_ts_module_reference(
        &mut self,
        reference: TsModuleReference,
        mode: ContentMode,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        if mode == ContentMode::Verbatim {
            return self.format_verbatim(reference.span(self.ast), writer);
        }
        let ast = self.ast;
        match ast.ts_module_reference_data(reference) {
            TsModuleReferenceData::TsTypeName(name) => self.format_type_name(name, mode, writer),
            TsModuleReferenceData::TsExternalModuleReference(node) => {
                let span = node.span(ast);
                let source = node.expression(ast);
                let source_span = source.span(ast);
                let keyword_end = span.start + "require".len() as u32;
                let opening_gap = Span::new(keyword_end, source_span.start);
                let comments = self.comments.in_range(opening_gap);
                let opening_start = self
                    .find_token(opening_gap, comments, "(")
                    .ok_or_else(|| self.unsupported("external module opening boundary", node))?;
                let opening = Span::new(opening_start, opening_start + 1);
                let prefix_count =
                    comments.partition_point(|comment| comment.span.end <= opening.start);
                let prefix = &comments[..prefix_count];
                let target = self.has_suppression.then_some(source_span);
                let binding =
                    self.comment_binding(prefix, Span::new(keyword_end, opening.end), target);
                let move_prefix = binding.allows_relocation(target);
                let content = Span::new(
                    if move_prefix && !prefix.is_empty() {
                        keyword_end
                    } else {
                        opening.end
                    },
                    span.end - 1,
                );
                writer.text("require");
                if !move_prefix {
                    self.format_module_gap(keyword_end, opening.start, false, writer)?;
                }
                self.format_import_arguments(
                    ImportArguments {
                        source: Expr::StringLiteral(source),
                        options: None,
                        content,
                        opening: opening.start,
                        last_mode: ContentMode::Formatted,
                    },
                    writer,
                )
            }
        }
    }

    pub(super) fn format_ts_export_assignment(
        &mut self,
        node: TsExportAssignment,
        punctuation: StatementPunctuation,
        writer: &mut Writer<'ast>,
    ) -> Result<PrintedStatement<'ast>, FormatError> {
        let ast = self.ast;
        let expression = node.expression(ast);
        let span = expression.span(ast);
        let keyword_end = node.span(ast).start + "export".len() as u32;
        let prefix = Span::new(keyword_end, span.start);
        let operator = self.module_operator(prefix, "=")?;
        let prefix_mode = if self.has_suppression {
            self.content_mode(self.comments.in_range(prefix))
        } else {
            ContentMode::Formatted
        };
        let mode = if self.has_suppression
            && self.content_mode(
                self.comments
                    .in_range(Span::new(span.end, node.span(ast).end)),
            ) == ContentMode::Verbatim
        {
            ContentMode::Verbatim
        } else {
            prefix_mode
        };
        let context = ExpressionContext::variable_initializer()
            .with_tail_boundary(punctuation.tail_boundary())
            .with_content_mode(mode);
        writer.text("export");
        let tail = if prefix_mode == ContentMode::Verbatim {
            self.format_verbatim(prefix, writer)?;
            self.format_expression(expression, context, writer)?
        } else {
            self.format_module_gap(keyword_end, operator.start, true, writer)?;
            writer.text("=");
            let comments = self.comments.between(Span::new(operator.end, span.start))?;
            let comments = self.comments.take(comments.len());
            writer.space();
            self.format_expression_with_leading_comments(
                expression,
                NodeHead {
                    comments,
                    content_start: span.start,
                    layout: CommentRunLayout::Source,
                    gap_end: comments.last().map(|_| span.start),
                },
                context,
                None,
                writer,
            )?
        };
        self.finish_statement(span.end, tail, punctuation, writer)
    }

    pub(super) fn format_ts_namespace_export(
        &mut self,
        node: TsNamespaceExportDeclaration,
        punctuation: StatementPunctuation,
        writer: &mut Writer<'ast>,
    ) -> Result<PrintedStatement<'ast>, FormatError> {
        let ast = self.ast;
        let name = node.id(ast);
        let name_span = name.span(ast);
        let keyword_end = node.span(ast).start + "export".len() as u32;
        let prefix = Span::new(keyword_end, name_span.start);
        let as_token = self.module_operator(prefix, "as")?;
        let namespace_token =
            self.module_operator(Span::new(as_token.end, name_span.start), "namespace")?;
        let prefix_mode = if self.has_suppression {
            self.content_mode(self.comments.in_range(prefix))
        } else {
            ContentMode::Formatted
        };
        let mode = if self.has_suppression
            && self.content_mode(
                self.comments
                    .in_range(Span::new(name_span.end, node.span(ast).end)),
            ) == ContentMode::Verbatim
        {
            ContentMode::Verbatim
        } else {
            prefix_mode
        };
        let before_as = self.prepare_type_gap(
            Span::new(keyword_end, as_token.start),
            prefix_mode == ContentMode::Verbatim,
        );
        let before_namespace = self.prepare_type_gap(
            Span::new(as_token.end, namespace_token.start),
            prefix_mode == ContentMode::Verbatim,
        );
        let before_name = self.prepare_type_gap(
            Span::new(namespace_token.end, name_span.start),
            prefix_mode == ContentMode::Verbatim,
        );
        writer.text("export");
        self.format_type_gap(before_as, FlatSeparator::Space, writer)?;
        writer.text("as");
        self.format_type_gap(before_namespace, FlatSeparator::Space, writer)?;
        writer.text("namespace");
        self.format_type_gap(before_name, FlatSeparator::Space, writer)?;
        if mode == ContentMode::Verbatim {
            self.format_verbatim(name_span, writer)?;
        } else {
            writer.text(ast.get_utf8(name.name(ast)));
        }
        let mut printed = self.finish_statement(name_span.end, None, punctuation, writer)?;
        printed.boundary = printed.boundary.type_declaration();
        Ok(printed)
    }
}
