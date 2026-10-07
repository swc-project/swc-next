//! Real type members share a body while retaining their key, signature, and tail owners.

use swc_next_ecma_ast::{
    Comment, CommentKind, FormalParameters, GetSpan, PropertyKey, Span, SubRange,
    TsCallSignatureDeclaration, TsConstructSignatureDeclaration, TsIndexSignature,
    TsIndexSignatureParameter, TsMethodSignature, TsMethodSignatureKind, TsPropertySignature,
    TsThisParameter, TsTypeAnnotation, TsTypeElement, TsTypeElementData, TsTypeLiteral,
    TsTypeParameterDeclaration,
};

use super::callable::{
    AnnotationContent, CallableParameters, SignatureGenerics, SignatureIncoming, SignatureReturn,
    SignatureReturnDelimiter, SignatureReturnEolTarget,
};
use super::{Formatter, PreparedType, TypeBoundaryLayout, TypeContext, TypeGap, TypeLayout};
use crate::formatter::comments::{CommentBoundary, CommentGap, CommentRunLayout, ExpressionTail};
use crate::formatter::diagnostics::{FlatSeparator, LineConstraint};
use crate::formatter::expressions::arguments::{ArgumentLayout, FollowingItem, Separator};
use crate::formatter::expressions::objects::{PropertyKeyContext, PropertyKeyQuoting};
use crate::formatter::suppression::{
    ContentMode, DelimitedFollowing, DelimitedSuppression, SuppressionPosition,
};
use crate::{FormatError, QuoteProperties, Semicolons, document::Writer};

struct MemberKey<'ast> {
    node: PropertyKey,
    computed: Option<Span>,
    end: u32,
    mode: ContentMode,
    prefix: Option<(&'static str, TypeGap<'ast>)>,
    leading: Option<TypeMemberLeading<'ast>>,
}

enum TypeMemberLeading<'ast> {
    Source(Span),
    Formatted {
        comments: &'ast [Comment],
        gap: CommentGap,
    },
}

struct PropertyMember<'ast> {
    key: MemberKey<'ast>,
    optional: bool,
    annotation: Option<(PreparedType, TypeBoundaryLayout<'ast>)>,
    ending: Option<TypeGap<'ast>>,
    riding: Option<ExpressionTail<'ast>>,
}

#[derive(Clone, Copy)]
enum MemberSignatureOwner {
    Method(TsMethodSignature),
    Call(TsCallSignatureDeclaration),
    Construct(TsConstructSignatureDeclaration),
}

struct MemberSignatureParts {
    parameters: FormalParameters,
    receiver: Option<TsThisParameter>,
    generics: Option<TsTypeParameterDeclaration>,
    result: Option<TsTypeAnnotation>,
}

impl MemberSignatureOwner {
    fn parts(self, ast: &swc_next_ecma_ast::Ast<'_>) -> MemberSignatureParts {
        match self {
            Self::Method(node) => MemberSignatureParts {
                parameters: node.params(ast),
                receiver: node.this_param(ast),
                generics: node.type_parameters(ast),
                result: node.return_type(ast),
            },
            Self::Call(node) => MemberSignatureParts {
                parameters: node.params(ast),
                receiver: node.this_param(ast),
                generics: node.type_parameters(ast),
                result: node.return_type(ast),
            },
            Self::Construct(node) => MemberSignatureParts {
                parameters: node.params(ast),
                receiver: None,
                generics: node.type_parameters(ast),
                result: node.return_type(ast),
            },
        }
    }
}

struct SignatureMember<'ast> {
    key: Option<MemberKey<'ast>>,
    construct: bool,
    optional: bool,
    optional_before: Option<TypeGap<'ast>>,
    prefix: TypeGap<'ast>,
    generics: Option<(TsTypeParameterDeclaration, ContentMode, TypeGap<'ast>)>,
    parameters: CallableParameters,
    result: Option<SignatureReturn<'ast>>,
}

pub(super) struct PreparedIndexSignature<'ast> {
    parameters: SubRange<TsIndexSignatureParameter>,
    content_end: u32,
    opening: u32,
    closing: u32,
    prefix: IndexSignaturePrefix<'ast>,
    leading: Option<TypeMemberLeading<'ast>>,
    closing_gap: TypeGap<'ast>,
    source_head: Option<TsIndexSignatureParameter>,
    source_tail: Option<TsIndexSignatureParameter>,
    result: SignatureReturn<'ast>,
}

enum IndexSignaturePrefix<'ast> {
    Bare,
    Readonly(TypeGap<'ast>),
}

impl PreparedIndexSignature<'_> {
    pub(super) fn content_end(&self) -> u32 {
        self.content_end
    }

    pub(super) fn protect_result_type(&mut self) {
        if let AnnotationContent::Type(node) = &mut self.result.content {
            node.source_owner = Some(node.node);
        }
    }
}

enum MemberContent<'ast> {
    Source(Span),
    Property(PropertyMember<'ast>),
    Signature(SignatureMember<'ast>),
    Index(PreparedIndexSignature<'ast>),
}

#[derive(Clone, Copy)]
struct MemberHandle {
    node: TsTypeElement,
    data: TsTypeElementData,
}

impl MemberHandle {
    fn span(self, ast: &swc_next_ecma_ast::Ast<'_>) -> Span {
        self.node.span(ast)
    }
}

#[derive(Clone, Copy)]
pub(super) enum TypeMemberBodyKind {
    Literal,
    Interface,
}

#[derive(Clone, Copy)]
pub(super) struct IncomingTypeMember {
    pub(super) node: TsTypeElement,
    pub(super) mode: ContentMode,
}

#[derive(Default)]
pub(super) struct TypeMemberBodyIncoming<'ast> {
    pub(super) member: Option<IncomingTypeMember>,
    pub(super) comments: Option<&'ast [Comment]>,
}

#[derive(Clone, Copy)]
enum MemberGapRole {
    Between(Option<Span>),
    Closing,
}

struct PreparedMember<'ast> {
    content_end: u32,
    separator: Option<Span>,
    content: MemberContent<'ast>,
    before_separator: Option<TypeGap<'ast>>,
    after_separator: Option<ExpressionTail<'ast>>,
}

