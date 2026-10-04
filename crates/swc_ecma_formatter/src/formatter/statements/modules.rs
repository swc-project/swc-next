//! Static module declarations, their typed names, and source-owned header gaps.

use swc_next_ecma_ast::{
    BindingIdentifier, Comment, CommentKind, Decl, DeclData, ExportAllDeclaration,
    ExportDefaultDeclaration, ExportDefaultDeclarationKindData, ExportNamedDeclaration,
    ExportSpecifier, Expr, GetSpan, ImportAttribute, ImportDeclaration, ImportDeclarationSpecifier,
    ImportDeclarationSpecifierData, ImportOrExportKind, ModuleExportName, ModuleExportNameData,
    PropertyKey, Span, StringLiteral, SubRange,
};

use super::{Formatter, PrintedStatement, StatementBoundary, StatementPunctuation};
use crate::formatter::bindings::InitializerLayout;
use crate::formatter::comments::{CommentRunLayout, NodeHead};
use crate::formatter::diagnostics::FlatSeparator;
use crate::formatter::expressions::arguments::{ArgumentLayout, FollowingItem, Separator};
use crate::formatter::expressions::assignment::{AssignmentBoundary, RhsLayout};
use crate::formatter::expressions::objects::{PropertyKeyContext, PropertyKeyQuoting};
use crate::formatter::expressions::{ExpressionContext, ExpressionPosition};
use crate::formatter::literals::StringContext;
use crate::formatter::suppression::{ContentMode, SuppressionPosition};
use crate::{FormatError, QuoteProperties, TrailingCommas, document::Writer};

#[derive(Clone, Copy)]
enum ModuleName {
    Export(ModuleExportName),
    Binding(BindingIdentifier),
}

impl ModuleName {
    fn span(self, formatter: &Formatter<'_, '_>) -> Span {
        match self {
            Self::Export(name) => name.span(formatter.ast),
            Self::Binding(name) => name.span(formatter.ast),
        }
    }
}

#[derive(Clone, Copy)]
enum ModuleItem {
    Import(ImportDeclarationSpecifier),
    Export(ExportSpecifier),
    Attribute(ImportAttribute),
}

impl ModuleItem {
    fn span(self, formatter: &Formatter<'_, '_>) -> Span {
        match self {
            Self::Import(node) => node.span(formatter.ast),
            Self::Export(node) => node.span(formatter.ast),
            Self::Attribute(node) => node.span(formatter.ast),
        }
    }
}

struct NameModes {
    left: ContentMode,
    right: ContentMode,
    enclosing: ContentMode,
}

enum ModuleItemContent<'ast> {
    Name(ModuleName),
    Alias {
        left: Option<ModuleName>,
        right: ModuleName,
        operator: Span,
        modes: NameModes,
        promoted: Option<Span>,
    },
    Attribute {
        quoting: PropertyKeyQuoting,
        key: ModuleExportName,
        value: StringLiteral,
        colon: Span,
        modes: NameModes,
        boundary: Option<AssignmentBoundary<'ast>>,
    },
}

struct ModuleNamePrefix<'ast> {
    span: Span,
    comments: &'ast [Comment],
    source: ModulePrefixSource,
    promoted: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ModulePrefixSource {
    Formatted,
    Name,
    Item,
}

struct PreparedModuleItem<'ast> {
    content: ModuleItemContent<'ast>,
    prefix: Option<ModuleNamePrefix<'ast>>,
}

struct ModuleList {
    key_quoting: PropertyKeyQuoting,
    span: Span,
    breakable: bool,
    expand: bool,
    named: bool,
}

enum ModuleSourcePrefix<'a> {
    SideEffect(u32),
    Standalone(Span),
    Braced {
        keyword_end: u32,
        list: &'a ModuleList,
        last: Option<Span>,
    },
    Star(Span),
}

struct ModuleAttributes {
    keyword: Span,
    list: ModuleList,
    mode: ContentMode,
    source_comments: Option<Span>,
}

