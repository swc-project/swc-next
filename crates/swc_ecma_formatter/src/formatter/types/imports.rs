//! Type import arguments, qualifier and type arguments keep distinct real owners.

use swc_next_ecma_ast::{
    Expr, GetSpan, Span, TsImportType, TsImportTypeQualifier, TsImportTypeQualifierData,
};

use super::generics::TypeListKind;
use super::{Formatter, TypeBoundaryTarget};
use crate::formatter::comments::ExpressionTail;
use crate::formatter::expressions::imports::ImportArguments;
use crate::formatter::suppression::ContentMode;
use crate::{FormatError, document::Writer};

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_import_type(
        &mut self,
        node: TsImportType,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let ast = self.ast;
        let span = node.span(ast);
        let source = node.source(ast);
        let source_span = source.span(ast);
        let options = node.options(ast);
        let qualifier = node.qualifier(ast);
        let arguments = node.type_arguments(ast);
        let keyword_end = span.start + "import".len() as u32;
        let opening_gap = Span::new(keyword_end, source_span.start);
        let comments = self.comments.in_range(opening_gap);
        let opening = self
            .find_token(opening_gap, comments, "(")
            .ok_or_else(|| self.unsupported("import type opening boundary", node))?;
        let prefix_count = comments.partition_point(|comment| comment.span.end <= opening);
        let prefix = &comments[..prefix_count];
        let target = self.has_suppression.then_some(source_span);
        let binding = self.comment_binding(prefix, Span::new(keyword_end, opening + 1), target);
        let move_prefix = binding.allows_relocation(target);
        let last_end = options.map_or(source_span.end, |node| node.span(ast).end);
        let suffix_start = qualifier.map_or_else(
            || arguments.map_or(span.end, |node| node.span(ast).start),
            |node| node.span(ast).start,
        );
        let closing_gap = Span::new(last_end, suffix_start);
        let closing = self
            .find_token(closing_gap, self.comments.in_range(closing_gap), ")")
            .ok_or_else(|| self.unsupported("import type closing boundary", node))?;
        let content = Span::new(
            if move_prefix && !prefix.is_empty() {
                keyword_end
            } else {
                opening + 1
            },
            closing,
        );
        let qualifier = qualifier
            .map(|node| {
                let boundary = self.prepare_type_boundary(
                    Span::new(closing + 1, node.span(ast).start),
                    TypeBoundaryTarget::Qualified,
                )?;
                Ok::<_, FormatError>((node, boundary))
            })
            .transpose()?;
        let arguments = arguments.map(|node| {
            let head_end = qualifier
                .as_ref()
                .map_or(closing + 1, |(node, _)| node.span(ast).end);
            let prefix = self.prepare_type_list_prefix(
                Span::new(head_end, node.span(ast).start),
                TypeListKind::Arguments,
            );
            (node, prefix)
        });
        let last_mode = qualifier
            .as_ref()
            .map_or(ContentMode::Formatted, |(_, boundary)| boundary.left_mode);
        let qualifier_mode = arguments
            .as_ref()
            .map_or(ContentMode::Formatted, |(_, prefix)| prefix.left_mode);
        writer.text("import");
        if !move_prefix {
            self.format_source_token_gap(keyword_end, opening, false, writer)?;
        }
        self.format_import_arguments(
            ImportArguments {
                source: Expr::StringLiteral(source),
                options,
                content,
                opening,
                last_mode,
            },
            writer,
        )?;
        if let Some((node, boundary)) = qualifier {
            let mode = if boundary.right_mode == ContentMode::Verbatim {
                ContentMode::Verbatim
            } else {
                qualifier_mode
            };
            self.format_type_boundary(&boundary, writer)?;
            self.format_import_type_qualifier(node, mode, writer)?;
        }
        if let Some((node, ref prefix)) = arguments {
            let mode = prefix.right_mode;
            self.format_type_list_prefix(prefix, writer)?;
            self.format_type_arguments(node, mode, writer)?;
        }
        Ok(None)
    }

    fn format_import_type_qualifier(
        &mut self,
        node: TsImportTypeQualifier,
        mode: ContentMode,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        if mode == ContentMode::Verbatim {
            return self.format_verbatim(node.span(self.ast), writer);
        }
        match self.ast.ts_import_type_qualifier_data(node) {
            TsImportTypeQualifierData::IdentifierName(node) => {
                writer.text(self.ast.get_utf8(node.name(self.ast)));
                Ok(())
            }
            TsImportTypeQualifierData::TsQualifiedName(node) => {
                self.format_qualified_type_name(node, writer)
            }
        }
    }
}
