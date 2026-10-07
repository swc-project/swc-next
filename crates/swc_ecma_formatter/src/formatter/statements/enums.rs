//! Enum declarations retain real member names, initializer expressions and comma owners.

use swc_next_ecma_ast::{
    BindingIdentifier, GetSpan, Span, SubRange, TsEnumBody, TsEnumDeclaration, TsEnumMember,
    TsEnumMemberName, TsEnumMemberNameData,
};

use super::Formatter;
use super::declarations::AmbientDeclarationHead;
use crate::formatter::bindings::{InitializerLayout, InitializerLeft, PreparedInitializer};
use crate::formatter::comments::{CommentBoundary, CommentRunLayout, ExpressionTail};
use crate::formatter::diagnostics::FlatSeparator;
use crate::formatter::expressions::arguments::{ArgumentLayout, FollowingItem, ItemBoundary};
use crate::formatter::expressions::assignment::RhsLayout;
use crate::formatter::expressions::objects::{PropertyKeyContext, PropertyKeyQuoting};
use crate::formatter::expressions::{ExpressionContext, ExpressionPosition};
use crate::formatter::literals::StringContext;
use crate::formatter::suppression::{ContentMode, SuppressionPosition};
use crate::formatter::types::TypeGap;
use crate::{FormatError, QuoteProperties, TrailingCommas, document::Writer};

enum EnumPrefix<'ast> {
    Name(CommentBoundary<'ast>),
    Source {
        ambient: Option<AmbientDeclarationHead<'ast>>,
        modifier: Option<TypeGap<'ast>>,
        name: TypeGap<'ast>,
    },
}

struct EnumHead<'ast> {
    declare: bool,
    constant: bool,
    name: BindingIdentifier,
    name_mode: ContentMode,
    prefix: EnumPrefix<'ast>,
    body: TsEnumBody,
    body_prefix: TypeGap<'ast>,
    body_mode: ContentMode,
}

struct EnumMemberKey {
    name: TsEnumMemberName,
    computed: Option<Span>,
    span: Span,
}

struct EnumMember {
    node: TsEnumMember,
    key: EnumMemberKey,
    span: Span,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_enum(
        &mut self,
        node: TsEnumDeclaration,
        ambient: Option<AmbientDeclarationHead<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        if node.declare(self.ast) && ambient.is_none() {
            return Err(self.unsupported("ambient enum declaration keyword", node));
        }
        let head = self.prepare_enum_head(node, ambient)?;
        match head.prefix {
            EnumPrefix::Name(boundary) => {
                if head.declare {
                    writer.text("declare");
                    writer.space();
                }
                if head.constant {
                    writer.text("const");
                    writer.space();
                }
                writer.text("enum");
                self.format_type_gap(TypeGap::Formatted(boundary), FlatSeparator::Space, writer)?;
            }
            EnumPrefix::Source {
                ambient,
                modifier,
                name,
            } => {
                if let Some(ambient) = ambient {
                    self.format_ambient_declaration_prefix(&ambient, writer)?;
                }
                if let Some(modifier) = modifier {
                    writer.text("const");
                    self.format_type_gap(modifier, FlatSeparator::Space, writer)?;
                }
                writer.text("enum");
                self.format_type_gap(name, FlatSeparator::Space, writer)?;
            }
        }
        if head.name_mode == ContentMode::Verbatim {
            self.format_verbatim(head.name.span(self.ast), writer)?;
        } else {
            writer.text(self.ast.get_utf8(head.name.name(self.ast)));
        }
        self.format_type_gap(head.body_prefix, FlatSeparator::Space, writer)?;
        if head.body_mode == ContentMode::Verbatim {
            self.format_verbatim(head.body.span(self.ast), writer)
        } else {
            self.format_enum_body(head.body, writer)
        }
    }

