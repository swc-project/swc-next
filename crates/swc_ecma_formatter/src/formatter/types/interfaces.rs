//! Interface heads keep their declaration, heritage, body and incoming member owners.

use swc_next_ecma_ast::{
    BindingIdentifier, Comment, GetSpan, Span, SubRange, TsInterfaceBody, TsInterfaceDeclaration,
    TsInterfaceHeritage, TsTypeElement, TsTypeParameterDeclaration,
};

use super::generics::TypeParameterContext;
use super::members::{IncomingTypeMember, TypeMemberBodyIncoming, TypeMemberBodyKind};
use super::{Formatter, TypeGap};
use crate::formatter::comments::CommentBinding;
use crate::formatter::diagnostics::FlatSeparator;
use crate::formatter::heritage::{HeritageItem, HeritageLayout, HeritageOwners, HeritagePrefix};
use crate::formatter::suppression::{ContentMode, SuppressionPosition};
use crate::{FormatError, document::Writer};

struct InterfaceParameters<'ast> {
    node: TsTypeParameterDeclaration,
    mode: ContentMode,
    prefix: TypeGap<'ast>,
}

struct InterfaceHead<'ast> {
    name: BindingIdentifier,
    name_mode: ContentMode,
    prefix: TypeGap<'ast>,
    parameters: Option<InterfaceParameters<'ast>>,
    heritage: Option<InterfaceHeritageList<'ast>>,
    body: TsInterfaceBody,
    body_mode: ContentMode,
    incoming: Option<IncomingTypeMember>,
    body_prefix: TypeGap<'ast>,
    body_leading: Option<&'ast [Comment]>,
}

struct InterfaceHeritageList<'ast> {
    first: TsInterfaceHeritage,
    remaining: SubRange<TsInterfaceHeritage>,
    first_mode: ContentMode,
    end: u32,
    prefix: HeritagePrefix<'ast>,
    layout: HeritageLayout,
}

