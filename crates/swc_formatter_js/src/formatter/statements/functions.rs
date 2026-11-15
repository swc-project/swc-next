//! Bodyless declarations consume real typed signatures and declaration endings.

use swc_next_ecma_ast::{BindingIdentifier, GetSpan, Span, TsFunction};

use super::declarations::AmbientDeclarationHead;
use super::{Formatter, PrintedStatement, StatementBoundary, StatementPunctuation};
use crate::formatter::diagnostics::FlatSeparator;
use crate::formatter::functions::SignaturePreceding;
use crate::formatter::functions::bodyless::{PreparedBodylessSignature, SourceBodylessSignature};
use crate::formatter::suppression::ContentMode;
use crate::formatter::types::TypeGap;
use crate::{FormatError, document::Writer};

struct FunctionName<'ast> {
    node: BindingIdentifier,
    mode: ContentMode,
    prefix: TypeGap<'ast>,
}

struct BodylessDeclaration<'ast> {
    generator: bool,
    async_prefix: Option<TypeGap<'ast>>,
    star_prefix: Option<TypeGap<'ast>>,
    name: Option<FunctionName<'ast>>,
    signature: PreparedBodylessSignature<'ast>,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_bodyless_function(
        &mut self,
        node: TsFunction,
        ambient: Option<AmbientDeclarationHead<'ast>>,
        punctuation: StatementPunctuation,
        writer: &mut Writer<'ast>,
    ) -> Result<PrintedStatement<'ast>, FormatError> {
        let ast = self.ast;
        let source = self.source_bodyless_signature(node);
        let parameter_span = source.parameter_span;
        let enclosing_prefix = self.has_suppression
            && !source.has_parameters()
            && node.id(ast).is_none()
            && source.generic.is_none()
            && source.annotation.is_none()
            && self.content_mode(
                self.comments
                    .in_range(Span::new(node.span(ast).start, parameter_span.start)),
            ) == ContentMode::Verbatim;
        if enclosing_prefix || self.bodyless_empty_parameters_are_verbatim(&source) {
            let span = node.span(ast);
            self.format_verbatim(span, writer)?;
            return Ok(StatementBoundary::TypeDeclarationEnd {
                end: span.end,
                has_terminator: self.source(span).ends_with(';'),
            }
            .into());
        }
        let declaration = self.prepare_bodyless_declaration(node, ambient.as_ref(), source)?;
        if let Some(head) = ambient {
            self.format_ambient_declaration_prefix(&head, writer)?;
        }
        let content_end = declaration.signature.content_end;
        let tail = writer.group(false, |writer| {
            if let Some(gap) = declaration.async_prefix {
                writer.text("async");
                self.format_type_gap(gap, FlatSeparator::Space, writer)?;
            }
            writer.text("function");
            if let Some(gap) = declaration.star_prefix {
                self.format_type_gap(gap, FlatSeparator::Empty, writer)?;
            }
            if declaration.generator {
                writer.text("*");
            }
            if let Some(name) = declaration.name {
                self.format_type_gap(name.prefix, FlatSeparator::Space, writer)?;
                if name.mode == ContentMode::Verbatim {
                    self.format_verbatim(name.node.span(ast), writer)?;
                } else {
                    writer.text(ast.get_utf8(name.node.name(ast)));
                }
            }
            self.format_bodyless_signature_suffix(declaration.signature, writer)
        })?;
        let mut printed = self.finish_statement(content_end, tail, punctuation, writer)?;
        printed.boundary = printed.boundary.type_declaration();
        Ok(printed)
    }

    fn prepare_bodyless_declaration(
        &self,
        node: TsFunction,
        ambient: Option<&AmbientDeclarationHead<'ast>>,
        source: SourceBodylessSignature,
    ) -> Result<BodylessDeclaration<'ast>, FormatError> {
        let ast = self.ast;
        let span = node.span(ast);
        let signature_start = source.header_start(ast);
        let name = node.id(ast);
        let head_end = name.map_or(signature_start, |name| name.span(ast).start);
        let function_start = if let Some(head) = ambient {
            head.keyword.span.start
        } else if node.r#async(ast) {
            let search = Span::new(span.start + 5, head_end);
            self.find_token(search, self.comments.in_range(search), "function")
                .ok_or_else(|| self.unsupported("bodyless async function keyword", node))?
        } else {
            span.start
        };
        let keyword_end = function_start + 8;
        let async_prefix = node
            .r#async(ast)
            .then(|| self.prepare_type_gap(Span::new(span.start + 5, function_start), false));
        let star = if node.generator(ast) {
            let search = Span::new(keyword_end, head_end);
            Some(
                self.find_token(search, self.comments.in_range(search), "*")
                    .ok_or_else(|| self.unsupported("bodyless generator star", node))?,
            )
        } else {
            None
        };
        let deferred_name_start = ambient.and_then(|head| {
            let start = head.keyword.name_prefix_start();
            (start < function_start).then_some(start)
        });
        let star_prefix = star
            .filter(|_| deferred_name_start.is_none())
            .map(|start| self.prepare_type_gap(Span::new(keyword_end, start), false));
        let head_start =
            deferred_name_start.unwrap_or_else(|| star.map_or(keyword_end, |start| start + 1));
        let mut name = name.map(|name| {
            let gap = Span::new(head_start, name.span(ast).start);
            let suppressed = self.has_suppression
                && self.content_mode(self.comments.in_range(gap)) == ContentMode::Verbatim;
            let mode =
                if suppressed || ambient.is_some_and(|head| head.keyword.suppresses_name(name)) {
                    ContentMode::Verbatim
                } else {
                    ContentMode::Formatted
                };
            FunctionName {
                node: name,
                mode,
                prefix: self.prepare_type_gap(gap, suppressed),
            }
        });
        let prefix_start = name
            .as_ref()
            .map_or(head_start, |name| name.node.span(ast).end);
        let signature = self.prepare_bodyless_signature_suffix(
            source,
            prefix_start,
            name.as_ref()
                .map(|name| SignaturePreceding::Name(name.node)),
            name.as_ref()
                .map_or(ContentMode::Formatted, |name| name.mode),
        )?;
        if let Some(name) = &mut name {
            name.mode = signature.preceding_mode;
        }
        Ok(BodylessDeclaration {
            generator: node.generator(ast),
            async_prefix,
            star_prefix,
            name,
            signature,
        })
    }
}
