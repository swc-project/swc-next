//! Ambient prefixes retain the real keyword and the actual following declaration owner.

use swc_next_ecma_ast::{
    BindingIdentifier, Decl, DeclData, GetSpan, IdentifierName, Span, TsModuleDeclarationKind,
    TsModuleName, VariableDeclarator,
};

use super::Formatter;
use crate::formatter::comments::CommentBoundary;
use crate::formatter::diagnostics::FlatSeparator;
use crate::formatter::suppression::ContentMode;
use crate::{FormatError, document::Writer};

#[derive(Clone, Copy)]
enum DeclarationLeading {
    Name(BindingIdentifier),
    Variable(VariableDeclarator),
    ModuleName(TsModuleName),
    Global(IdentifierName),
}

impl DeclarationLeading {
    fn span(self, formatter: &Formatter<'_, '_>) -> Span {
        match self {
            Self::Name(node) => node.span(formatter.ast),
            Self::Variable(node) => node.span(formatter.ast),
            Self::ModuleName(node) => node.span(formatter.ast),
            Self::Global(node) => node.span(formatter.ast),
        }
    }
}

#[derive(Clone, Copy)]
pub(in crate::formatter) struct DeclarationKeyword {
    pub(in crate::formatter) span: Span,
    incoming: Option<DeclarationLeading>,
    name_prefix_start: u32,
}

impl DeclarationKeyword {
    pub(super) fn for_name(span: Span, name: BindingIdentifier, mode: ContentMode) -> Self {
        Self {
            span,
            name_prefix_start: span.end,
            incoming: (mode == ContentMode::Verbatim).then_some(DeclarationLeading::Name(name)),
        }
    }

    pub(in crate::formatter) fn name_prefix_start(self) -> u32 {
        self.name_prefix_start
    }

    pub(in crate::formatter) fn suppresses_name(self, name: BindingIdentifier) -> bool {
        matches!(self.incoming, Some(DeclarationLeading::Name(owner)) if owner == name)
    }

    pub(in crate::formatter) fn suppresses_module_name(self, node: TsModuleName) -> bool {
        matches!(self.incoming, Some(DeclarationLeading::ModuleName(owner)) if owner == node)
    }

    pub(in crate::formatter) fn suppresses_global(self, node: IdentifierName) -> bool {
        matches!(self.incoming, Some(DeclarationLeading::Global(owner)) if owner == node)
    }

    pub(in crate::formatter) fn suppresses_variable(self, node: VariableDeclarator) -> bool {
        matches!(self.incoming, Some(DeclarationLeading::Variable(owner)) if owner == node)
    }
}

enum DeclarationPrefix<'ast> {
    Name,
    Source(Span),
    Formatted(CommentBoundary<'ast>),
}

pub(super) struct AmbientDeclarationHead<'ast> {
    pub(super) keyword: DeclarationKeyword,
    prefix: DeclarationPrefix<'ast>,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn prepare_ambient_declaration_keyword(
        &self,
        declaration: Decl,
    ) -> Result<AmbientDeclarationHead<'ast>, FormatError> {
        let (keyword, following) = match self.ast.decl_data(declaration) {
            DeclData::TsTypeAliasDeclaration(node) => {
                ("type", DeclarationLeading::Name(node.id(self.ast)))
            }
            DeclData::TsInterfaceDeclaration(node) => {
                ("interface", DeclarationLeading::Name(node.id(self.ast)))
            }
            DeclData::VariableDeclaration(node) => {
                let Some(first) = node.declarators(self.ast).first() else {
                    return Err(self.unsupported("empty ambient variable declaration", declaration));
                };
                (
                    node.kind(self.ast).as_str(),
                    DeclarationLeading::Variable(self.ast.get_node_in_sub_range(first)),
                )
            }
            DeclData::TsModuleDeclaration(node) => (
                match node.kind(self.ast) {
                    TsModuleDeclarationKind::Namespace => "namespace",
                    TsModuleDeclarationKind::Module => "module",
                },
                DeclarationLeading::ModuleName(node.id(self.ast)),
            ),
            DeclData::TsGlobalDeclaration(node) => {
                ("global", DeclarationLeading::Global(node.id(self.ast)))
            }
            _ => return Err(self.unsupported("ambient declaration prefix", declaration)),
        };
        let start = declaration.span(self.ast).start + "declare".len() as u32;
        let following_span = following.span(self);
        let search_end = if matches!(following, DeclarationLeading::Global(_)) {
            following_span.end
        } else {
            following_span.start
        };
        let gap = Span::new(start, search_end);
        let comments = self.comments.in_range(gap);
        let keyword_start = self
            .find_token(gap, comments, keyword)
            .ok_or_else(|| self.unsupported("ambient declaration keyword", declaration))?;
        let prefix_span = Span::new(start, keyword_start);
        let prefix_comments = self.comments.in_range(prefix_span);
        let suppressed =
            self.has_suppression && self.content_mode(prefix_comments) == ContentMode::Verbatim;
        let keyword_end = keyword_start + keyword.len() as u32;
        let head_span = Span::new(start, following_span.start);
        let name_prefix = !prefix_comments.is_empty()
            && !matches!(following, DeclarationLeading::Global(_))
            && self
                .comment_binding(self.comments.in_range(head_span), head_span, None)
                .allows_relocation(None);
        let prefix = if name_prefix {
            DeclarationPrefix::Name
        } else if suppressed {
            DeclarationPrefix::Source(prefix_span)
        } else {
            DeclarationPrefix::Formatted(self.prepare_comment_boundary(
                prefix_comments,
                start,
                keyword_start,
            ))
        };
        Ok(AmbientDeclarationHead {
            keyword: DeclarationKeyword {
                span: Span::new(keyword_start, keyword_end),
                name_prefix_start: if name_prefix { start } else { keyword_end },
                incoming: suppressed.then_some(following),
            },
            prefix,
        })
    }

    pub(super) fn format_ambient_declaration_prefix(
        &mut self,
        head: &AmbientDeclarationHead<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        writer.text("declare");
        match &head.prefix {
            DeclarationPrefix::Name => {
                writer.space();
                Ok(())
            }
            DeclarationPrefix::Source(span) => self.format_verbatim(*span, writer),
            DeclarationPrefix::Formatted(boundary) => {
                self.format_comment_boundary(boundary, writer)?;
                boundary
                    .gap()
                    .write(writer, FlatSeparator::Space, Writer::space);
                Ok(())
            }
        }
    }
}
