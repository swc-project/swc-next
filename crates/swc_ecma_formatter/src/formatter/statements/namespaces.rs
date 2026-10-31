//! TypeScript declaration containers retain their names, actual bodies and physical gaps.

use swc_next_ecma_ast::{
    GetSpan, Span, TsGlobalDeclaration, TsModuleBodyData, TsModuleDeclaration,
    TsModuleDeclarationKind, TsModuleName, TsModuleNameData,
};

use super::{
    DeclarationKeyword, Formatter, PrintedStatement, StatementBoundary, StatementPunctuation,
};
use crate::formatter::diagnostics::FlatSeparator;
use crate::formatter::literals::StringContext;
use crate::formatter::suppression::ContentMode;
use crate::formatter::types::{TypeBoundary, TypeGap};
use crate::{FormatError, document::Writer};

struct NamespaceHead<'ast> {
    name: TsModuleName,
    name_mode: ContentMode,
    prefix: Option<TypeGap<'ast>>,
    body: NamespaceBody<'ast>,
}

enum NamespaceEnd {
    Block(u32),
    Forward(u32),
}

enum NamespaceBody<'ast> {
    Absent,
    Block {
        node: swc_next_ecma_ast::TsModuleBlock,
        mode: ContentMode,
        prefix: TypeGap<'ast>,
    },
    Nested {
        node: TsModuleDeclaration,
        name_mode: ContentMode,
        boundary: TypeBoundary<'ast>,
    },
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_namespace_declaration(
        &mut self,
        node: TsModuleDeclaration,
        keyword: Option<DeclarationKeyword>,
        punctuation: StatementPunctuation,
        writer: &mut Writer<'ast>,
    ) -> Result<PrintedStatement<'ast>, FormatError> {
        let token = match node.kind(self.ast) {
            TsModuleDeclarationKind::Namespace => "namespace",
            TsModuleDeclarationKind::Module => "module",
        };
        let keyword_end = keyword
            .map_or(node.span(self.ast).start + token.len() as u32, |keyword| {
                keyword.name_prefix_start()
            });
        let incoming =
            if keyword.is_some_and(|keyword| keyword.suppresses_module_name(node.id(self.ast))) {
                ContentMode::Verbatim
            } else {
                ContentMode::Formatted
            };
        let head = self.prepare_namespace_head(node, Some(keyword_end), incoming)?;
        writer.text(token);
        let end = writer.group(false, |writer| self.format_namespace_head(head, writer))?;
        match end {
            NamespaceEnd::Forward(end) => {
                let mut printed = self.finish_statement(end, None, punctuation, writer)?;
                printed.boundary = printed.boundary.type_declaration();
                Ok(printed)
            }
            NamespaceEnd::Block(end) => Ok(StatementBoundary::TypeDeclarationEnd {
                end,
                has_terminator: false,
            }
            .into()),
        }
    }

    fn prepare_namespace_head(
        &self,
        node: TsModuleDeclaration,
        keyword_end: Option<u32>,
        incoming: ContentMode,
    ) -> Result<NamespaceHead<'ast>, FormatError> {
        let ast = self.ast;
        let name = node.id(ast);
        let name_span = name.span(ast);
        let mut name_mode = incoming;
        let prefix = keyword_end.map(|end| {
            let gap = Span::new(end, name_span.start);
            let source = self.has_suppression
                && self.content_mode(self.comments.in_range(gap)) == ContentMode::Verbatim;
            if source {
                name_mode = ContentMode::Verbatim;
            }
            self.prepare_type_gap(gap, source)
        });
        let body = match node.body(ast).map(|body| ast.ts_module_body_data(body)) {
            None => NamespaceBody::Absent,
            Some(TsModuleBodyData::TsModuleBlock(body)) => {
                let gap = Span::new(name_span.end, body.span(ast).start);
                let source = self.has_suppression
                    && self.content_mode(self.comments.in_range(gap)) == ContentMode::Verbatim;
                NamespaceBody::Block {
                    node: body,
                    mode: if source {
                        ContentMode::Verbatim
                    } else {
                        ContentMode::Formatted
                    },
                    prefix: self.prepare_type_gap(gap, source),
                }
            }
            Some(TsModuleBodyData::TsModuleDeclaration(nested)) => {
                let gap = Span::new(name_span.end, nested.id(ast).span(ast).start);
                let boundary = self.prepare_qualified_name_boundary(gap)?;
                if boundary.left_mode == ContentMode::Verbatim {
                    name_mode = boundary.left_mode;
                }
                NamespaceBody::Nested {
                    node: nested,
                    name_mode: boundary.right_mode,
                    boundary,
                }
            }
        };
        Ok(NamespaceHead {
            name,
            name_mode,
            prefix,
            body,
        })
    }

    fn format_namespace_head(
        &mut self,
        head: NamespaceHead<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<NamespaceEnd, FormatError> {
        if let Some(prefix) = head.prefix {
            self.format_type_gap(prefix, FlatSeparator::Space, writer)?;
        }
        if head.name_mode == ContentMode::Verbatim {
            self.format_verbatim(head.name.span(self.ast), writer)?;
        } else {
            match self.ast.ts_module_name_data(head.name) {
                TsModuleNameData::BindingIdentifier(name) => {
                    writer.text(self.ast.get_utf8(name.name(self.ast)))
                }
                TsModuleNameData::StringLiteral(name) => {
                    self.format_string_literal(name, StringContext::Expression, writer)
                }
                TsModuleNameData::TsQualifiedName(name) => {
                    self.format_qualified_type_name(name, writer)?
                }
            }
        }
        match head.body {
            NamespaceBody::Absent => Ok(NamespaceEnd::Forward(head.name.span(self.ast).end)),
            NamespaceBody::Block { node, mode, prefix } => {
                self.format_type_gap(prefix, FlatSeparator::Space, writer)?;
                self.format_module_body(node, mode, writer)?;
                Ok(NamespaceEnd::Block(node.span(self.ast).end))
            }
            NamespaceBody::Nested {
                node,
                name_mode,
                boundary,
            } => {
                let nested = self.prepare_namespace_head(node, None, name_mode)?;
                self.format_type_boundary(&boundary, writer)?;
                self.format_namespace_head(nested, writer)
            }
        }
    }

    pub(super) fn format_global_declaration(
        &mut self,
        node: TsGlobalDeclaration,
        keyword: Option<DeclarationKeyword>,
        writer: &mut Writer<'ast>,
    ) -> Result<PrintedStatement<'ast>, FormatError> {
        let ast = self.ast;
        let id = node.id(ast);
        let body = node.body(ast);
        let gap = Span::new(id.span(ast).end, body.span(ast).start);
        let source = self.has_suppression
            && self.content_mode(self.comments.in_range(gap)) == ContentMode::Verbatim;
        let prefix = self.prepare_type_gap(gap, source);
        if keyword.is_some_and(|keyword| keyword.suppresses_global(id)) {
            self.format_verbatim(id.span(ast), writer)?;
        } else {
            writer.text("global");
        }
        self.format_type_gap(prefix, FlatSeparator::Space, writer)?;
        self.format_module_body(
            body,
            if source {
                ContentMode::Verbatim
            } else {
                ContentMode::Formatted
            },
            writer,
        )?;
        Ok(StatementBoundary::TypeDeclarationEnd {
            end: body.span(ast).end,
            has_terminator: false,
        }
        .into())
    }
}