    fn prepare_enum_head(
        &self,
        node: TsEnumDeclaration,
        ambient: Option<AmbientDeclarationHead<'ast>>,
    ) -> Result<EnumHead<'ast>, FormatError> {
        let ast = self.ast;
        let name = node.id(ast);
        let name_span = name.span(ast);
        let declare = node.declare(ast);
        let constant = node.is_const(ast);
        let keyword = ambient.as_ref().map(|head| head.keyword);
        let start = keyword.map_or(node.span(ast).start, |keyword| keyword.span.start);
        let (modifier, enum_end) = if constant {
            let const_end = start + "const".len() as u32;
            let search = Span::new(const_end, name_span.start);
            let enum_start = self
                .find_token(search, self.comments.in_range(search), "enum")
                .ok_or_else(|| self.unsupported("const enum keyword boundary", node))?;
            (
                Some(Span::new(const_end, enum_start)),
                enum_start + "enum".len() as u32,
            )
        } else {
            (None, start + "enum".len() as u32)
        };
        let prefix_start = if declare {
            node.span(ast).start + "declare".len() as u32
        } else {
            modifier.map_or(enum_end, |span| span.start)
        };
        let prefix_span = Span::new(prefix_start, name_span.start);
        let prefix_comments = self.comments.in_range(prefix_span);
        let name_mode = if keyword.is_some_and(|keyword| keyword.suppresses_name(name)) {
            ContentMode::Verbatim
        } else {
            self.content_mode(prefix_comments)
        };
        let binding = self.comment_binding(prefix_comments, prefix_span, Some(name_span));
        let prefix = if binding.allows_relocation(Some(name_span)) {
            EnumPrefix::Name(self.prepare_comment_boundary(
                prefix_comments,
                prefix_start,
                name_span.start,
            ))
        } else {
            let name_prefix = Span::new(enum_end, name_span.start);
            let name_comments = self.comments.in_range(name_prefix);
            EnumPrefix::Source {
                ambient,
                modifier: modifier.map(|span| {
                    let mode = self.content_mode(self.comments.in_range(span));
                    self.prepare_type_gap(span, mode == ContentMode::Verbatim)
                }),
                name: self.prepare_type_gap(
                    name_prefix,
                    self.content_mode(name_comments) == ContentMode::Verbatim,
                ),
            }
        };
        let body = node.body(ast);
        let body_span = Span::new(name_span.end, body.span(ast).start);
        let body_comments = self.comments.in_range(body_span);
        let body_mode = self.content_mode(body_comments);
        Ok(EnumHead {
            declare,
            constant,
            name,
            name_mode,
            prefix,
            body,
            body_prefix: self.prepare_type_gap(body_span, body_mode == ContentMode::Verbatim),
            body_mode,
        })
    }

    fn format_enum_body(
        &mut self,
        body: TsEnumBody,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
        let span = body.span(ast);
        let content = Span::new(span.start + 1, span.end - 1);
        let quoting = match self.options.quote_properties {
            QuoteProperties::AsNeeded => PropertyKeyQuoting::AsNeeded,
            QuoteProperties::Preserve => PropertyKeyQuoting::Preserve,
        };
        let mut remaining = body.members(ast);
        let Some(first) = remaining.first().map(|id| ast.get_node_in_sub_range(id)) else {
            let comments = self.comments.between(content)?;
            let boundary = self.prepare_comment_boundary(comments, content.start, content.end);
            self.comments.claim(comments)?;
            writer.text("{");
            if !comments.is_empty() {
                writer.group(false, |writer| {
                    writer.indent(|writer| {
                        self.opening_line_constraint(content).write(
                            writer,
                            FlatSeparator::Empty,
                            Writer::soft_line,
                        );
                        self.format_comment_run(comments, CommentRunLayout::Source, writer)
                    })?;
                    boundary
                        .gap()
                        .write(writer, FlatSeparator::Empty, Writer::soft_line);
                    Ok::<_, FormatError>(())
                })?;
            }
            writer.text("}");
            return Ok(());
        };
        let remaining = remaining.split_off(1);
        self.format_enum_members(content, first, remaining, quoting, writer)
    }

    fn format_enum_members(
        &mut self,
        content: Span,
        first: TsEnumMember,
        remaining: SubRange<TsEnumMember>,
        quoting: PropertyKeyQuoting,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
        let mut members = remaining.iter().map(|id| ast.get_node_in_sub_range(id));
        let mut member = self.prepare_enum_member(first, content.start)?;
        writer.group(true, |writer| {
            writer.text("{");
            let end = writer.indent(|writer| {
                self.opening_line_constraint(content).write(
                    writer,
                    FlatSeparator::Empty,
                    Writer::line,
                );
                let mut previous: Option<ItemBoundary> = None;
                loop {
                    let previous_end = previous.map_or(content.start, |boundary| boundary.end);
                    let next = members
                        .next()
                        .map(|node| self.prepare_enum_member(node, member.span.end))
                        .transpose()?;
                    if let Some(boundary) = previous {
                        let next_start = self
                            .comments
                            .in_range(Span::new(previous_end, member.span.start))
                            .first()
                            .map_or(member.span.start, |comment| comment.span.start);
                        ArgumentLayout::Lines.separator(
                            boundary.separator,
                            self.line_constraint(previous_end, next_start),
                            writer,
                        );
                    }
                    let following = next
                        .as_ref()
                        .map_or(FollowingItem::End, |next| FollowingItem::Node(next.span));
                    let mut item = self.prepare_delimited_item(
                        member.span,
                        following,
                        Span::new(previous_end, content.end),
                        self.options.trailing_commas != TrailingCommas::None,
                        writer,
                    )?;
                    if let Some(boundary) = previous {
                        item.set_leading_mode(boundary.following_mode);
                    }
                    let tail = self.format_enum_member(member, item.mode, quoting, writer)?;
                    previous = Some(self.finish_delimited_item(item, tail, writer)?);
                    let Some(next) = next else {
                        break;
                    };
                    member = next;
                }
                Ok::<_, FormatError>(previous.map_or(content.start, |boundary| boundary.end))
            })?;
            self.line_constraint(end, content.end).write(
                writer,
                FlatSeparator::Empty,
                Writer::line,
            );
            writer.text("}");
            Ok(())
        })
    }

    fn prepare_enum_member(
        &self,
        node: TsEnumMember,
        previous_end: u32,
    ) -> Result<EnumMember, FormatError> {
        let ast = self.ast;
        let name = node.id(ast);
        let name_span = name.span(ast);
        let computed = if node.computed(ast) {
            let prefix = Span::new(previous_end, name_span.start);
            let opening = self
                .find_token(prefix, self.comments.in_range(prefix), "[")
                .ok_or_else(|| self.unsupported("computed enum name opening", node))?;
            let suffix = Span::new(
                name_span.end,
                node.initializer(ast)
                    .map_or(node.span(ast).end, |right| right.span(ast).start),
            );
            let closing = self
                .find_token(suffix, self.comments.in_range(suffix), "]")
                .ok_or_else(|| self.unsupported("computed enum name closing", node))?;
            Some(Span::new(opening, closing + 1))
        } else {
            None
        };
        let key_span = computed.unwrap_or(name_span);
        Ok(EnumMember {
            node,
            key: EnumMemberKey {
                name,
                computed,
                span: key_span,
            },
            span: Span::new(key_span.start, node.span(ast).end),
        })
    }

    fn format_enum_member(
        &mut self,
        member: EnumMember,
        mode: ContentMode,
        quoting: PropertyKeyQuoting,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        if mode == ContentMode::Verbatim {
            self.format_verbatim(member.span, writer)?;
            return Ok(None);
        }
        let ast = self.ast;
        let Some(right) = member.node.initializer(ast) else {
            self.format_enum_member_key(
                member.key.name,
                member.key.computed,
                mode,
                quoting,
                writer,
            )?;
            return Ok(None);
        };
        let key_end = member.key.span.end;
        let gap = Span::new(key_end, right.span(ast).start);
        let comments = self.comments.in_range(gap);
        let operator = self
            .find_token(gap, comments, "=")
            .ok_or_else(|| self.unsupported("enum initializer boundary", member.node))?;
        let content = self.expression_content(right);
        let mut boundary = self.assignment_boundary(
            key_end,
            &content,
            Span::new(operator, operator + 1),
            right.span(self.ast).end,
        )?;
        let mut left_mode = ContentMode::Formatted;
        if self.has_suppression {
            let before =
                &comments[..comments.partition_point(|comment| comment.span.end <= operator)];
            for position in self.suppression_positions(before, operator) {
                match position {
                    SuppressionPosition::Inline => left_mode = ContentMode::Verbatim,
                    SuppressionPosition::OwnLine | SuppressionPosition::EndOfLine => {
                        boundary.mode = ContentMode::Verbatim;
                        if let Some(comment) = boundary.before.last() {
                            boundary.before_operator =
                                self.source_line_constraint(comment.span.end, operator);
                        }
                    }
                }
            }
        }
        let rhs_layout = if boundary.own_line || !boundary.operator_line.is_empty() {
            RhsLayout::BreakAfterOperator
        } else {
            RhsLayout::NeverBreakAfterOperator
        };
        self.format_initializer(
            PreparedInitializer {
                left: InitializerLeft::Enum {
                    name: member.key.name,
                    computed: member.key.computed,
                    quoting,
                },
                right,
                layout: InitializerLayout::Formatted {
                    boundary,
                    rhs_layout,
                },
                left_mode,
                context: ExpressionContext::at(ExpressionPosition::EnumInitializer),
            },
            writer,
        )
    }

    pub(in crate::formatter) fn format_enum_member_key(
        &mut self,
        name: TsEnumMemberName,
        computed: Option<Span>,
        mode: ContentMode,
        quoting: PropertyKeyQuoting,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
        let span = computed.unwrap_or_else(|| name.span(ast));
        if mode == ContentMode::Verbatim {
            return self.format_verbatim(span, writer);
        }
        if let Some(computed) = computed {
            writer.text("[");
            self.format_module_gap(computed.start + 1, name.span(ast).start, false, writer)?;
        }
        match ast.ts_enum_member_name_data(name) {
            TsEnumMemberNameData::IdentifierName(node) => writer.text(ast.get_utf8(node.name(ast))),
            TsEnumMemberNameData::StringLiteral(node) => {
                if computed.is_some() {
                    self.format_string_literal(node, StringContext::Expression, writer);
                } else {
                    self.format_string_property_name(
                        node,
                        PropertyKeyContext::Ordinary,
                        quoting,
                        writer,
                    );
                }
            }
            TsEnumMemberNameData::TemplateLiteral(node) => {
                if computed.is_none() {
                    return Err(self.unsupported("noncomputed template enum member name", node));
                }
                self.format_zero_slot_template(node, writer)?;
            }
        }
        if let Some(computed) = computed {
            self.format_module_gap(name.span(ast).end, computed.end - 1, false, writer)?;
            writer.text("]");
        }
        Ok(())
    }
}