#[derive(Clone, Copy)]
enum InterfaceBodyOwner {
    Member(TsTypeElement),
    Body,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(in crate::formatter) fn format_interface(
        &mut self,
        node: TsInterfaceDeclaration,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        if node.declare(self.ast) {
            return Err(self.unsupported("ambient interface declaration", node));
        }
        let head = self.prepare_interface(node)?;
        let InterfaceHead {
            name,
            name_mode,
            prefix,
            parameters,
            heritage,
            body,
            body_mode,
            incoming,
            body_prefix,
            body_leading,
        } = head;
        let grouped = heritage
            .as_ref()
            .is_some_and(|heritage| heritage.layout.grouped());
        writer.text("interface");
        let write_head = |formatter: &mut Self, writer: &mut Writer<'ast>| {
            formatter.format_type_gap(prefix, FlatSeparator::Space, writer)?;
            if name_mode == ContentMode::Verbatim {
                formatter.format_verbatim(name.span(formatter.ast), writer)?;
            } else {
                writer.text(formatter.ast.get_utf8(name.name(formatter.ast)));
            }
            if let Some(parameters) = parameters {
                formatter.format_type_gap(parameters.prefix, FlatSeparator::Empty, writer)?;
                formatter.format_type_parameters(
                    parameters.node,
                    TypeParameterContext::TypeDeclaration,
                    parameters.mode,
                    writer,
                )?;
            }
            if let Some(heritage) = heritage {
                if grouped {
                    writer
                        .indent(|writer| formatter.format_interface_heritage(heritage, writer))?;
                } else {
                    formatter.format_interface_heritage(heritage, writer)?;
                }
            }
            Ok(())
        };
        if grouped {
            writer.group(false, |writer| write_head(self, writer))?;
        } else {
            write_head(self, writer)?;
        }
        self.format_type_gap(body_prefix, FlatSeparator::Space, writer)?;
        let body_span = body.span(self.ast);
        if body_mode == ContentMode::Verbatim {
            self.format_verbatim(body_span, writer)
        } else {
            self.format_type_member_body(
                body_span,
                body.body(self.ast),
                TypeMemberBodyKind::Interface,
                TypeMemberBodyIncoming {
                    member: incoming,
                    comments: body_leading,
                },
                writer,
            )
        }
    }

    fn prepare_interface(
        &self,
        node: TsInterfaceDeclaration,
    ) -> Result<InterfaceHead<'ast>, FormatError> {
        let ast = self.ast;
        let name = node.id(ast);
        let name_span = name.span(ast);
        let keyword_end = node.span(ast).start + "interface".len() as u32;
        let head_gap = Span::new(keyword_end, name_span.start);
        let comments = self.comments.in_range(head_gap);
        let mut name_mode = self.content_mode(comments);
        let prefix = if name_mode == ContentMode::Verbatim {
            TypeGap::Source(head_gap)
        } else {
            TypeGap::Formatted(self.prepare_comment_boundary(
                comments,
                head_gap.start,
                head_gap.end,
            ))
        };
        let mut parameters = node.type_parameters(ast).map(|node| {
            let gap = Span::new(name_span.end, node.span(ast).start);
            let comments = self.comments.in_range(gap);
            let source =
                self.has_suppression && self.content_mode(comments) == ContentMode::Verbatim;
            let mut mode = ContentMode::Formatted;
            if source {
                for (_, position) in self.suppressed_comment_runs(comments, gap.end) {
                    if matches!(position, SuppressionPosition::EndOfLine) {
                        name_mode = ContentMode::Verbatim;
                    } else {
                        mode = ContentMode::Verbatim;
                    }
                }
            }
            InterfaceParameters {
                node,
                mode,
                prefix: if source {
                    TypeGap::Source(gap)
                } else {
                    TypeGap::Formatted(self.prepare_comment_boundary(comments, gap.start, gap.end))
                },
            }
        });
        let head_end = parameters
            .as_ref()
            .map_or(name_span.end, |parameters| parameters.node.span(ast).end);
        let body = node.body(ast);
        let mut heritage_nodes = node.extends(ast);
        let mut end = head_end;
        let heritage = if let Some(first) = heritage_nodes.first() {
            let first = ast.get_node_in_sub_range(first);
            let remaining = heritage_nodes.split_off(1);
            end = remaining.last().map_or(first.span(ast).end, |last| {
                ast.get_node_in_sub_range(last).span(ast).end
            });
            let gap = Span::new(head_end, first.span(ast).start);
            let comments = self.comments.in_range(gap);
            let keyword = self
                .find_token(gap, comments, "extends")
                .ok_or_else(|| FormatError::unsupported("interface extends boundary", None, gap))?;
            let keyword = Span::new(keyword, keyword + "extends".len() as u32);
            let prepared = self.prepare_heritage_prefix(
                gap,
                keyword,
                comments,
                || {
                    let preceding = parameters
                        .as_ref()
                        .map_or(name_span, |parameters| parameters.node.span(ast));
                    HeritageOwners {
                        inline_before: preceding,
                        line_preceding: preceding,
                    }
                },
                first.span(ast),
            );
            if !matches!(prepared.preceding, CommentBinding::Unbound) {
                if let Some(parameters) = &mut parameters {
                    parameters.mode = ContentMode::Verbatim;
                } else {
                    name_mode = ContentMode::Verbatim;
                }
            }
            let layout = if !remaining.is_empty() {
                HeritageLayout::Multiple
            } else if prepared.prefix.has_comments
                || first.type_arguments(ast).is_none()
                    && self.heritage_expression_is_member(first.expression(ast))
            {
                HeritageLayout::Single
            } else {
                HeritageLayout::Inline
            };
            Some(InterfaceHeritageList {
                first,
                remaining,
                first_mode: prepared.first_mode,
                end,
                prefix: prepared.prefix,
                layout,
            })
        } else {
            None
        };
        let body_gap = Span::new(end, body.span(ast).start);
        let mut comments = self.comments.in_range(body_gap);
        let source = self.has_suppression && self.content_mode(comments) == ContentMode::Verbatim;
        let mut body_mode = ContentMode::Formatted;
        let mut incoming = None;
        if source {
            let mut selected = None;
            for (_, position) in self.suppressed_comment_runs(comments, body_gap.end) {
                if matches!(position, SuppressionPosition::Inline) {
                    body_mode = ContentMode::Verbatim;
                } else {
                    let owner = *selected.get_or_insert_with(|| {
                        body.body(ast)
                            .first()
                            .map_or(InterfaceBodyOwner::Body, |first| {
                                InterfaceBodyOwner::Member(ast.get_node_in_sub_range(first))
                            })
                    });
                    match owner {
                        InterfaceBodyOwner::Member(node) => {
                            incoming = Some(IncomingTypeMember {
                                node,
                                mode: ContentMode::Verbatim,
                            });
                        }
                        InterfaceBodyOwner::Body => body_mode = ContentMode::Verbatim,
                    }
                }
            }
        }
        let mut body_leading = None;
        if self.declaration_body_comments_can_move(comments, body_gap) {
            let first = body
                .body(ast)
                .first()
                .map(|id| ast.get_node_in_sub_range(id));
            let leading_end = first.map_or(body.span(ast).end - 1, |node: TsTypeElement| {
                node.span(ast).start
            });
            let leading = self
                .comments
                .in_range(Span::new(comments[0].span.start, leading_end));
            if first.is_some() || self.content_mode(leading) == ContentMode::Formatted {
                body_leading = Some(leading);
                comments = &[];
            }
        }
        Ok(InterfaceHead {
            name,
            name_mode,
            prefix,
            parameters,
            heritage,
            body,
            body_mode,
            incoming,
            body_leading,
            body_prefix: if source {
                TypeGap::Source(body_gap)
            } else {
                TypeGap::Formatted(self.prepare_comment_boundary(
                    comments,
                    body_gap.start,
                    body_gap.end,
                ))
            },
        })
    }

    fn format_interface_heritage(
        &mut self,
        list: InterfaceHeritageList<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
        let items = std::iter::once(list.first)
            .chain(
                list.remaining
                    .iter()
                    .map(|id| ast.get_node_in_sub_range(id)),
            )
            .map(|node| HeritageItem {
                span: node.span(ast),
                expression: node.expression(ast),
                arguments: node.type_arguments(ast),
            });
        self.format_heritage_clause(
            list.prefix,
            "extends",
            list.layout,
            writer,
            |formatter, writer| {
                formatter.format_heritage_items(items, list.first_mode, list.end, writer)
            },
        )
    }
}