enum MemberGap<'ast> {
    Source(Span),
    Formatted {
        trailing: CommentBoundary<'ast>,
        leading: &'ast [Comment],
        next_start: u32,
    },
}

enum MemberClosing {
    Source(Span),
    Formatted(CommentGap),
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn type_body_expands(&self, opening_end: u32, first_start: u32) -> bool {
        self.line_breaks(opening_end, first_start) != 0
    }

    pub(super) fn format_type_literal(
        &mut self,
        node: TsTypeLiteral,
        layout: TypeLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_type_member_body_with_layout(
            node.span(self.ast),
            node.members(self.ast),
            TypeMemberBodyKind::Literal,
            TypeMemberBodyIncoming::default(),
            layout,
            writer,
        )?;
        Ok(None)
    }

    pub(super) fn format_type_member_body(
        &mut self,
        span: Span,
        members: SubRange<TsTypeElement>,
        kind: TypeMemberBodyKind,
        incoming: TypeMemberBodyIncoming<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        self.format_type_member_body_with_layout(
            span,
            members,
            kind,
            incoming,
            TypeLayout::Root,
            writer,
        )
    }

    fn format_type_member_body_with_layout(
        &mut self,
        span: Span,
        members: SubRange<TsTypeElement>,
        kind: TypeMemberBodyKind,
        incoming: TypeMemberBodyIncoming<'ast>,
        layout: TypeLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
        let TypeMemberBodyIncoming {
            member: incoming,
            comments: head_comments,
        } = incoming;
        let content = Span::new(span.start + 1, span.end - 1);
        let quoting = match self.options.quote_properties {
            QuoteProperties::AsNeeded => PropertyKeyQuoting::AsNeeded,
            QuoteProperties::Preserve => PropertyKeyQuoting::Preserve,
        };
        let mut members = members
            .iter()
            .map(|id| {
                let node = ast.get_node_in_sub_range(id);
                MemberHandle {
                    node,
                    data: ast.ts_type_element_data(node),
                }
            })
            .peekable();
        let Some(first) = members.peek().copied() else {
            let comments = head_comments.unwrap_or_else(|| self.comments.in_range(content));
            if self.content_mode(comments) == ContentMode::Verbatim {
                return self.format_verbatim(span, writer);
            }
            if comments.is_empty() {
                writer.text("{}");
                return Ok(());
            }
            let opening_end =
                head_comments.map_or(content.start, |comments| comments[0].span.start);
            let boundary = self.prepare_comment_boundary(comments, opening_end, content.end);
            let can_relocate = self
                .comment_binding(comments, content, None)
                .allows_relocation(None);
            let comment_layout = if can_relocate {
                CommentRunLayout::CanonicalDangling
            } else {
                CommentRunLayout::Source
            };
            let expanded = matches!(kind, TypeMemberBodyKind::Interface)
                || comments
                    .iter()
                    .any(|comment| comment.kind == CommentKind::Line);
            let mut write = |writer: &mut Writer<'ast>| {
                writer.text("{");
                writer.indent(|writer| {
                    self.line_constraint(opening_end, comments[0].span.start)
                        .write(writer, FlatSeparator::Empty, |writer| {
                            if expanded {
                                writer.hard_line();
                            } else {
                                writer.soft_line();
                            }
                        });
                    self.format_comment_boundary_with_layout(&boundary, comment_layout, writer)
                })?;
                boundary
                    .gap()
                    .write(writer, FlatSeparator::Empty, |writer| {
                        if expanded {
                            writer.hard_line();
                        } else {
                            writer.soft_line();
                        }
                    });
                writer.text("}");
                Ok(())
            };
            return if layout == TypeLayout::SignatureParameter {
                write(writer)
            } else {
                writer.group(expanded, write)
            };
        };
        let first_start = first.span(ast).start;
        let leading_span = Span::new(content.start, first_start);
        let leading_comments =
            head_comments.unwrap_or_else(|| self.comments.in_range(leading_span));
        let mut mode = incoming
            .filter(|member| member.node == first.node)
            .map_or_else(|| self.content_mode(leading_comments), |member| member.mode);
        let opening_start = head_comments.map_or(content.start, |comments| comments[0].span.start);
        let expanded = matches!(kind, TypeMemberBodyKind::Interface)
            || self.type_body_expands(content.start, first_start);
        let mut write = |writer: &mut Writer<'ast>| {
            writer.text("{");
            let closing = writer.indent(|writer| {
                self.format_member_opening(
                    leading_comments,
                    opening_start,
                    first_start,
                    kind,
                    writer,
                )?;
                while let Some(node) = members.next() {
                    let span = node.span(ast);
                    let next = members.peek().copied();
                    let next_start = next.map_or(content.end, |node| node.span(ast).start);
                    let suppression = self.delimited_suppression(
                        span,
                        next_start,
                        if next.is_some() {
                            DelimitedFollowing::Node
                        } else {
                            DelimitedFollowing::End
                        },
                    );
                    if suppression
                        .as_ref()
                        .is_some_and(|source| source.preceding.is_some())
                    {
                        mode = ContentMode::Verbatim;
                    }
                    let mut prepared = self.prepare_type_member(node, mode)?;
                    let source = matches!(prepared.content, MemberContent::Source(_));
                    let gap_span = Span::new(prepared.content_end, next_start);
                    let role = if next.is_some() {
                        MemberGapRole::Between(prepared.separator)
                    } else {
                        MemberGapRole::Closing
                    };
                    // An ignored member without a source separator relies on the
                    // original newline. Adding a separator would change its raw owner.
                    let gap = if source && prepared.separator.is_none() && next.is_some() {
                        MemberGap::Source(gap_span)
                    } else {
                        self.prepare_member_gap(gap_span, role)
                    };
                    let source_gap = matches!(gap, MemberGap::Source(_));
                    // The prepared key can remain verbatim even when its member is
                    // formatted. Its actual printed kind determines contextual tokens;
                    // diagnostic-covered members can also require a same-line separator.
                    let required_separator = self.options.semicolons == Semicolons::AsNeeded
                        && (self.type_member_requires_separator(&prepared.content, next, quoting)
                            || next.is_some()
                                && self.line_constraint(gap_span.start, gap_span.end)
                                    == LineConstraint::SameLine);
                    let after_separator = prepared.after_separator.take();
                    let tail = self.format_type_member(prepared, quoting, writer)?;
                    self.format_expression_tail(tail, writer)?;
                    if !source && !source_gap {
                        if required_separator
                            || next.is_some() && self.options.semicolons == Semicolons::Always
                        {
                            writer.text(";");
                        } else if self.options.semicolons == Semicolons::Always {
                            writer.if_break(";");
                        } else if next.is_some() {
                            writer.if_flat(";");
                        }
                    }
                    self.format_expression_tail(after_separator, writer)?;
                    if next.is_none() {
                        return match gap {
                            MemberGap::Source(span) => Ok(MemberClosing::Source(span)),
                            MemberGap::Formatted { trailing, .. } => {
                                self.format_comment_boundary(&trailing, writer)?;
                                Ok(MemberClosing::Formatted(trailing.gap()))
                            }
                        };
                    }
                    self.format_member_gap(gap, role, kind, writer)?;
                    mode = suppression.map_or(ContentMode::Formatted, |source| source.following);
                }
                unreachable!("a nonempty type body retains its final member")
            })?;
            match closing {
                MemberClosing::Source(span) => self.format_verbatim(span, writer)?,
                MemberClosing::Formatted(gap) => gap.write(
                    writer,
                    if self.options.bracket_spacing {
                        FlatSeparator::Space
                    } else {
                        FlatSeparator::Empty
                    },
                    |writer| {
                        if matches!(kind, TypeMemberBodyKind::Interface) {
                            writer.hard_line();
                        } else if self.options.bracket_spacing {
                            writer.line();
                        } else {
                            writer.soft_line();
                        }
                    },
                ),
            }
            writer.text("}");
            Ok(())
        };
        if layout == TypeLayout::SignatureParameter {
            write(writer)
        } else {
            writer.group(expanded, write)
        }
    }

    fn format_member_opening(
        &mut self,
        comments: &'ast [Comment],
        start: u32,
        next_start: u32,
        kind: TypeMemberBodyKind,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let spaced = self.options.bracket_spacing;
        let leading_start = comments
            .first()
            .map_or(next_start, |comment| comment.span.start);
        let flat = if spaced {
            FlatSeparator::Space
        } else {
            FlatSeparator::Empty
        };
        self.line_constraint(start, leading_start)
            .write(writer, flat, |writer| {
                if matches!(kind, TypeMemberBodyKind::Interface) {
                    writer.hard_line();
                } else if spaced {
                    writer.line();
                } else {
                    writer.soft_line();
                }
            });
        if let Some(last) = comments.last() {
            self.comments.claim(comments)?;
            self.format_comment_run(comments, CommentRunLayout::Source, writer)?;
            self.line_constraint(last.span.end, next_start).write(
                writer,
                FlatSeparator::Space,
                |writer| self.comment_gap(last.span.end, next_start, writer),
            );
        }
        Ok(())
    }

    fn prepare_member_gap(&self, gap: Span, role: MemberGapRole) -> MemberGap<'ast> {
        let comments = self.comments.in_range(gap);
        if self.content_mode(comments) == ContentMode::Verbatim {
            return MemberGap::Source(gap);
        }
        let (trailing, leading) = match role {
            MemberGapRole::Between(separator) => {
                let before_separator = separator.map_or(0, |separator| {
                    comments.partition_point(|comment| comment.span.end <= separator.start)
                });
                self.split_separator_comments(comments, before_separator, gap.start, Some(gap.end))
            }
            MemberGapRole::Closing => (comments, &[][..]),
        };
        let leading_start = leading
            .first()
            .map_or(gap.end, |comment| comment.span.start);
        MemberGap::Formatted {
            trailing: self.prepare_comment_boundary(trailing, gap.start, leading_start),
            leading,
            next_start: gap.end,
        }
    }

    fn format_member_gap(
        &mut self,
        gap: MemberGap<'ast>,
        role: MemberGapRole,
        kind: TypeMemberBodyKind,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let spaced = self.options.bracket_spacing || matches!(role, MemberGapRole::Between(_));
        let flat = if spaced {
            FlatSeparator::Space
        } else {
            FlatSeparator::Empty
        };
        match gap {
            MemberGap::Source(span) => self.format_verbatim(span, writer),
            MemberGap::Formatted {
                trailing,
                leading,
                next_start,
            } => {
                self.format_comment_boundary(&trailing, writer)?;
                let leading_start = leading
                    .first()
                    .map_or(next_start, |comment| comment.span.start);
                if matches!(role, MemberGapRole::Between(_))
                    && self.next_line_is_empty(trailing.end(), leading_start)
                {
                    self.line_constraint(trailing.end(), leading_start).write(
                        writer,
                        flat,
                        |writer| {
                            writer.hard_line();
                            writer.hard_line();
                        },
                    );
                } else {
                    trailing.gap().write(writer, flat, |writer| {
                        if matches!(kind, TypeMemberBodyKind::Interface) {
                            writer.hard_line();
                        } else if spaced {
                            writer.line();
                        } else {
                            writer.soft_line();
                        }
                    });
                }
                if let Some(last) = leading.last() {
                    self.comments.claim(leading)?;
                    self.format_comment_run(leading, CommentRunLayout::Source, writer)?;
                    self.line_constraint(last.span.end, next_start).write(
                        writer,
                        FlatSeparator::Space,
                        |writer| self.comment_gap(last.span.end, next_start, writer),
                    );
                }
                Ok(())
            }
        }
    }

    fn type_member_requires_separator(
        &self,
        content: &MemberContent<'ast>,
        next: Option<MemberHandle>,
        quoting: PropertyKeyQuoting,
    ) -> bool {
        let MemberContent::Property(property) = content else {
            return false;
        };
        if property.annotation.is_some() || property.key.computed.is_some() {
            return false;
        }
        if next.is_some_and(|next| {
            matches!(next.data, TsTypeElementData::TsCallSignatureDeclaration(_))
        }) {
            return true;
        }
        self.property_key_identifier(
            property.key.node,
            PropertyKeyContext::TypeProperty,
            property.key.mode,
            quoting,
        )
        .is_some_and(|name| matches!(name, "get" | "set" | "static"))
    }

    fn prepare_type_member(
        &self,
        handle: MemberHandle,
        mode: ContentMode,
    ) -> Result<PreparedMember<'ast>, FormatError> {
        let ast = self.ast;
        let node = handle.node;
        let span = node.span(ast);
        let separator = self
            .source(span)
            .as_bytes()
            .last()
            .filter(|&&byte| matches!(byte, b';' | b','))
            .map(|_| Span::new(span.end - 1, span.end));
        if mode == ContentMode::Verbatim {
            return Ok(PreparedMember {
                content_end: span.end,
                separator,
                content: MemberContent::Source(span),
                before_separator: None,
                after_separator: None,
            });
        }
        let data = handle.data;
        let (mut content, content_end) = match data {
            TsTypeElementData::TsPropertySignature(property) => {
                let key = self.prepare_member_key(
                    property.key(ast),
                    property.computed(ast),
                    span,
                    property
                        .type_annotation(ast)
                        .map_or(span.end, |annotation| annotation.span(ast).start),
                    property.readonly(ast).then_some("readonly"),
                )?;
                let end = if let Some(annotation) = property.type_annotation(ast) {
                    annotation.span(ast).end
                } else if property.optional(ast) {
                    let gap = Span::new(key.end, span.end);
                    let Some(optional) = self.find_token(gap, self.comments.in_range(gap), "?")
                    else {
                        return Err(self.unsupported("type property optional marker", property));
                    };
                    optional + 1
                } else {
                    key.end
                };
                let member = self.prepare_property_member(property, key, end)?;
                (MemberContent::Property(member), end)
            }
            TsTypeElementData::TsMethodSignature(method) => {
                self.prepare_signature_member(node, MemberSignatureOwner::Method(method))?
            }
            TsTypeElementData::TsCallSignatureDeclaration(call) => {
                self.prepare_signature_member(node, MemberSignatureOwner::Call(call))?
            }
            TsTypeElementData::TsConstructSignatureDeclaration(construct) => {
                self.prepare_signature_member(node, MemberSignatureOwner::Construct(construct))?
            }
            TsTypeElementData::TsIndexSignature(index) => {
                if index.r#static(ast) {
                    return Err(self.unsupported("static index signature", index));
                }
                let index = self.prepare_index_signature(index)?;
                let end = index.content_end();
                (MemberContent::Index(index), end)
            }
        };
        let before_separator = separator.and_then(|separator| {
            if !matches!(&content, MemberContent::Signature(signature) if signature.result.is_some()) {
                return None;
            }
            let gap = Span::new(content_end, separator.start);
            let comments = self.comments.in_range(gap);
            let bound = !self.comment_binding(comments, gap, None).allows_relocation(None);
            (!comments.is_empty() && (bound || comments.iter().all(|comment| comment.kind == CommentKind::Block)))
                .then(|| self.prepare_type_gap(gap, bound))
        });
        let after_separator = match &mut content {
            MemberContent::Property(property) => property.riding.take(),
            _ => None,
        };
        let mut prepared = PreparedMember {
            content_end: if before_separator.is_some() {
                separator.map_or(content_end, |separator| separator.start)
            } else {
                content_end
            },
            separator,
            content,
            before_separator,
            after_separator,
        };
        if !self.has_suppression {
            return Ok(prepared);
        }
        let trailing = self.comments.in_range(Span::new(content_end, span.end));
        if self.content_mode(trailing) == ContentMode::Verbatim {
            match &mut prepared.content {
                MemberContent::Property(property) => match &mut property.annotation {
                    Some((node, _)) => node.source_owner = Some(node.node),
                    None => prepared.content = MemberContent::Source(span),
                },
                MemberContent::Signature(signature) => {
                    match signature.result.as_mut().map(|result| &mut result.content) {
                        Some(AnnotationContent::Type(node)) => node.source_owner = Some(node.node),
                        Some(AnnotationContent::Wrapper(_)) => {}
                        None => prepared.content = MemberContent::Source(span),
                    }
                }
                MemberContent::Index(index) => index.protect_result_type(),
                MemberContent::Source(_) => {}
            }
            if matches!(prepared.content, MemberContent::Source(_)) {
                prepared.content_end = span.end;
            }
        }
        Ok(prepared)
    }

    fn prepare_member_key(
        &self,
        node: PropertyKey,
        computed: bool,
        span: Span,
        following_start: u32,
        keyword: Option<&'static str>,
    ) -> Result<MemberKey<'ast>, FormatError> {
        let key = node.span(self.ast);
        let computed = if computed {
            let before = Span::new(span.start, key.start);
            let after = Span::new(key.end, following_start);
            let Some(opening) = self.find_token(before, self.comments.in_range(before), "[") else {
                return Err(self.unsupported("type member computed opening", node));
            };
            let Some(closing) = self.find_token(after, self.comments.in_range(after), "]") else {
                return Err(self.unsupported("type member computed closing", node));
            };
            Some(Span::new(opening, closing + 1))
        } else {
            None
        };
        let start = computed.map_or(key.start, |span| span.start);
        let keyword = keyword
            .map(|keyword| {
                let search = Span::new(span.start, start);
                let position = self
                    .find_token(search, self.comments.in_range(search), keyword)
                    .ok_or_else(|| self.unsupported("type member keyword boundary", node))?;
                Ok::<_, FormatError>((
                    keyword,
                    Span::new(position, position + keyword.len() as u32),
                ))
            })
            .transpose()?;
        let leading_end = keyword.map_or(start, |(_, span)| span.start);
        let leading = self.prepare_type_member_leading(Span::new(span.start, leading_end));
        let prefix = keyword.map(|(keyword, token)| {
            let gap = Span::new(token.end, start);
            (
                keyword,
                self.prepare_type_gap(
                    gap,
                    self.has_suppression
                        && self.content_mode(self.comments.in_range(gap)) == ContentMode::Verbatim,
                ),
            )
        });
        let source_prefix = matches!(leading, Some(TypeMemberLeading::Source(_)))
            || matches!(prefix, Some((_, TypeGap::Source(_))));
        let mode = if source_prefix {
            ContentMode::Verbatim
        } else {
            ContentMode::Formatted
        };
        Ok(MemberKey {
            node,
            computed,
            end: computed.map_or(key.end, |span| span.end),
            mode,
            prefix,
            leading,
        })
    }

    fn prepare_property_member(
        &self,
        node: TsPropertySignature,
        mut key: MemberKey<'ast>,
        content_end: u32,
    ) -> Result<PropertyMember<'ast>, FormatError> {
        let ast = self.ast;
        let mut riding = None;
        let annotation = node
            .type_annotation(ast)
            .map(|annotation| {
                let mut type_node =
                    self.prepare_type(annotation.type_annotation(ast), ContentMode::Formatted);
                let gap = Span::new(key.end, type_node.node.span(ast).start);
                let comments = self.comments.in_range(gap);
                let Some(colon) = self.find_token(gap, comments, ":") else {
                    return Err(self.unsupported("type property colon", node));
                };
                let (before, after) = comments
                    .split_at(comments.partition_point(|comment| comment.span.end <= colon));
                if self.has_suppression {
                    let before = if key.computed.is_some() {
                        self.comments
                            .in_range(Span::new(key.node.span(ast).end, colon))
                    } else {
                        before
                    };
                    for (_, position) in self.suppressed_comment_runs(before, colon) {
                        if matches!(position, SuppressionPosition::OwnLine) {
                            self.select_type_head(&mut type_node, position);
                        } else {
                            key.mode = ContentMode::Verbatim;
                        }
                    }
                    self.select_type_head_comments(&mut type_node, after, gap.end);
                }
                let relocate = key.mode == ContentMode::Formatted
                    && type_node.source_owner.is_none()
                    && after.last().is_some_and(|comment| {
                        comment.kind == CommentKind::Line
                            && self.line_breaks(colon + 1, comment.span.start) == 0
                    })
                    && self.comments.in_range(type_node.node.span(ast)).is_empty()
                    && !matches!(
                        type_node.data,
                        swc_next_ecma_ast::TsTypeData::TsUnionType(_)
                            | swc_next_ecma_ast::TsTypeData::TsIntersectionType(_)
                            | swc_next_ecma_ast::TsTypeData::TsParenthesizedType(_)
                    )
                    && self
                        .comment_binding(after, Span::new(colon + 1, gap.end), None)
                        .allows_relocation(None);
                let after = if relocate {
                    riding = Some(ExpressionTail {
                        source_end: colon + 1,
                        comments: after,
                    });
                    &[][..]
                } else {
                    after
                };
                let prefix =
                    if key.mode == ContentMode::Verbatim || type_node.source_owner.is_some() {
                        TypeBoundaryLayout::Source(gap)
                    } else {
                        TypeBoundaryLayout::Formatted {
                            before: self.prepare_comment_boundary(before, gap.start, colon),
                            after: self.prepare_comment_boundary(after, colon + 1, gap.end),
                        }
                    };
                Ok((type_node, prefix))
            })
            .transpose()?;
        let ending = if annotation.is_none() {
            let gap = Span::new(key.end, content_end - u32::from(node.optional(ast)));
            let source = self.has_suppression
                && self.content_mode(self.comments.in_range(gap)) == ContentMode::Verbatim;
            if source {
                key.mode = ContentMode::Verbatim;
            }
            if self.has_suppression
                && let Some(computed) = key.computed
            {
                let comments = self
                    .comments
                    .in_range(Span::new(key.node.span(ast).end, computed.end - 1));
                if self.content_mode(comments) == ContentMode::Verbatim {
                    key.mode = ContentMode::Verbatim;
                }
            }
            Some(self.prepare_type_gap(gap, source))
        } else {
            None
        };
        Ok(PropertyMember {
            key,
            optional: node.optional(ast),
            annotation,
            ending,
            riding,
        })
    }

    fn prepare_signature_member(
        &self,
        node: TsTypeElement,
        owner: MemberSignatureOwner,
    ) -> Result<(MemberContent<'ast>, u32), FormatError> {
        let ast = self.ast;
        let span = node.span(ast);
        let parts = owner.parts(ast);
        let parameter_span = parts.parameters.span(ast);
        if self.has_suppression
            && parts.receiver.is_none()
            && parts.parameters.items(ast).is_empty()
            && parts.parameters.rest(ast).is_none()
            && self.content_mode(
                self.comments
                    .in_range(Span::new(parameter_span.start + 1, parameter_span.end - 1)),
            ) == ContentMode::Verbatim
        {
            return Ok((MemberContent::Source(span), span.end));
        }
        let head_end = parts
            .generics
            .map_or(parameter_span.start, |node| node.span(ast).start);
        let mut key = match owner {
            MemberSignatureOwner::Method(method) => {
                let keyword = match method.kind(ast) {
                    TsMethodSignatureKind::Method => None,
                    TsMethodSignatureKind::Get => Some("get"),
                    TsMethodSignatureKind::Set => Some("set"),
                };
                Some(self.prepare_member_key(
                    method.key(ast),
                    method.computed(ast),
                    span,
                    head_end,
                    keyword,
                )?)
            }
            _ => None,
        };
        let optional =
            matches!(owner, MemberSignatureOwner::Method(method) if method.optional(ast));
        let construct = matches!(owner, MemberSignatureOwner::Construct(_));
        let prefix_end = key
            .as_ref()
            .map_or(span.start + if construct { 3 } else { 0 }, |key| key.end);
        let head_gap = Span::new(prefix_end, head_end);
        let mut generic_mode = ContentMode::Formatted;
        let mut parameters = self.callable_parameters(parts.parameters, parts.receiver);
        let mut source_return = None;
        let mut incoming = None;
        let head_source = self.has_suppression
            && self.content_mode(self.comments.in_range(head_gap)) == ContentMode::Verbatim
            || !self
                .comment_binding(self.comments.in_range(head_gap), head_gap, None)
                .allows_relocation(None);
        if head_source {
            for (_, position) in
                self.suppressed_comment_runs(self.comments.in_range(head_gap), head_end)
            {
                if key.is_some() && matches!(position, SuppressionPosition::EndOfLine) {
                    if let Some(key) = &mut key {
                        key.mode = ContentMode::Verbatim;
                    }
                } else if parts.generics.is_some() {
                    generic_mode = ContentMode::Verbatim;
                } else {
                    match self.signature_incoming(&parameters, parts.result, &mut incoming)? {
                        SignatureIncoming::Parameter(first) => parameters.source_head = Some(first),
                        SignatureIncoming::Return(annotation) => source_return = Some(annotation),
                        SignatureIncoming::Empty => {
                            if let Some(key) = &mut key {
                                key.mode = ContentMode::Verbatim;
                            } else {
                                return Ok((MemberContent::Source(span), span.end));
                            }
                        }
                    }
                }
            }
        }
        let mut optional_before = None;
        let mut prefix_gap = head_gap;
        if optional && !head_source {
            let comments = self.comments.in_range(head_gap);
            let marker = self
                .find_token(head_gap, comments, "?")
                .ok_or_else(|| self.unsupported("type method optional marker", node))?;
            let after = Span::new(marker + 1, head_gap.end);
            let relocate = head_end == parameter_span.start
                && self.prepare_incoming_parameter_head(&mut parameters, after)?;
            if !parameters.has_parameters() && head_end == parameter_span.start {
                optional_before = Some(self.prepare_type_gap(head_gap, false));
                prefix_gap = Span::new(head_gap.end, head_gap.end);
            } else {
                optional_before =
                    Some(self.prepare_type_gap(Span::new(head_gap.start, marker), false));
                prefix_gap = if relocate {
                    Span::new(after.end, after.end)
                } else {
                    after
                };
            }
        } else if construct
            && !head_source
            && head_end == parameter_span.start
            && self.prepare_incoming_parameter_head(&mut parameters, head_gap)?
        {
            prefix_gap = Span::new(head_gap.end, head_gap.end);
        }
        let prefix = self.prepare_type_gap(prefix_gap, head_source);
        let mut generics = parts
            .generics
            .map(|generic| {
                let gap = Span::new(generic.span(ast).end, parameter_span.start);
                let comments = self.comments.in_range(gap);
                let source =
                    self.has_suppression && self.content_mode(comments) == ContentMode::Verbatim;
                if source {
                    for (_, position) in self.suppressed_comment_runs(comments, gap.end) {
                        if matches!(position, SuppressionPosition::EndOfLine) {
                            generic_mode = ContentMode::Verbatim;
                        } else {
                            match self.signature_incoming(
                                &parameters,
                                parts.result,
                                &mut incoming,
                            )? {
                                SignatureIncoming::Parameter(first) => {
                                    parameters.source_head = Some(first)
                                }
                                SignatureIncoming::Return(annotation) => {
                                    source_return = Some(annotation)
                                }
                                SignatureIncoming::Empty => generic_mode = ContentMode::Verbatim,
                            }
                        }
                    }
                }
                Ok((generic, generic_mode, self.prepare_type_gap(gap, source)))
            })
            .transpose()?;
        let result = parts.result.map(|annotation| {
            let eol_target = if parameters.has_parameters() {
                SignatureReturnEolTarget::Parameter
            } else if let Some((_, mode, _)) = &mut generics {
                SignatureReturnEolTarget::Header(mode)
            } else if let Some(key) = &mut key {
                SignatureReturnEolTarget::Header(&mut key.mode)
            } else {
                SignatureReturnEolTarget::Annotation
            };
            self.prepare_signature_return(
                parameter_span.end,
                eol_target,
                annotation,
                source_return,
                SignatureReturnDelimiter::Colon,
            )
        });
        self.prepare_callable_parameter_tail(
            &mut parameters,
            result
                .as_ref()
                .map_or(ContentMode::Formatted, |result| result.preceding_mode),
        )?;
        let content_end = parts
            .result
            .map_or(parameter_span.end, |annotation| annotation.span(ast).end);
        Ok((
            MemberContent::Signature(SignatureMember {
                key,
                construct,
                optional,
                optional_before,
                prefix,
                generics,
                parameters,
                result,
            }),
            content_end,
        ))
    }

    pub(super) fn prepare_index_signature(
        &self,
        node: TsIndexSignature,
    ) -> Result<PreparedIndexSignature<'ast>, FormatError> {
        let ast = self.ast;
        let span = node.span(ast);
        let annotation = node.type_annotation(ast);
        let annotation_span = annotation.span(ast);
        let parameters = node.parameters(ast);
        let (Some(first_id), Some(last_id)) = (parameters.first(), parameters.last()) else {
            return Err(self.unsupported("empty index signature", node));
        };
        let first = ast.get_node_in_sub_range(first_id);
        let last = ast.get_node_in_sub_range(last_id);
        let first_start = first.span(ast).start;
        let last_end = last.span(ast).end;
        let before = Span::new(span.start, first_start);
        let (opening, leading, prefix, head_start) =
            self.prepare_index_signature_prefix(node, before)?;
        let after = Span::new(last_end, annotation_span.start);
        let Some(closing) = self.find_token(after, self.comments.in_range(after), "]") else {
            return Err(self.unsupported("index signature closing", node));
        };
        let source_head = if self.has_suppression {
            (self.content_mode(self.comments.in_range(Span::new(head_start, first_start)))
                == ContentMode::Verbatim)
                .then_some(first)
        } else {
            None
        };
        let closing_span = Span::new(last_end, closing);
        let closing_comments = self.comments.in_range(closing_span);
        let mut source_return = None;
        let mut source_tail = None;
        if self.has_suppression {
            for (_, position) in self.suppressed_comment_runs(closing_comments, closing) {
                if matches!(position, SuppressionPosition::OwnLine) {
                    source_return = Some(annotation);
                } else {
                    source_tail = Some(last);
                }
            }
        }
        let closing_gap = self.prepare_type_gap(
            closing_span,
            self.content_mode(closing_comments) == ContentMode::Verbatim,
        );
        let result = self.prepare_signature_return(
            closing + 1,
            SignatureReturnEolTarget::Parameter,
            annotation,
            source_return,
            SignatureReturnDelimiter::Colon,
        );
        if result.preceding_mode == ContentMode::Verbatim {
            source_tail = Some(last);
        }
        Ok(PreparedIndexSignature {
            parameters,
            content_end: annotation_span.end,
            opening,
            closing,
            prefix,
            leading,
            closing_gap,
            source_head,
            source_tail,
            result,
        })
    }

    fn prepare_index_signature_prefix(
        &self,
        node: TsIndexSignature,
        span: Span,
    ) -> Result<
        (
            u32,
            Option<TypeMemberLeading<'ast>>,
            IndexSignaturePrefix<'ast>,
            u32,
        ),
        FormatError,
    > {
        let comments = self.comments.in_range(span);
        let opening = self
            .find_token(span, comments, "[")
            .ok_or_else(|| self.unsupported("index signature opening", node))?;
        let (prefix, head_start, leading_end) = if node.readonly(self.ast) {
            let prefix_span = Span::new(span.start, opening);
            let token = self
                .find_token(prefix_span, comments, "readonly")
                .ok_or_else(|| self.unsupported("index signature modifier boundary", node))?;
            let end = token + 8;
            let gap = Span::new(end, opening);
            (
                IndexSignaturePrefix::Readonly(self.prepare_type_gap(
                    gap,
                    self.has_suppression
                        && self.content_mode(self.comments.in_range(gap)) == ContentMode::Verbatim,
                )),
                end,
                token,
            )
        } else {
            (IndexSignaturePrefix::Bare, opening + 1, opening)
        };
        let leading = self.prepare_type_member_leading(Span::new(span.start, leading_end));
        Ok((opening, leading, prefix, head_start))
    }

    fn prepare_type_member_leading(&self, span: Span) -> Option<TypeMemberLeading<'ast>> {
        let comments = self.comments.in_range(span);
        if comments.is_empty() {
            return None;
        }
        if self.has_suppression && self.content_mode(comments) == ContentMode::Verbatim {
            return Some(TypeMemberLeading::Source(span));
        }
        let gap = self
            .prepare_comment_boundary(comments, span.start, span.end)
            .gap();
        Some(TypeMemberLeading::Formatted { comments, gap })
    }

    fn format_type_member_leading(
        &mut self,
        leading: Option<TypeMemberLeading<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        match leading {
            None => Ok(()),
            Some(TypeMemberLeading::Source(span)) => self.format_verbatim(span, writer),
            Some(TypeMemberLeading::Formatted { comments, gap }) => {
                self.comments.claim(comments)?;
                self.format_comment_run(comments, CommentRunLayout::Source, writer)?;
                gap.write(writer, FlatSeparator::Space, Writer::line);
                Ok(())
            }
        }
    }

    fn format_type_member_key(
        &mut self,
        key: MemberKey<'ast>,
        context: PropertyKeyContext,
        quoting: PropertyKeyQuoting,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        if let Some((keyword, gap)) = key.prefix {
            writer.text(keyword);
            self.format_type_gap(gap, FlatSeparator::Space, writer)?;
        }
        self.format_property_key_context(key.node, key.computed, key.mode, context, quoting, writer)
    }

    fn format_type_member(
        &mut self,
        mut member: PreparedMember<'ast>,
        quoting: PropertyKeyQuoting,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let before_separator = member.before_separator.take();
        let tail = self.format_type_member_content(member.content, quoting, writer)?;
        if let Some(before) = before_separator {
            self.format_expression_tail(tail, writer)?;
            self.format_type_gap(before, FlatSeparator::Empty, writer)?;
            Ok(None)
        } else {
            Ok(tail)
        }
    }

    fn format_type_member_content(
        &mut self,
        content: MemberContent<'ast>,
        quoting: PropertyKeyQuoting,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        match content {
            MemberContent::Source(span) => {
                self.format_verbatim(span, writer)?;
                Ok(None)
            }
            MemberContent::Property(property) => {
                self.format_property_member(property, quoting, writer)
            }
            MemberContent::Signature(mut signature) => {
                let generics = signature.generics.as_ref().map(|(node, _, _)| *node);
                let can_group_parameters = self.signature_parameters_can_group(
                    signature.parameters.count() == 1,
                    generics,
                    signature.result.as_ref(),
                );
                let group_return = signature.key.is_some();
                if let Some(key) = &mut signature.key {
                    self.format_type_member_leading(key.leading.take(), writer)?;
                }
                writer.group(false, |writer| {
                    if let Some(key) = signature.key {
                        self.format_type_member_key(
                            key,
                            PropertyKeyContext::TypeMethod,
                            quoting,
                            writer,
                        )?;
                    } else if signature.construct {
                        writer.text("new");
                    }
                    if let Some(before) = signature.optional_before {
                        self.format_type_gap(before, FlatSeparator::Empty, writer)?;
                    }
                    if signature.optional && matches!(signature.prefix, TypeGap::Formatted(_)) {
                        writer.text("?");
                    }
                    self.format_type_gap(
                        signature.prefix,
                        if signature.construct {
                            FlatSeparator::Space
                        } else {
                            FlatSeparator::Empty
                        },
                        writer,
                    )?;
                    self.format_callable_signature_contents(
                        signature.parameters,
                        signature.result,
                        signature
                            .generics
                            .map(|(node, mode, following)| SignatureGenerics {
                                node,
                                mode,
                                following,
                            }),
                        can_group_parameters,
                        group_return,
                        writer,
                    )
                })
            }
            MemberContent::Index(index) => self.format_index_signature(index, writer),
        }
    }

    fn format_property_member(
        &mut self,
        mut property: PropertyMember<'ast>,
        quoting: PropertyKeyQuoting,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_type_member_leading(property.key.leading.take(), writer)?;
        let source_prefix = property
            .annotation
            .as_ref()
            .is_some_and(|(_, prefix)| matches!(prefix, TypeBoundaryLayout::Source(_)));
        self.format_type_member_key(
            property.key,
            PropertyKeyContext::TypeProperty,
            quoting,
            writer,
        )?;
        if let Some((node, prefix)) = property.annotation {
            if property.optional && !source_prefix {
                writer.text("?");
            }
            match prefix {
                TypeBoundaryLayout::Source(span) => self.format_verbatim(span, writer)?,
                TypeBoundaryLayout::Formatted { before, after } => {
                    self.format_type_comment_gap(&before, FlatSeparator::Empty, writer)?;
                    writer.text(":");
                    return self.format_prepared_type_after_comments(
                        node,
                        TypeContext::Root,
                        &after,
                        TypeLayout::Root,
                        writer,
                    );
                }
            }
            self.format_prepared_type_with_layout(
                node,
                TypeContext::Root,
                TypeLayout::OpeningProvided,
                writer,
            )
        } else {
            if let Some(ending) = property.ending {
                self.format_type_gap(ending, FlatSeparator::Empty, writer)?;
            }
            if property.optional {
                writer.text("?");
            }
            Ok(None)
        }
    }

    pub(super) fn format_index_signature(
        &mut self,
        index: PreparedIndexSignature<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let ast = self.ast;
        self.format_type_member_leading(index.leading, writer)?;
        match index.prefix {
            IndexSignaturePrefix::Bare => {}
            IndexSignaturePrefix::Readonly(gap) => {
                writer.text("readonly");
                self.format_type_gap(gap, FlatSeparator::Space, writer)?;
            }
        };

        writer.group(false, |writer| {
            writer.text("[");
            writer.indent(|writer| {
                let mut members = index
                    .parameters
                    .iter()
                    .map(|id| ast.get_node_in_sub_range(id))
                    .peekable();
                let mut end = index.opening + 1;
                let mut following_mode = None;
                let mut separator = Separator::Line;
                while let Some(node) = members.next() {
                    let span = node.span(ast);
                    if following_mode.is_some() {
                        ArgumentLayout::Lines.separator(
                            separator,
                            self.line_constraint(end, span.start),
                            writer,
                        );
                    }
                    let following = members.peek().map_or(
                        FollowingItem::ResolvedTerminal(DelimitedSuppression {
                            preceding: index.source_tail.map(|node| node.span(ast)),
                            following: ContentMode::Formatted,
                        }),
                        |next| FollowingItem::Node(next.span(ast)),
                    );
                    let mut item = self.prepare_delimited_item(
                        span,
                        following,
                        Span::new(end, index.closing),
                        false,
                        writer,
                    )?;
                    if let Some(mode) = following_mode {
                        item.set_leading_mode(mode);
                    }
                    if index.source_head == Some(node) || index.source_tail == Some(node) {
                        item.set_leading_mode(ContentMode::Verbatim);
                    }
                    let tail = if item.mode == ContentMode::Verbatim {
                        self.format_verbatim(span, writer)?;
                        None
                    } else {
                        let name = node.name(ast);
                        let annotation = self.prepare_annotation_content(
                            name.span(ast).end,
                            node.type_annotation(ast),
                        )?;
                        writer.text(ast.get_utf8(name.name(ast)));
                        self.format_annotation_content(annotation, writer)?
                    };
                    let boundary = if members.peek().is_some() {
                        Some(self.finish_delimited_item(item, tail, writer)?)
                    } else {
                        self.format_expression_tail(tail, writer)?;
                        None
                    };
                    if let Some(boundary) = boundary {
                        end = boundary.end;
                        following_mode = Some(boundary.following_mode);
                        separator = boundary.separator;
                    }
                }
                self.format_type_gap(index.closing_gap, FlatSeparator::Empty, writer)
            })?;
            writer.text("]");
            self.format_signature_return(index.result, writer)
        })
    }

    pub(super) fn type_literal_requires_break(&self, node: TsTypeLiteral) -> bool {
        let ast = self.ast;
        node.members(ast).first().is_some_and(|id| {
            let member: TsTypeElement = ast.get_node_in_sub_range(id);
            self.type_body_expands(node.span(ast).start + 1, member.span(ast).start)
        })
    }
}