struct ModuleSource {
    literal: StringLiteral,
    from: Option<Span>,
    mode: ContentMode,
    preceding: ContentMode,
    attributes: Option<ModuleAttributes>,
    empty_header: Option<Span>,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_declaration(
        &mut self,
        declaration: Decl,
        mode: ContentMode,
        punctuation: StatementPunctuation,
        writer: &mut Writer<'ast>,
    ) -> Result<PrintedStatement<'ast>, FormatError> {
        let data = self.ast.decl_data(declaration);
        if mode == ContentMode::Verbatim {
            let span = declaration.span(self.ast);
            let end = self.declaration_content_end(declaration);
            self.format_verbatim(Span::new(span.start, end), writer)?;
            return match data {
                DeclData::VariableDeclaration(_) => {
                    self.finish_statement(end, None, punctuation, writer)
                }
                DeclData::TsTypeAliasDeclaration(_) => {
                    let mut printed = self.finish_statement(end, None, punctuation, writer)?;
                    printed.boundary = printed.boundary.type_declaration();
                    Ok(printed)
                }
                _ => Ok(StatementBoundary::SourceEnd {
                    end,
                    has_terminator: false,
                }
                .into()),
            };
        }
        match data {
            DeclData::Function(function) => {
                self.format_function(function, writer)?;
                Ok(StatementBoundary::SourceEnd {
                    end: function.span(self.ast).end,
                    has_terminator: false,
                }
                .into())
            }
            DeclData::Class(class) => {
                self.format_class(class, ExpressionPosition::ExportDefault, writer)?;
                Ok(StatementBoundary::SourceEnd {
                    end: class.span(self.ast).end,
                    has_terminator: false,
                }
                .into())
            }
            DeclData::VariableDeclaration(variable) => {
                let (end, tail) = self.format_variable_declaration(variable, writer)?;
                self.finish_statement(end, tail, punctuation, writer)
            }
            DeclData::TsTypeAliasDeclaration(alias) => {
                let (end, tail) = self.format_type_alias(alias, writer)?;
                let mut printed = self.finish_statement(end, tail, punctuation, writer)?;
                printed.boundary = printed.boundary.type_declaration();
                Ok(printed)
            }
            _ => Err(self.unsupported("declaration", declaration)),
        }
    }

    pub(super) fn format_import_declaration(
        &mut self,
        node: ImportDeclaration,
        punctuation: StatementPunctuation,
        writer: &mut Writer<'ast>,
    ) -> Result<PrintedStatement<'ast>, FormatError> {
        let ast = self.ast;
        if node.phase(ast).is_some() {
            return Err(self.unsupported("import phase", node));
        }
        let span = node.span(ast);
        let keyword_end = span.start + 6;
        if node.import_kind(ast) == ImportOrExportKind::Type {
            return Err(self.unsupported("type-only import", node));
        }
        let mut specifiers = node.specifiers(ast);
        let first = specifiers.first().map(|id| ast.get_node_in_sub_range(id));
        let standalone = first.filter(|&node| {
            !matches!(
                ast.import_declaration_specifier_data(node),
                ImportDeclarationSpecifierData::ImportSpecifier(_)
            )
        });
        if standalone.is_some() {
            specifiers.remove_first();
        }
        let second = specifiers.first().map(|id| ast.get_node_in_sub_range(id));
        let namespace = second.filter(|&node| {
            matches!(
                ast.import_declaration_specifier_data(node),
                ImportDeclarationSpecifierData::ImportNamespaceSpecifier(_)
            )
        });
        if namespace.is_some() {
            specifiers.remove_first();
        }
        let first_named = specifiers.first().map(|id| ast.get_node_in_sub_range(id));
        let last = node
            .specifiers(ast)
            .last()
            .map(|id| ast.get_node_in_sub_range(id));
        let source_start = node.source(ast).span(ast).start;
        let list = if first_named.is_some() {
            Some(
                self.module_list(
                    standalone.map_or(keyword_end, |node| node.span(ast).end),
                    source_start,
                    first_named.map(|node| node.span(ast)),
                    specifiers
                        .last()
                        .map(|id| ast.get_node_in_sub_range(id).span(ast)),
                    true,
                    standalone.is_some(),
                )?,
            )
        } else if first.is_none() {
            let header = Span::new(keyword_end, source_start);
            let comments = self.comments.in_range(header);
            self.find_token(header, comments, "{")
                .map(|_| self.module_list(keyword_end, source_start, None, None, true, false))
                .transpose()?
        } else {
            None
        };
        let source_prefix = match list.as_ref() {
            Some(list) => ModuleSourcePrefix::Braced {
                keyword_end,
                list,
                last: last.map(|node| node.span(ast)),
            },
            None => match last {
                Some(node) => ModuleSourcePrefix::Standalone(node.span(ast)),
                None => ModuleSourcePrefix::SideEffect(keyword_end),
            },
        };
        let source = self.prepare_module_source(
            node.source(ast),
            node.attributes(ast),
            span.end,
            source_prefix,
        )?;
        writer.text("import");
        let mut end = keyword_end;
        let mut incoming = ContentMode::Formatted;
        if let Some(node) = standalone {
            let owner = node.span(ast);
            let next = namespace.or(first_named);
            let target_end = next.map_or(
                source.from.map_or(source_start, |token| token.start),
                |node| node.span(ast).start,
            );
            let gap_mode =
                self.content_mode(self.comments.in_range(Span::new(keyword_end, owner.start)));
            let suppression = self.delimited_suppression(
                owner,
                target_end,
                crate::formatter::suppression::DelimitedFollowing::Node,
            );
            let mode = if gap_mode == ContentMode::Verbatim
                || suppression
                    .as_ref()
                    .is_some_and(|mode| mode.preceding.is_some())
                || next.is_none() && source.preceding == ContentMode::Verbatim
            {
                ContentMode::Verbatim
            } else {
                ContentMode::Formatted
            };
            let content =
                self.prepare_module_item(ModuleItem::Import(node), PropertyKeyQuoting::Preserve)?;
            self.format_module_gap(end, owner.start, true, writer)?;
            self.format_module_item(owner, content, mode, writer)?;
            end = owner.end;
            if let Some(next) = next {
                incoming = suppression
                    .as_ref()
                    .map_or(ContentMode::Formatted, |mode| mode.following);
                let slot_start = list
                    .as_ref()
                    .filter(|_| namespace.is_none())
                    .map_or(next.span(ast).start, |list| list.span.start);
                if suppression
                    .as_ref()
                    .is_some_and(|mode| mode.preceding.is_some())
                {
                    let packet = self.format_delimited_source_gap(
                        Span::new(end, slot_start),
                        crate::formatter::suppression::DelimitedFollowing::Node,
                        writer,
                    )?;
                    packet
                        .following
                        .write(writer, FlatSeparator::Space, Writer::space);
                } else {
                    self.format_module_comma_gap(end, slot_start, writer)?;
                }
                end = slot_start;
            }
        }
        if let Some(node) = namespace {
            let owner = node.span(ast);
            let content =
                self.prepare_module_item(ModuleItem::Import(node), PropertyKeyQuoting::Preserve)?;
            self.format_module_item(
                owner,
                content,
                if source.preceding == ContentMode::Verbatim {
                    source.preceding
                } else {
                    incoming
                },
                writer,
            )?;
            end = owner.end;
        }
        if source.empty_header.is_some() {
            writer.space();
            writer.text("{}");
        } else if let Some(list) = list {
            if standalone.is_none() {
                let comments = self.comments.in_range(Span::new(end, list.span.start));
                incoming = self.content_mode(comments);
                self.format_module_gap(end, list.span.start, true, writer)?;
            }
            end = list.span.end;
            self.format_module_list(
                specifiers
                    .iter()
                    .map(|id| ModuleItem::Import(ast.get_node_in_sub_range(id)))
                    .peekable(),
                list,
                incoming,
                source.preceding,
                writer,
            )?;
        }
        let end = self.format_module_source(source, node.attributes(ast), end, writer)?;
        self.finish_statement(end, None, punctuation, writer)
    }

    pub(super) fn format_export_named(
        &mut self,
        node: ExportNamedDeclaration,
        punctuation: StatementPunctuation,
        writer: &mut Writer<'ast>,
    ) -> Result<PrintedStatement<'ast>, FormatError> {
        let ast = self.ast;
        let span = node.span(ast);
        let keyword_end = span.start + 6;
        if let Some(declaration) = node.declaration(ast) {
            let mode = self.content_mode(
                self.comments
                    .in_range(Span::new(keyword_end, declaration.span(ast).start)),
            );
            writer.text("export");
            self.format_module_gap(keyword_end, declaration.span(ast).start, true, writer)?;
            return self.format_declaration(declaration, mode, punctuation, writer);
        }
        if node.export_kind(ast) == ImportOrExportKind::Type {
            return Err(self.unsupported("type-only export", node));
        }
        let specifiers = node.specifiers(ast);
        let first = specifiers.first().map(|id| ast.get_node_in_sub_range(id));
        let last = specifiers.last().map(|id| ast.get_node_in_sub_range(id));
        let list = self.module_list(
            keyword_end,
            node.source(ast)
                .map_or(span.end, |node| node.span(ast).start),
            first.map(|node| node.span(ast)),
            last.map(|node| node.span(ast)),
            true,
            false,
        )?;
        let source = node
            .source(ast)
            .map(|literal| {
                self.prepare_module_source(
                    literal,
                    node.attributes(ast),
                    span.end,
                    ModuleSourcePrefix::Braced {
                        keyword_end,
                        list: &list,
                        last: last.map(|node| node.span(ast)),
                    },
                )
            })
            .transpose()?;
        if first.is_none()
            && source.is_none()
            && self.content_mode(
                self.comments
                    .in_range(Span::new(keyword_end, list.span.end - 1)),
            ) == ContentMode::Verbatim
        {
            self.format_verbatim(Span::new(span.start, list.span.end), writer)?;
            return self.finish_statement(list.span.end, None, punctuation, writer);
        }
        let comments = self
            .comments
            .in_range(Span::new(keyword_end, list.span.start));
        let mode = self.content_mode(comments);
        writer.text("export");
        let head_end = keyword_end;
        let empty_header = source
            .as_ref()
            .is_some_and(|source| source.empty_header.is_some());
        if empty_header {
            writer.space();
            writer.text("{}");
        } else {
            let dangling = first.is_none()
                && source.is_none()
                && self
                    .comments
                    .in_range(Span::new(keyword_end, list.span.start))
                    .last()
                    .is_some_and(|comment| comment.kind == CommentKind::Line);
            self.format_module_gap(head_end, list.span.start, true, writer)?;
            // The empty specifier prefix owns its literal space after dangling comments.
            if dangling {
                writer.space();
            }
        }
        let mut end = list.span.end;
        if !empty_header {
            self.format_module_list(
                specifiers
                    .iter()
                    .map(|id| ModuleItem::Export(ast.get_node_in_sub_range(id)))
                    .peekable(),
                list,
                mode,
                source
                    .as_ref()
                    .map_or(ContentMode::Formatted, |source| source.preceding),
                writer,
            )?;
        }
        if let Some(source) = source {
            end = self.format_module_source(source, node.attributes(ast), end, writer)?;
        }
        self.finish_statement(end, None, punctuation, writer)
    }

    pub(super) fn format_export_all(
        &mut self,
        node: ExportAllDeclaration,
        punctuation: StatementPunctuation,
        writer: &mut Writer<'ast>,
    ) -> Result<PrintedStatement<'ast>, FormatError> {
        let ast = self.ast;
        let span = node.span(ast);
        if node.export_kind(ast) == ImportOrExportKind::Type {
            return Err(self.unsupported("type-only export", node));
        }
        let header = Span::new(span.start + 6, node.source(ast).span(ast).start);
        let comments = self.comments.in_range(header);
        let Some(star) = self.find_token(header, comments, "*") else {
            return Err(self.unsupported("export namespace boundary", node));
        };
        let namespace = node
            .exported(ast)
            .map(|name| {
                let owner = Span::new(star, name.span(ast).end);
                self.prepare_module_namespace(owner, ModuleName::Export(name))
                    .map(|content| {
                        (
                            owner,
                            PreparedModuleItem {
                                content,
                                prefix: None,
                            },
                        )
                    })
            })
            .transpose()?;
        let end = namespace.as_ref().map_or(star + 1, |(owner, _)| owner.end);
        let prefix = namespace.as_ref().map_or(
            ModuleSourcePrefix::Star(Span::new(header.start, end)),
            |(owner, _)| ModuleSourcePrefix::Standalone(*owner),
        );
        let source =
            self.prepare_module_source(node.source(ast), node.attributes(ast), span.end, prefix)?;
        let incoming = if self.has_suppression && namespace.is_some() {
            self.content_mode(self.comments.in_range(Span::new(header.start, star)))
        } else {
            ContentMode::Formatted
        };
        writer.text("export");
        let head_end = header.start;
        if let Some((owner, content)) = namespace {
            let mode = if source.preceding == ContentMode::Verbatim {
                source.preceding
            } else {
                incoming
            };
            self.format_module_gap(head_end, star, true, writer)?;
            self.format_module_item(owner, content, mode, writer)?;
        } else {
            self.format_module_gap(head_end, star, true, writer)?;
            writer.text("*");
        }
        let end = self.format_module_source(source, node.attributes(ast), end, writer)?;
        self.finish_statement(end, None, punctuation, writer)
    }

    pub(super) fn format_export_default(
        &mut self,
        node: ExportDefaultDeclaration,
        punctuation: StatementPunctuation,
        writer: &mut Writer<'ast>,
    ) -> Result<PrintedStatement<'ast>, FormatError> {
        let ast = self.ast;
        let declaration = node.declaration(ast);
        let span = declaration.span(ast);
        let prefix = Span::new(node.span(ast).start + 6, span.start);
        let operator = self.module_operator(prefix, "default")?;
        let mode = self.content_mode(self.comments.in_range(prefix));
        let prefix_comments = self.comments.in_range(prefix);
        let moved_prefix = !prefix_comments.is_empty()
            && prefix_comments
                .iter()
                .all(|&comment| self.comment_semantics(comment).is_ordinary());
        writer.text("export");
        if moved_prefix {
            writer.space();
        } else {
            self.format_module_gap(prefix.start, operator.start, true, writer)?;
        }
        writer.text("default");
        let head_start = if moved_prefix {
            prefix.start
        } else {
            operator.end
        };
        match ast.export_default_declaration_kind_data(declaration) {
            ExportDefaultDeclarationKindData::Expr(expression) => {
                let comments = self.comments.between(Span::new(head_start, span.start))?;
                let comments = self.comments.take(comments.len());
                writer.space();
                let source = mode == ContentMode::Verbatim
                    || comments
                        .iter()
                        .any(|comment| comment.kind == CommentKind::Line);
                let tail = self.format_expression_with_leading_comments(
                    expression,
                    NodeHead {
                        comments,
                        content_start: span.start,
                        layout: if source {
                            CommentRunLayout::Source
                        } else {
                            CommentRunLayout::FlatLeading(span.start)
                        },
                        gap_end: source
                            .then_some(span.start)
                            .filter(|_| !comments.is_empty()),
                    },
                    ExpressionContext::at(ExpressionPosition::ExportDefault)
                        .with_content_mode(mode),
                    None,
                    writer,
                )?;
                self.finish_statement(span.end, tail, punctuation, writer)
            }
            kind => {
                let declaration = match kind {
                    ExportDefaultDeclarationKindData::Function(function) => {
                        Decl::Function(function)
                    }
                    ExportDefaultDeclarationKindData::Class(class) => Decl::Class(class),
                    _ => return Err(self.unsupported("default export declaration", declaration)),
                };
                if moved_prefix {
                    let comments = self.comments.between(prefix)?;
                    self.comments.claim(comments)?;
                    writer.space();
                    self.format_comment_run(
                        comments,
                        CommentRunLayout::FlatLeading(span.start),
                        writer,
                    )?;
                } else {
                    self.format_module_gap(operator.end, span.start, true, writer)?;
                }
                self.format_declaration(declaration, mode, punctuation, writer)
            }
        }
    }

    pub(super) fn module_source_content_end(
        &self,
        literal: StringLiteral,
        attributes: SubRange<ImportAttribute>,
        end: u32,
    ) -> u32 {
        let start = attributes.last().map_or(literal.span(self.ast).end, |id| {
            self.ast.get_node_in_sub_range(id).span(self.ast).end
        });
        let gap = Span::new(start, end);
        let comments = self.comments.in_range(gap);
        self.find_token(gap, comments, "}")
            .map_or(literal.span(self.ast).end, |position| position + 1)
    }

    pub(super) fn module_named_content_end(&self, node: ExportNamedDeclaration) -> u32 {
        if let Some(declaration) = node.declaration(self.ast) {
            return self.declaration_content_end(declaration);
        }
        if let Some(source) = node.source(self.ast) {
            return self.module_source_content_end(
                source,
                node.attributes(self.ast),
                node.span(self.ast).end,
            );
        }
        let start = node
            .specifiers(self.ast)
            .last()
            .map_or(node.span(self.ast).start + 6, |id| {
                self.ast.get_node_in_sub_range(id).span(self.ast).end
            });
        let gap = Span::new(start, node.span(self.ast).end);
        self.find_token(gap, self.comments.in_range(gap), "}")
            .map_or(node.span(self.ast).end, |position| position + 1)
    }

    pub(super) fn module_operator(&self, span: Span, token: &str) -> Result<Span, FormatError> {
        let comments = self.comments.in_range(span);
        let Some(start) = self.find_token(span, comments, token) else {
            return Err(FormatError::unsupported(
                "module token boundary",
                None,
                span,
            ));
        };
        Ok(Span::new(start, start + token.len() as u32))
    }

    fn module_list(
        &self,
        start: u32,
        end: u32,
        first: Option<Span>,
        last: Option<Span>,
        named: bool,
        standalone: bool,
    ) -> Result<ModuleList, FormatError> {
        let opening =
            self.module_operator(Span::new(start, first.map_or(end, |span| span.start)), "{")?;
        let closing = self.module_operator(
            Span::new(last.map_or(opening.end, |span| span.end), end),
            "}",
        )?;
        let span = Span::new(opening.start, closing.end);
        let comments = self.comments.in_range(span);
        let singleton = first == last;
        Ok(ModuleList {
            key_quoting: PropertyKeyQuoting::Preserve,
            span,
            breakable: !named || !singleton || standalone || !comments.is_empty(),
            expand: !named
                && self.line_breaks(opening.end, first.map_or(closing.start, |span| span.start))
                    != 0,
            named,
        })
    }

    fn prepare_module_source(
        &self,
        literal: StringLiteral,
        attributes: SubRange<ImportAttribute>,
        end: u32,
        prefix: ModuleSourcePrefix<'_>,
    ) -> Result<ModuleSource, FormatError> {
        let ast = self.ast;
        let (preceding_end, preceding, list) = match &prefix {
            ModuleSourcePrefix::SideEffect(end) => (*end, None, None),
            ModuleSourcePrefix::Star(header) => (header.end, None, None),
            ModuleSourcePrefix::Standalone(target) => (target.end, Some(*target), None),
            ModuleSourcePrefix::Braced { list, last, .. } => (list.span.end, *last, Some(*list)),
        };
        let source_span = literal.span(ast);
        let from = (!matches!(prefix, ModuleSourcePrefix::SideEffect(_)))
            .then(|| self.module_operator(Span::new(preceding_end, source_span.start), "from"))
            .transpose()?;
        let mut mode = self.content_mode(self.comments.in_range(Span::new(
            from.map_or(preceding_end, |token| token.end),
            source_span.start,
        )));
        if self.has_suppression
            && let ModuleSourcePrefix::Star(header) = &prefix
            && self.content_mode(self.comments.in_range(*header)) == ContentMode::Verbatim
        {
            mode = ContentMode::Verbatim;
        }
        let mut preceding_mode = ContentMode::Formatted;
        if let Some(from) = from {
            let before = self.comments.in_range(Span::new(preceding_end, from.start));
            for position in self.suppression_positions(before, from.start) {
                if matches!(position, SuppressionPosition::OwnLine) && list.is_none() {
                    mode = ContentMode::Verbatim;
                } else if preceding.is_some() {
                    preceding_mode = ContentMode::Verbatim;
                } else {
                    mode = ContentMode::Verbatim;
                }
            }
        }
        if let Some(list) = list
            && preceding.is_none()
            && self.content_mode(
                self.comments
                    .in_range(Span::new(list.span.start + 1, list.span.end - 1)),
            ) == ContentMode::Verbatim
        {
            mode = ContentMode::Verbatim;
        }
        let empty_header = match prefix {
            ModuleSourcePrefix::Braced {
                keyword_end,
                last: None,
                ..
            } => {
                let header = Span::new(keyword_end, source_span.start);
                (!self.diagnostic_range_is_sensitive(header)).then_some(header)
            }
            _ => None,
        };
        if let Some(header) = empty_header
            && self.content_mode(self.comments.in_range(header)) == ContentMode::Verbatim
        {
            mode = ContentMode::Verbatim;
        }
        let first = attributes.first().map(|id| ast.get_node_in_sub_range(id));
        let last = attributes.last().map(|id| ast.get_node_in_sub_range(id));
        let header = Span::new(
            source_span.end,
            first.map_or(end, |node| node.span(ast).start),
        );
        let comments = self.comments.in_range(header);
        let attribute_keyword = self
            .find_token(header, comments, "with")
            .map(|start| Span::new(start, start + 4))
            .or_else(|| {
                self.find_token(header, comments, "assert")
                    .map(|start| Span::new(start, start + 6))
            });
        let mut attribute_list = attribute_keyword
            .map(|token| {
                self.module_list(
                    token.end,
                    end,
                    first.map(|node| node.span(ast)),
                    last.map(|node| node.span(ast)),
                    false,
                    false,
                )
            })
            .transpose()?;
        if let Some(list) = attribute_list.as_mut() {
            list.key_quoting = match self.options.quote_properties {
                QuoteProperties::AsNeeded => PropertyKeyQuoting::AsNeeded,
                QuoteProperties::Preserve => PropertyKeyQuoting::Preserve,
            };
        }
        if let (Some(list), Some(first)) = (attribute_list.as_mut(), first) {
            let key = first.key(ast);
            let type_key = match ast.module_export_name_data(key) {
                ModuleExportNameData::IdentifierName(name) => {
                    ast.get_utf8(name.name(ast)) == "type"
                }
                ModuleExportNameData::StringLiteral(literal) => {
                    let raw = ast.get_utf8(literal.raw(ast));
                    &raw[1..raw.len() - 1] == "type"
                }
            };
            if attributes.len() == 1 && type_key && self.comments.in_range(list.span).is_empty() {
                list.breakable = false;
                list.expand = false;
            }
        }
        if let Some(list) = attribute_list.as_ref()
            && first.is_none()
            && self.content_mode(
                self.comments
                    .in_range(Span::new(list.span.start + 1, list.span.end - 1)),
            ) == ContentMode::Verbatim
        {
            mode = ContentMode::Verbatim;
        }
        let attributes = attribute_keyword
            .zip(attribute_list)
            .map(|(keyword, list)| {
                let head_end = if first.is_some() {
                    list.span.start
                } else {
                    list.span.end - 1
                };
                let header = Span::new(source_span.end, head_end);
                let comments = self.comments.in_range(header);
                let mut following_mode = ContentMode::Formatted;
                if first.is_none() {
                    if self.content_mode(comments) == ContentMode::Verbatim {
                        mode = ContentMode::Verbatim;
                    }
                } else {
                    for position in self.suppression_positions(comments, list.span.start) {
                        if matches!(position, SuppressionPosition::OwnLine) {
                            following_mode = ContentMode::Verbatim;
                        } else {
                            mode = ContentMode::Verbatim;
                        }
                    }
                }
                let source_comments = (!self.diagnostic_range_is_sensitive(header)
                    && (first.is_none()
                        || comments
                            .iter()
                            .all(|&comment| self.comment_semantics(comment).is_ordinary())))
                .then_some(header);
                ModuleAttributes {
                    keyword,
                    list,
                    mode: following_mode,
                    source_comments,
                }
            });
        Ok(ModuleSource {
            literal,
            from,
            mode,
            preceding: preceding_mode,
            attributes,
            empty_header,
        })
    }

    fn format_module_source(
        &mut self,
        source: ModuleSource,
        attributes: SubRange<ImportAttribute>,
        preceding_end: u32,
        writer: &mut Writer<'ast>,
    ) -> Result<u32, FormatError> {
        let ast = self.ast;
        let literal = source.literal;
        let mut end = preceding_end;
        if let Some(header) = source.empty_header {
            let comments = self.comments.between(header)?;
            self.comments.claim(comments)?;
            writer.text(" from");
            writer.space();
            self.format_comment_run(
                comments,
                CommentRunLayout::FlatLeading(literal.span(ast).start),
                writer,
            )?;
        } else {
            if let Some(from) = source.from {
                self.format_module_gap(end, from.start, true, writer)?;
                writer.text("from");
                end = from.end;
            }
            self.format_module_gap(end, literal.span(ast).start, true, writer)?;
        }
        if source.mode == ContentMode::Verbatim {
            self.format_verbatim(literal.span(ast), writer)?;
        } else {
            self.format_string_literal(literal, StringContext::Expression, writer);
        }
        end = literal.span(ast).end;
        if let Some(attributes_source) = source.attributes {
            let ModuleAttributes {
                keyword,
                list,
                mode,
                source_comments,
            } = attributes_source;
            if let Some(header) = source_comments {
                let comments = self.comments.between(header)?;
                self.comments.claim(comments)?;
                writer.space();
                self.format_comment_run(
                    comments,
                    CommentRunLayout::FlatLeading(header.end),
                    writer,
                )?;
                writer.text(self.source(keyword));
                writer.space();
            } else {
                self.format_module_gap(end, keyword.start, true, writer)?;
                writer.text(self.source(keyword));
                self.format_module_gap(keyword.end, list.span.start, true, writer)?;
            }
            end = list.span.end;
            if attributes.is_empty() && source_comments.is_some() {
                writer.text("{}");
            } else {
                self.format_module_list(
                    attributes
                        .iter()
                        .map(|id| ModuleItem::Attribute(ast.get_node_in_sub_range(id)))
                        .peekable(),
                    list,
                    mode,
                    ContentMode::Formatted,
                    writer,
                )?;
            }
        }
        Ok(end)
    }

    fn format_module_list(
        &mut self,
        mut items: std::iter::Peekable<impl Iterator<Item = ModuleItem>>,
        list: ModuleList,
        mut incoming: ContentMode,
        last_mode: ContentMode,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let content = Span::new(list.span.start + 1, list.span.end - 1);
        let Some(first) = items.next() else {
            writer.text("{");
            let comments = self.comments.between(content)?;
            let comments = self.comments.take(comments.len());
            if !comments.is_empty() {
                writer.indent(|writer| {
                    writer.space();
                    self.format_comment_run(comments, CommentRunLayout::Source, writer)
                })?;
                let end = comments
                    .last()
                    .map_or(content.start, |comment| comment.span.end);
                self.comment_gap(end, content.end, writer);
            }
            writer.text("}");
            return Ok(());
        };
        let mut write = |writer: &mut Writer<'ast>| {
            writer.text("{");
            let end = writer.indent(|writer| {
                if list.breakable {
                    self.opening_line_constraint(content).write(
                        writer,
                        if self.options.bracket_spacing {
                            FlatSeparator::Space
                        } else {
                            FlatSeparator::Empty
                        },
                        |writer| {
                            if self.options.bracket_spacing {
                                writer.line()
                            } else {
                                writer.soft_line()
                            }
                        },
                    );
                } else if self.options.bracket_spacing {
                    writer.space();
                }
                let mut current = Some((first, self.prepare_module_item(first, list.key_quoting)?));
                let mut end = content.start;
                while let Some((item, prepared_content)) = current {
                    let next = items.next();
                    let span = item.span(self);
                    let following = match next {
                        Some(next) if list.named => FollowingItem::ModuleSpecifier(next.span(self)),
                        Some(next) => FollowingItem::Node(next.span(self)),
                        None => FollowingItem::End,
                    };
                    let mut prepared = self.prepare_delimited_item(
                        span,
                        following,
                        content,
                        list.breakable && self.options.trailing_commas != TrailingCommas::None,
                        writer,
                    )?;
                    if next.is_none() && last_mode == ContentMode::Verbatim {
                        incoming = last_mode;
                    }
                    if incoming == ContentMode::Verbatim {
                        prepared.set_leading_mode(incoming);
                    }
                    self.format_module_item(span, prepared_content, prepared.mode, writer)?;
                    let boundary = self.finish_delimited_item(prepared, None, writer)?;
                    end = boundary.end;
                    let next = next
                        .map(|item| {
                            self.prepare_module_item(item, list.key_quoting)
                                .map(|content| (item, content))
                        })
                        .transpose()?;
                    if let Some((next, next_content)) = next.as_ref() {
                        let constraint = self.line_constraint(end, next.span(self).start);
                        let promoted = matches!(
                            &next_content.content,
                            ModuleItemContent::Alias {
                                promoted: Some(_),
                                ..
                            }
                        );
                        let separator = if list.named
                            && promoted
                            && boundary.following_mode == ContentMode::Formatted
                            && matches!(boundary.separator, Separator::EmptyLine)
                        {
                            Separator::Line
                        } else {
                            boundary.separator
                        };
                        ArgumentLayout::Lines.separator(separator, constraint, writer);
                        incoming = boundary.following_mode;
                    }
                    current = next;
                }
                Ok::<_, FormatError>(end)
            })?;
            self.line_constraint(end, content.end).write(
                writer,
                if self.options.bracket_spacing {
                    FlatSeparator::Space
                } else {
                    FlatSeparator::Empty
                },
                |writer| {
                    if list.breakable {
                        if self.options.bracket_spacing {
                            writer.line()
                        } else {
                            writer.soft_line()
                        }
                    } else if self.options.bracket_spacing {
                        writer.space()
                    }
                },
            );
            writer.text("}");
            Ok(())
        };
        if list.breakable {
            writer.group(list.expand, write)
        } else {
            write(writer)
        }
    }

    fn prepare_module_item(
        &self,
        item: ModuleItem,
        quoting: PropertyKeyQuoting,
    ) -> Result<PreparedModuleItem<'ast>, FormatError> {
        let ast = self.ast;
        let content = match item {
            ModuleItem::Import(node) => match ast.import_declaration_specifier_data(node) {
                ImportDeclarationSpecifierData::ImportDefaultSpecifier(node) => Ok(
                    ModuleItemContent::Name(ModuleName::Binding(node.local(ast))),
                ),
                ImportDeclarationSpecifierData::ImportNamespaceSpecifier(node) => self
                    .prepare_module_namespace(node.span(ast), ModuleName::Binding(node.local(ast))),
                ImportDeclarationSpecifierData::ImportSpecifier(node) => {
                    return self.prepare_module_specifier(
                        node.span(ast),
                        ModuleName::Export(node.imported(ast)),
                        ModuleName::Binding(node.local(ast)),
                        node.import_kind(ast),
                    );
                }
            },
            ModuleItem::Export(node) => {
                return self.prepare_module_specifier(
                    node.span(ast),
                    ModuleName::Export(node.local(ast)),
                    ModuleName::Export(node.exported(ast)),
                    node.export_kind(ast),
                );
            }
            ModuleItem::Attribute(node) => {
                let key = node.key(ast);
                let value = node.value(ast);
                let gap = Span::new(key.span(ast).end, value.span(ast).start);
                let colon = self.module_operator(gap, ":")?;
                let mut modes = NameModes {
                    left: ContentMode::Formatted,
                    right: ContentMode::Formatted,
                    enclosing: ContentMode::Formatted,
                };
                if self.has_suppression {
                    let key_printed = match ast.module_export_name_data(key) {
                        ModuleExportNameData::IdentifierName(_) => true,
                        ModuleExportNameData::StringLiteral(literal) => self
                            .unquoted_string_key(
                                literal,
                                PropertyKeyContext::ModuleAttribute,
                                quoting,
                            )
                            .is_none(),
                    };
                    if key_printed
                        && self
                            .content_mode(self.comments.in_range(Span::new(gap.start, colon.start)))
                            == ContentMode::Verbatim
                    {
                        modes.left = ContentMode::Verbatim;
                    }
                    for position in self.suppression_positions(
                        self.comments.in_range(Span::new(colon.end, gap.end)),
                        gap.end,
                    ) {
                        if matches!(position, SuppressionPosition::EndOfLine) {
                            if key_printed {
                                modes.left = ContentMode::Verbatim;
                            }
                        } else {
                            modes.right = ContentMode::Verbatim;
                        }
                    }
                }
                let boundary = if modes.left == ContentMode::Formatted
                    && modes.right == ContentMode::Formatted
                {
                    Some(self.assignment_boundary(
                        gap.start,
                        &self.expression_content(Expr::StringLiteral(value)),
                        colon,
                        value.span(ast).end,
                    )?)
                } else {
                    None
                };
                Ok(ModuleItemContent::Attribute {
                    quoting,
                    key,
                    value,
                    colon,
                    modes,
                    boundary,
                })
            }
        }?;
        Ok(PreparedModuleItem {
            content,
            prefix: None,
        })
    }

    fn prepare_module_specifier(
        &self,
        owner: Span,
        left: ModuleName,
        right: ModuleName,
        kind: ImportOrExportKind,
    ) -> Result<PreparedModuleItem<'ast>, FormatError> {
        if kind == ImportOrExportKind::Type {
            return Err(FormatError::unsupported("type-only specifier", None, owner));
        }
        let span = Span::new(owner.start, left.span(self).start);
        let prefix = if span.start < span.end {
            let comments = if span.start < span.end {
                self.comments.in_range(span)
            } else {
                &[]
            };
            let promoted = !comments.is_empty()
                && !self.diagnostic_range_is_sensitive(span)
                && comments
                    .iter()
                    .all(|&comment| self.comment_semantics(comment).is_ordinary())
                && comments.iter().any(|comment| {
                    comment.kind == CommentKind::Line
                        || self.source_position_starts_line(comment.span.start)
                        || self.line_breaks(comment.span.end, span.end) != 0
                });
            let mut source = ModulePrefixSource::Formatted;
            if self.has_suppression {
                for position in self.suppression_positions(comments, span.end) {
                    if matches!(position, SuppressionPosition::Inline) {
                        source = ModulePrefixSource::Name;
                    } else {
                        source = ModulePrefixSource::Item;
                        break;
                    }
                }
            }
            Some(ModuleNamePrefix {
                span,
                comments,
                source,
                promoted,
            })
        } else {
            None
        };
        Ok(PreparedModuleItem {
            content: self.prepare_module_alias(left, right)?,
            prefix,
        })
    }

    fn prepare_module_namespace(
        &self,
        owner: Span,
        right: ModuleName,
    ) -> Result<ModuleItemContent<'ast>, FormatError> {
        let gap = Span::new(owner.start + 1, right.span(self).start);
        let operator = self.module_operator(gap, "as")?;
        let mode = if self.has_suppression {
            self.content_mode(self.comments.in_range(gap))
        } else {
            ContentMode::Formatted
        };
        Ok(ModuleItemContent::Alias {
            left: None,
            right,
            operator,
            modes: NameModes {
                left: ContentMode::Formatted,
                right: mode,
                enclosing: ContentMode::Formatted,
            },
            promoted: None,
        })
    }

    fn prepare_module_alias(
        &self,
        left: ModuleName,
        right: ModuleName,
    ) -> Result<ModuleItemContent<'ast>, FormatError> {
        let left_span = left.span(self);
        let right_span = right.span(self);
        if left_span == right_span {
            return Ok(ModuleItemContent::Name(left));
        }
        let operator = self.module_operator(Span::new(left_span.end, right_span.start), "as")?;
        let mut modes = NameModes {
            left: ContentMode::Formatted,
            right: ContentMode::Formatted,
            enclosing: ContentMode::Formatted,
        };
        if self.has_suppression {
            for position in self.suppression_positions(
                self.comments
                    .in_range(Span::new(left_span.end, operator.start)),
                operator.start,
            ) {
                if matches!(position, SuppressionPosition::Inline) {
                    modes.left = ContentMode::Verbatim;
                } else {
                    modes.enclosing = ContentMode::Verbatim;
                }
            }
            for position in self.suppression_positions(
                self.comments
                    .in_range(Span::new(operator.end, right_span.start)),
                right_span.start,
            ) {
                if matches!(position, SuppressionPosition::Inline) {
                    modes.right = ContentMode::Verbatim;
                } else {
                    modes.enclosing = ContentMode::Verbatim;
                }
            }
        }
        let gap = Span::new(left_span.end, right_span.start);
        let comments = self.comments.in_range(gap);
        let promoted = (!comments.is_empty()
            && comments
                .iter()
                .all(|&comment| self.comment_semantics(comment).is_ordinary())
            && comments.iter().any(|comment| {
                comment.kind == CommentKind::Line
                    || self.source_position_starts_line(comment.span.start)
                    || self.line_breaks(comment.span.end, right_span.start) != 0
            }))
        .then_some(gap);
        Ok(ModuleItemContent::Alias {
            left: Some(left),
            right,
            operator,
            modes,
            promoted,
        })
    }

    fn format_module_item(
        &mut self,
        owner: Span,
        item: PreparedModuleItem<'ast>,
        mode: ContentMode,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let PreparedModuleItem { content, prefix } = item;
        let enclosing = match &content {
            ModuleItemContent::Alias { modes, .. } | ModuleItemContent::Attribute { modes, .. } => {
                modes.enclosing
            }
            _ => ContentMode::Formatted,
        };
        if mode == ContentMode::Verbatim
            || enclosing == ContentMode::Verbatim
            || prefix
                .as_ref()
                .is_some_and(|prefix| prefix.source == ModulePrefixSource::Item)
        {
            return self.format_verbatim(owner, writer);
        }
        if let Some(prefix) = &prefix {
            self.comments.claim(prefix.comments)?;
            if prefix.promoted {
                self.format_comment_run(prefix.comments, CommentRunLayout::Lines, writer)?;
                writer.hard_line();
            }
        }
        if let ModuleItemContent::Alias {
            promoted: Some(gap),
            ..
        } = &content
        {
            let comments = self.comments.between(*gap)?;
            self.comments.claim(comments)?;
            self.format_comment_run(comments, CommentRunLayout::Lines, writer)?;
            writer.hard_line();
        }
        let name_mode = prefix
            .as_ref()
            .filter(|prefix| prefix.source == ModulePrefixSource::Name)
            .map_or(ContentMode::Formatted, |_| ContentMode::Verbatim);
        if let Some(prefix) = prefix
            && !prefix.promoted
            && !prefix.comments.is_empty()
        {
            self.format_comment_run(
                prefix.comments,
                CommentRunLayout::FlatLeading(prefix.span.end),
                writer,
            )?;
            let end = prefix
                .comments
                .last()
                .map_or(prefix.span.start, |comment| comment.span.end);
            let constraint = if name_mode == ContentMode::Verbatim {
                self.source_line_constraint(end, prefix.span.end)
            } else {
                self.line_constraint(end, prefix.span.end)
            };
            constraint.write(writer, FlatSeparator::Space, Writer::space);
        }
        match content {
            ModuleItemContent::Name(name) => self.format_module_name(name, name_mode, writer),
            ModuleItemContent::Alias {
                left,
                right,
                operator,
                mut modes,
                promoted: _,
            } => {
                if name_mode == ContentMode::Verbatim {
                    modes.left = name_mode;
                }
                let end = if let Some(left) = left {
                    self.format_module_name(left, modes.left, writer)?;
                    left.span(self).end
                } else {
                    writer.text("*");
                    owner.start + 1
                };
                self.format_module_gap(end, operator.start, true, writer)?;
                writer.text("as");
                self.format_module_gap(operator.end, right.span(self).start, true, writer)?;
                self.format_module_name(right, modes.right, writer)
            }
            ModuleItemContent::Attribute {
                quoting,
                key,
                value,
                colon,
                modes,
                boundary,
            } => {
                match self.ast.module_export_name_data(key) {
                    ModuleExportNameData::IdentifierName(name) => self.format_property_key(
                        PropertyKey::IdentifierName(name),
                        None,
                        modes.left,
                        quoting,
                        writer,
                    )?,
                    ModuleExportNameData::StringLiteral(literal)
                        if modes.left == ContentMode::Verbatim =>
                    {
                        self.format_verbatim(literal.span(self.ast), writer)?
                    }
                    ModuleExportNameData::StringLiteral(literal) => self
                        .format_string_property_name(
                            literal,
                            PropertyKeyContext::ModuleAttribute,
                            quoting,
                            writer,
                        ),
                }
                if let Some(boundary) = boundary {
                    let rhs_layout =
                        if boundary.rhs_requires_line(RhsLayout::NeverBreakAfterOperator) {
                            RhsLayout::BreakAfterOperator
                        } else {
                            RhsLayout::NeverBreakAfterOperator
                        };
                    let tail = self.format_operator_right(
                        key.span(self.ast).end,
                        Expr::StringLiteral(value),
                        InitializerLayout::Formatted {
                            boundary,
                            rhs_layout,
                        },
                        ExpressionContext::at(ExpressionPosition::Nested),
                        FlatSeparator::Empty,
                        writer,
                    )?;
                    return self.format_expression_tail(tail, writer);
                }
                self.format_module_gap(key.span(self.ast).end, colon.start, false, writer)?;
                writer.text(":");
                self.format_module_gap(colon.end, value.span(self.ast).start, true, writer)?;
                if modes.right == ContentMode::Verbatim {
                    self.format_verbatim(value.span(self.ast), writer)
                } else {
                    self.format_string_literal(value, StringContext::Expression, writer);
                    Ok(())
                }
            }
        }
    }

    fn format_module_name(
        &mut self,
        name: ModuleName,
        mode: ContentMode,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        if mode == ContentMode::Verbatim {
            return self.format_verbatim(name.span(self), writer);
        }
        match name {
            ModuleName::Binding(name) => writer.text(self.ast.get_utf8(name.name(self.ast))),
            ModuleName::Export(name) => match self.ast.module_export_name_data(name) {
                ModuleExportNameData::IdentifierName(name) => {
                    writer.text(self.ast.get_utf8(name.name(self.ast)))
                }
                ModuleExportNameData::StringLiteral(value) => {
                    self.format_string_literal(value, StringContext::Expression, writer)
                }
            },
        }
        Ok(())
    }

    fn format_module_comma_gap(
        &mut self,
        start: u32,
        end: u32,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let comma = self.module_operator(Span::new(start, end), ",")?;
        self.format_module_gap(start, comma.start, false, writer)?;
        writer.text(",");
        self.format_module_gap(comma.end, end, true, writer)
    }

    pub(super) fn format_module_gap(
        &mut self,
        start: u32,
        end: u32,
        space: bool,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let comments = self.comments.between(Span::new(start, end))?;
        let source = self.content_mode(comments) == ContentMode::Verbatim
            || comments
                .iter()
                .any(|comment| comment.kind == CommentKind::Line);
        if !source {
            return self.format_source_token_gap(start, end, space, writer);
        }
        self.comments.claim(comments)?;
        self.format_comment_run(comments, CommentRunLayout::SourceAfter(start), writer)?;
        let last = comments.last().map_or(start, |comment| comment.span.end);
        self.source_line_constraint(last, end).write(
            writer,
            if space {
                FlatSeparator::Space
            } else {
                FlatSeparator::Empty
            },
            |writer| {
                if comments
                    .last()
                    .is_some_and(|comment| comment.kind == CommentKind::Line)
                {
                    writer.hard_line()
                } else if space {
                    writer.space()
                }
            },
        );
        Ok(())
    }
}
