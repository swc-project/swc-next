//! Real generic list owners share the item/comma protocol and retain child bindings.

use swc_next_ecma_ast::{
    Ast, BindingIdentifier, ClassBody, Comment, CommentKind, FunctionBody, GetSpan,
    ObjectExpression, Span, TemplateElement, TsLiteralData, TsMappedType, TsType, TsTypeData,
    TsTypeLiteral, TsTypeParameter, TsTypeParameterDeclaration, TsTypeParameterInstantiation,
    TsTypeReference, UnaryOperator, Visit, VisitWith,
};

use super::compound::PendingTypeHead;
use super::{
    Formatter, PreparedType, TypeAttachmentHead, TypeBoundary, TypeBoundaryTail,
    TypeBoundaryTarget, TypeContext, TypeLayout,
};
use crate::formatter::comments::{
    CommentBinding, CommentBoundary, CommentGap, CommentOpening, CommentRunLayout, ExpressionTail,
    PreparedBoundaryComments,
};
use crate::formatter::diagnostics::{FlatSeparator, LineConstraint};
use crate::formatter::expressions::arguments::{
    ArgumentLayout, FollowingItem, Separator, TrailingCommaPolicy,
};
use crate::formatter::source::is_source_horizontal_whitespace;
use crate::formatter::suppression::{ContentMode, SuppressionPosition, is_suppression_comment};
use crate::{
    FormatError, TrailingCommas,
    document::{ConditionTarget, GroupId, Writer},
};

#[derive(Clone, Copy)]
pub(in crate::formatter) enum TypeListKind {
    Arguments,
    Parameters { has_left_owner: bool },
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(in crate::formatter) enum TypeArgumentOwner {
    Ordinary,
    TypeQuery,
    ArrowVariableAnnotation,
}

#[derive(Clone, Copy)]
pub(in crate::formatter) enum TypeParameterContext {
    TypeDeclaration,
    Class,
    Callable,
    Infer,
}

#[derive(Clone, Copy)]
enum TypeList {
    Arguments(TsTypeParameterInstantiation, TypeArgumentOwner),
    Parameters(TsTypeParameterDeclaration),
}

impl TypeList {
    fn span(self, ast: &swc_next_ecma_ast::Ast<'_>) -> Span {
        match self {
            Self::Arguments(node, _) => node.span(ast),
            Self::Parameters(node) => node.span(ast),
        }
    }
}

#[derive(Clone, Copy)]
enum GenericMember {
    Type(TsType),
    Parameter(TsTypeParameter, TypeParameterContext),
}

impl GenericMember {
    fn span(self, ast: &swc_next_ecma_ast::Ast<'_>) -> Span {
        match self {
            Self::Type(node) => node.span(ast),
            Self::Parameter(node, _) => node.span(ast),
        }
    }
}

enum PreparedGenericMember {
    Type(PreparedType),
    Parameter(TsTypeParameter, TypeParameterContext),
}

enum TypeListPrefixLayout<'ast> {
    Source(Span),
    Formatted(CommentBoundary<'ast>),
    TrailingName(CommentBoundary<'ast>),
}

pub(in crate::formatter) struct TypeListPrefix<'ast> {
    pub(in crate::formatter) left_mode: ContentMode,
    pub(in crate::formatter) right_mode: ContentMode,
    layout: TypeListPrefixLayout<'ast>,
}

impl TypeListPrefix<'_> {
    pub(in crate::formatter) fn has_trailing_name_comment(&self) -> bool {
        matches!(self.layout, TypeListPrefixLayout::TrailingName(_))
    }
}

struct ParameterBound<'ast> {
    node: PreparedType,
    boundary: TypeBoundary<'ast>,
}

enum PreparedBoundContent<'ast> {
    Body {
        body: super::PreparedDelimitedType<'ast>,
        tail: Option<TypeBoundaryTail<'ast>>,
    },
    Head {
        recorded: super::RecordedTypeHead<'ast>,
        tail: Option<TypeBoundaryTail<'ast>>,
    },
}

#[derive(Clone, Copy)]
struct TypeParameterModifiers {
    r#const: bool,
    r#in: bool,
    out: bool,
}

impl TypeParameterModifiers {
    fn is_empty(self) -> bool {
        !self.r#const && !self.r#in && !self.out
    }
}

enum PreparedParameterPrefix<'ast> {
    Canonical {
        modifiers: TypeParameterModifiers,
        name_leading: Option<CommentBoundary<'ast>>,
    },
    Source(Span),
}

pub(super) struct PreparedParameter<'ast> {
    prefix: PreparedParameterPrefix<'ast>,
    name: BindingIdentifier,
    name_mode: ContentMode,
    constraint: Option<ParameterBound<'ast>>,
    default: Option<ParameterBound<'ast>>,
}

/// A complete head whose raw type owner survives both angle layouts.
#[derive(Clone, Copy)]
pub(in crate::formatter) struct TypeAnglePacket<'ast> {
    comments: &'ast [Comment],
    next_start: u32,
}

/// Source expansion preferences and literal lines in a comment/diagnostic-clear
/// type list. Other type groups remain free to break by width.
struct RequiredTypeBreak<'formatter, 'ast, 'source> {
    formatter: &'formatter Formatter<'ast, 'source>,
    required: bool,
}

impl<'source> Visit<'source> for RequiredTypeBreak<'_, '_, 'source> {
    fn ast(&self) -> &Ast<'source> {
        self.formatter.ast
    }

    fn visit_ts_type(&mut self, node: TsType) {
        if !self.required {
            node.visit_children_with(self);
        }
    }

    fn visit_ts_type_literal(&mut self, node: TsTypeLiteral) {
        self.required |= self.formatter.type_literal_requires_break(node);
        if !self.required {
            node.visit_children_with(self);
        }
    }

    fn visit_ts_mapped_type(&mut self, node: TsMappedType) {
        self.required |= self.formatter.mapped_type_requires_break(node);
        if !self.required {
            node.visit_children_with(self);
        }
    }

    fn visit_object_expression(&mut self, node: ObjectExpression) {
        self.required |= self.formatter.object_body_expands(node);
        if !self.required {
            node.visit_children_with(self);
        }
    }

    fn visit_function_body(&mut self, node: FunctionBody) {
        self.required |= self.formatter.function_body_requires_break(node);
    }

    fn visit_class_body(&mut self, node: ClassBody) {
        self.required |= self.formatter.class_body_first_member(node).is_some();
    }

    fn visit_template_element(&mut self, node: TemplateElement) {
        self.required |= self
            .ast()
            .get_utf8(node.raw(self.ast()))
            .contains(['\r', '\n']);
    }
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(in crate::formatter) fn prepare_type_list_prefix(
        &self,
        gap: Span,
        kind: TypeListKind,
    ) -> TypeListPrefix<'ast> {
        if gap.start == gap.end {
            return TypeListPrefix {
                left_mode: ContentMode::Formatted,
                right_mode: ContentMode::Formatted,
                layout: TypeListPrefixLayout::Formatted(CommentBoundary::adjacent(gap.start)),
            };
        }
        let comments = self.comments.in_range(gap);
        let mut left_mode = ContentMode::Formatted;
        let mut right_mode = ContentMode::Formatted;
        if self.has_suppression && self.content_mode(comments) == ContentMode::Verbatim {
            for (run, position) in self.suppressed_comment_runs(comments, gap.end) {
                for &comment in run {
                    if !is_suppression_comment(self.ast, comment) {
                        continue;
                    }
                    if matches!(
                        kind,
                        TypeListKind::Parameters {
                            has_left_owner: true
                        }
                    ) && matches!(position, SuppressionPosition::EndOfLine)
                        && comment.kind == CommentKind::Line
                    {
                        left_mode = ContentMode::Verbatim;
                    } else {
                        right_mode = ContentMode::Verbatim;
                    }
                }
            }
        }
        let layout = if left_mode == ContentMode::Verbatim || right_mode == ContentMode::Verbatim {
            TypeListPrefixLayout::Source(gap)
        } else {
            TypeListPrefixLayout::Formatted(
                self.prepare_comment_boundary(comments, gap.start, gap.end),
            )
        };
        TypeListPrefix {
            left_mode,
            right_mode,
            layout,
        }
    }

    pub(in crate::formatter) fn prepare_declaration_type_list_prefix(
        &self,
        gap: Span,
        node: TsTypeParameterDeclaration,
        following_start: u32,
    ) -> TypeListPrefix<'ast> {
        let mut prefix = self.prepare_type_list_prefix(
            gap,
            TypeListKind::Parameters {
                has_left_owner: true,
            },
        );
        let TypeListPrefixLayout::Formatted(boundary) = &prefix.layout else {
            return prefix;
        };
        // A name's trailing Line can cross the angle suffix only while no
        // intervening comment can be overtaken before the heritage break.
        let [comment] = boundary.comments() else {
            return prefix;
        };
        if comment.kind != CommentKind::Line
            || self.line_breaks(gap.start, comment.span.start) != 0
            || self.diagnostic_range_is_sensitive(gap)
        {
            return prefix;
        }
        let semantics = self.comment_semantics(*comment);
        if semantics.diagnostic
            || !self
                .comments
                .in_range(Span::new(node.span(self.ast).start, following_start))
                .is_empty()
        {
            return prefix;
        }
        if let TypeListPrefixLayout::Formatted(boundary) = prefix.layout {
            prefix.layout = TypeListPrefixLayout::TrailingName(boundary);
        }
        prefix
    }

    pub(in crate::formatter) fn format_declaration_type_parameters(
        &mut self,
        node: TsTypeParameterDeclaration,
        prefix: TypeListPrefix<'ast>,
        mode: ContentMode,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let tail = match &prefix.layout {
            TypeListPrefixLayout::TrailingName(boundary) => {
                // Claim in source order, then attach the suffix to the completed
                // angle group so its internal breaks cannot flush this Line.
                let (tail, ()) = writer
                    .record_with_value(|writer| self.format_comment_boundary(boundary, writer))?;
                Some(tail)
            }
            _ => {
                self.format_type_list_prefix(&prefix, writer)?;
                None
            }
        };
        self.format_type_parameters(node, TypeParameterContext::Class, mode, writer)?;
        if let Some(tail) = tail {
            writer.write_fragment(tail);
        }
        Ok(())
    }

    pub(in crate::formatter) fn format_type_list_prefix(
        &mut self,
        prefix: &TypeListPrefix<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        match &prefix.layout {
            TypeListPrefixLayout::Source(gap) => self.format_verbatim(*gap, writer),
            TypeListPrefixLayout::Formatted(boundary) => {
                self.format_type_comment_gap(boundary, FlatSeparator::Empty, writer)
            }
            TypeListPrefixLayout::TrailingName(boundary) => {
                self.format_comment_boundary(boundary, writer)
            }
        }
    }

    pub(super) fn format_type_reference(
        &mut self,
        node: TsTypeReference,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_type_reference_with_owner(node, TypeArgumentOwner::Ordinary, writer)
    }

    pub(super) fn format_type_reference_with_owner(
        &mut self,
        node: TsTypeReference,
        owner: TypeArgumentOwner,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let ast = self.ast;
        let name = node.type_name(ast);
        let arguments = node.type_arguments(ast).map(|node| {
            let prefix = self.prepare_type_list_prefix(
                Span::new(name.span(ast).end, node.span(ast).start),
                TypeListKind::Arguments,
            );
            (node, prefix)
        });
        let mode = arguments
            .as_ref()
            .map_or(ContentMode::Formatted, |(_, prefix)| prefix.left_mode);
        self.format_type_name(name, mode, writer)?;
        if let Some((arguments, ref prefix)) = arguments {
            let mode = prefix.right_mode;
            self.format_type_list_prefix(prefix, writer)?;
            self.format_type_arguments_with_owner(arguments, mode, owner, writer)?;
        }
        Ok(None)
    }

    pub(in crate::formatter) fn format_type_arguments(
        &mut self,
        node: TsTypeParameterInstantiation,
        mode: ContentMode,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        self.format_type_arguments_with_owner(node, mode, TypeArgumentOwner::Ordinary, writer)
    }

    pub(super) fn format_type_arguments_with_owner(
        &mut self,
        node: TsTypeParameterInstantiation,
        mode: ContentMode,
        owner: TypeArgumentOwner,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        if mode == ContentMode::Verbatim {
            return self.format_verbatim(node.span(self.ast), writer);
        }
        let ast = self.ast;
        let members = node
            .params(ast)
            .iter()
            .map(|id| GenericMember::Type(ast.get_node_in_sub_range(id)));
        self.format_generic_list(
            TypeList::Arguments(node, owner),
            members,
            TrailingCommaPolicy::Forbidden,
            writer,
        )
    }

    pub(in crate::formatter) fn format_type_parameters(
        &mut self,
        node: TsTypeParameterDeclaration,
        context: TypeParameterContext,
        mode: ContentMode,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let tail = if self.options.trailing_commas == TrailingCommas::None {
            TrailingCommaPolicy::Forbidden
        } else {
            TrailingCommaPolicy::IfBreak(ConditionTarget::Current)
        };
        self.format_type_parameters_with_tail(node, context, mode, tail, writer)
    }

    pub(in crate::formatter) fn format_type_parameters_with_tail(
        &mut self,
        node: TsTypeParameterDeclaration,
        context: TypeParameterContext,
        mode: ContentMode,
        tail: TrailingCommaPolicy,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        if mode == ContentMode::Verbatim {
            return self.format_verbatim(node.span(self.ast), writer);
        }
        let ast = self.ast;
        let members = node
            .params(ast)
            .iter()
            .map(|id| GenericMember::Parameter(ast.get_node_in_sub_range(id), context));
        self.format_generic_list(TypeList::Parameters(node), members, tail, writer)
    }

    fn simple_type_argument(&self, data: TsTypeData) -> bool {
        match data {
            TsTypeData::TsAnyKeyword(_)
            | TsTypeData::TsUnknownKeyword(_)
            | TsTypeData::TsNeverKeyword(_)
            | TsTypeData::TsVoidKeyword(_)
            | TsTypeData::TsNullKeyword(_)
            | TsTypeData::TsUndefinedKeyword(_)
            | TsTypeData::TsStringKeyword(_)
            | TsTypeData::TsNumberKeyword(_)
            | TsTypeData::TsBigIntKeyword(_)
            | TsTypeData::TsBooleanKeyword(_)
            | TsTypeData::TsSymbolKeyword(_)
            | TsTypeData::TsObjectKeyword(_)
            | TsTypeData::TsIntrinsicKeyword(_)
            | TsTypeData::TsThisType(_)
            | TsTypeData::TsTemplateLiteralType(_) => true,
            TsTypeData::TsTypeReference(reference) => reference.type_arguments(self.ast).is_none(),
            TsTypeData::TsLiteralType(literal) => {
                match self.ast.ts_literal_data(literal.literal(self.ast)) {
                    TsLiteralData::StringLiteral(_)
                    | TsLiteralData::NumericLiteral(_)
                    | TsLiteralData::BigIntLiteral(_)
                    | TsLiteralData::BooleanLiteral(_) => true,
                    TsLiteralData::TemplateLiteral(template) => {
                        template.expressions(self.ast).is_empty()
                    }
                    TsLiteralData::UnaryExpression(unary) => {
                        unary.operator(self.ast) == UnaryOperator::Negate
                            && matches!(
                                self.ast.expr_data(unary.argument(self.ast)),
                                swc_next_ecma_ast::ExprData::NumericLiteral(_)
                                    | swc_next_ecma_ast::ExprData::BigIntLiteral(_)
                            )
                    }
                }
            }
            _ => false,
        }
    }

    /// The type selector, rather than comment spelling, proves that changing
    /// this complete packet's physical runs cannot change the protected child.
    pub(in crate::formatter) fn prepare_type_angle_packet(
        &self,
        node: PreparedType,
        attachment: &TypeAttachmentHead,
        gap: Span,
        comments: &'ast [Comment],
    ) -> Option<TypeAnglePacket<'ast>> {
        let owner = node.source_owner?;
        if comments.is_empty()
            || self
                .diagnostic_range_is_sensitive(Span::new(gap.start, node.node.span(self.ast).end))
            || !self
                .comment_binding(comments, gap, Some(owner.span(self.ast)))
                .allows_relocation(Some(owner.span(self.ast)))
        {
            return None;
        }
        let retention = self.prepare_type_suppression_head(attachment, comments);
        let candidate = self.prepare_type(node.node, ContentMode::Formatted);
        for position in [
            SuppressionPosition::Inline,
            SuppressionPosition::EndOfLine,
            SuppressionPosition::OwnLine,
        ] {
            let mut moved = candidate;
            self.select_type_head_with_retention(&mut moved, attachment, retention, position);
            if moved.source_owner != node.source_owner || moved.source_head != node.source_head {
                return None;
            }
        }
        Some(TypeAnglePacket {
            comments,
            next_start: gap.end,
        })
    }

    pub(in crate::formatter) fn format_type_angle_packet(
        &mut self,
        packet: TypeAnglePacket<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        self.comments.claim(packet.comments)?;
        self.format_comment_run(
            packet.comments,
            CommentRunLayout::GroupedLeading(packet.next_start),
            writer,
        )
    }

    /// The angle token owns its horizontal opening; the source packet keeps
    /// every gap that can select a different ignored type or physical line.
    pub(in crate::formatter) fn type_angle_source_head(
        &self,
        span: Span,
        comments: &[Comment],
    ) -> Span {
        let Some(first) = comments.first() else {
            return span;
        };
        if self
            .source(Span::new(span.start, first.span.start))
            .chars()
            .all(is_source_horizontal_whitespace)
        {
            Span::new(first.span.start, span.end)
        } else {
            span
        }
    }

    fn format_generic_list(
        &mut self,
        node: TypeList,
        members: impl Iterator<Item = GenericMember>,
        tail: TrailingCommaPolicy,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
        let span = node.span(ast);
        let content = Span::new(span.start + 1, span.end - 1);
        let type_query = matches!(node, TypeList::Arguments(_, TypeArgumentOwner::TypeQuery));
        let opening_comments = if type_query {
            self.comments.in_range(content)
        } else {
            &[]
        };
        let mut members = members.peekable();
        let mut first_type = None;
        let mut first_angle_packet = None;
        let leading_source = if self.has_suppression {
            members.peek().and_then(|member| {
                let GenericMember::Type(first) = *member else {
                    return None;
                };
                let gap = Span::new(content.start, first.span(ast).start);
                let comments = self.comments.in_range(gap);
                if self.content_mode(comments) != ContentMode::Verbatim {
                    return None;
                }
                let mut node = self.prepare_type(first, ContentMode::Formatted);
                let attachment = self.type_attachment_head(node.node);
                self.resolve_type_head_comments(&mut node, &attachment, comments, gap.end);
                self.finalize_type_head(&mut node, &attachment, comments);
                first_type = Some(node);
                first_angle_packet =
                    self.prepare_type_angle_packet(node, &attachment, gap, comments);
                if first_angle_packet.is_some() {
                    return None;
                }
                // Compact and expanded openings must select the same ignored type.
                let opening = self.opening_line_constraint(content);
                let mut moved = node;
                let changes_own_line = if opening == LineConstraint::SameLine {
                    false
                } else {
                    moved.source_owner = None;
                    self.select_type_head_with_attachment(
                        &mut moved,
                        &attachment,
                        SuppressionPosition::OwnLine,
                    );
                    moved.source_owner != node.source_owner
                };
                // Source comment gaps containing a line force the list to expand.
                let can_inline = !matches!(opening, LineConstraint::NewLine(_))
                    && comments.iter().enumerate().all(|(index, comment)| {
                        let next = comments
                            .get(index + 1)
                            .map_or(gap.end, |comment| comment.span.start);
                        comment.kind != CommentKind::Line
                            && self.source_line_constraint(comment.span.start, next)
                                == LineConstraint::SameLine
                    });
                let changes_inline = if can_inline {
                    moved.source_owner = None;
                    self.select_type_head_with_attachment(
                        &mut moved,
                        &attachment,
                        SuppressionPosition::Inline,
                    );
                    moved.source_owner != node.source_owner
                } else {
                    false
                };
                (changes_own_line || changes_inline)
                    .then(|| self.type_angle_source_head(gap, comments))
            })
        } else {
            None
        };
        let mut first_body = None;
        let mut first_packet = None;
        let hug = if matches!(node, TypeList::Arguments(arguments, _) if arguments.params(ast).len() == 1)
            && let Some(GenericMember::Type(first)) = members.peek().copied()
        {
            let prepared =
                first_type.get_or_insert_with(|| self.prepare_type(first, ContentMode::Formatted));
            let simple_owner = matches!(
                node,
                TypeList::Arguments(
                    _,
                    TypeArgumentOwner::Ordinary | TypeArgumentOwner::TypeQuery
                )
            );
            let simple = prepared.source_owner.is_none()
                && simple_owner
                && self.simple_type_argument(prepared.data);
            let candidate = prepared.source_owner.is_none()
                && match prepared.data {
                    TsTypeData::TsTypeLiteral(_)
                    | TsTypeData::TsMappedType(_)
                    | TsTypeData::TsParenthesizedType(_) => true,
                    TsTypeData::TsUnionType(_) => true,
                    TsTypeData::TsIntersectionType(intersection) => {
                        intersection.types(ast).len() == 1
                    }
                    _ => simple,
                };
            if candidate {
                let first_span = first.span(ast);
                let before = Span::new(content.start, first_span.start);
                let after = Span::new(first_span.end, content.end);
                if [before, after].into_iter().all(|gap| {
                    let comments = self.comments.in_range(gap);
                    matches!(
                        self.comment_binding(comments, gap, Some(first_span)),
                        CommentBinding::Unbound
                    ) && !comments
                        .iter()
                        .any(|comment| comment.kind == CommentKind::Line)
                        && comments
                            .last()
                            .is_none_or(|comment| !self.comment_run_ends_line(*comment, gap.end))
                }) {
                    if !type_query
                        && (simple
                            || matches!(
                                prepared.data,
                                TsTypeData::TsTypeLiteral(_) | TsTypeData::TsMappedType(_)
                            ))
                    {
                        true
                    } else {
                        let mut body = self.prepare_delimited_type(*prepared)?;
                        let comments = self
                            .comments
                            .in_range(Span::new(content.start, first_span.end));
                        let movable = match &body.content {
                            super::compound::PreparedTypeContent::Parentheses(parentheses) => {
                                parentheses.movable
                            }
                            super::compound::PreparedTypeContent::Compound(single) => {
                                single.movable()
                            }
                            _ => false,
                        };
                        let mut head_inline = true;
                        if movable {
                            // Claim the complete exterior and moved head in source order.
                            // The original member span still owns its delimiter protocol.
                            let (head, (item, normalized)) =
                                writer.record_with_value(|writer| {
                                    let item = self.prepare_delimited_item(
                                        first_span,
                                        FollowingItem::End,
                                        content,
                                        false,
                                        None,
                                        writer,
                                    )?;
                                    let normalized = match body.content {
                                        super::compound::PreparedTypeContent::Compound(single) => {
                                            self.format_single_compound_head(
                                                single,
                                                TypeContext::Root,
                                                writer,
                                            )?
                                        }
                                        content => self.format_prepared_type_content_head(
                                            content,
                                            &mut PendingTypeHead::new(&[], first_span.start, None),
                                            TypeContext::Root,
                                            writer,
                                        )?,
                                    };
                                    Ok::<_, FormatError>((item, normalized))
                                })?;
                            if let super::compound::PreparedTypeContent::Type(node) = &normalized {
                                let next_start = node.node.span(ast).start;
                                let count = comments
                                    .partition_point(|comment| comment.span.end <= next_start);
                                let leading = &comments[..count];
                                head_inline = !leading
                                    .iter()
                                    .any(|comment| comment.kind == CommentKind::Line)
                                    && leading.last().is_none_or(|comment| {
                                        !self.comment_run_ends_line(*comment, next_start)
                                    });
                            }
                            body.content = normalized;
                            first_packet = Some((item, head));
                        }
                        let simple = matches!(&body.content, super::compound::PreparedTypeContent::Type(node)
                        if node.source_owner.is_none()
                            && if node.node == prepared.node {
                                simple
                            } else {
                                simple_owner && self.simple_type_argument(node.data)
                            });
                        let hug = (body.is_object_type()
                            || simple
                            || simple_owner
                                && matches!(&body.content, super::compound::PreparedTypeContent::Union(union)
                                if union.hug_head_start().is_some_and(|start| {
                                    comments.first().is_none_or(|comment| comment.span.start >= start)
                                })))
                            && head_inline;
                        first_body = Some(body);
                        hug
                    }
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };
        if type_query
            && first_body.is_none()
            && let Some(GenericMember::Type(first)) = members.peek().copied()
        {
            let prepared =
                first_type.get_or_insert_with(|| self.prepare_type(first, ContentMode::Formatted));
            first_body = Some(self.prepare_delimited_type(*prepared)?);
        }
        // The query owns a standalone `<` token. Prepare the actual first body
        // before choosing its flat gap: transparent wrappers can expose a generic
        // function, while retained pairs and leading comment packets separate it.
        let separate_opening = type_query
            && first_body.as_ref().is_some_and(|body| {
                matches!(&body.content, super::compound::PreparedTypeContent::Type(first)
                    if matches!(first.data, TsTypeData::TsFunctionType(function)
                        if function.type_parameters(ast).is_some())
                        && opening_comments.first().is_none_or(|comment| comment.span.start >= first.node.span(ast).start))
            });
        let opening_separator = if separate_opening {
            FlatSeparator::Space
        } else {
            FlatSeparator::Empty
        };
        let layout = if hug {
            TypeLayout::Root
        } else {
            TypeLayout::CallerIndented
        };
        let mut format = |writer: &mut Writer<'ast>| {
            writer.text("<");
            let mut format_members = |writer: &mut Writer<'ast>| {
                if leading_source.is_none() {
                    self.opening_line_constraint(content).write(
                        writer,
                        opening_separator,
                        |writer| {
                            if separate_opening {
                                writer.line();
                            } else if !hug {
                                writer.soft_line();
                            }
                        },
                    );
                }
                let mut end = content.start;
                let mut source_end = content.start;
                let mut transition = None;
                let mut separator = Separator::Line;
                let mut following_mode = None;
                while let Some(member) = members.next() {
                    let member_span = member.span(ast);
                    let leading = self
                        .comments
                        .in_range(Span::new(source_end, member_span.start));
                    let mode = following_mode.unwrap_or_else(|| self.content_mode(leading));
                    let mut prepared = match member {
                        GenericMember::Type(node) => PreparedGenericMember::Type(
                            first_type
                                .take()
                                .unwrap_or_else(|| self.prepare_type(node, ContentMode::Formatted)),
                        ),
                        GenericMember::Parameter(node, context) => {
                            PreparedGenericMember::Parameter(node, context)
                        }
                    };
                    if mode == ContentMode::Verbatim
                        && let PreparedGenericMember::Type(node) = &mut prepared
                        && node.source_owner.is_none()
                    {
                        self.select_delimited_type_head(
                            node,
                            leading,
                            member_span.start,
                            following_mode.is_none(),
                        );
                    }
                    if following_mode.is_some() {
                        let next_start = self
                            .comments
                            .between(Span::new(end, member_span.start))?
                            .first()
                            .map_or(member_span.start, |comment| comment.span.start);
                        ArgumentLayout::Lines.separator(
                            separator,
                            self.source_separator_constraint(end, next_start, transition.as_ref()),
                            writer,
                        );
                    }
                    let following = members.peek().map_or(FollowingItem::End, |member| {
                        FollowingItem::Node(member.span(ast))
                    });
                    // The first marker's physical run selects Whole/FirstMember.
                    // Keep that delimiter gap before claiming the comments, so a
                    // broken list cannot change its actual type owner next pass.
                    let item_start = if following_mode.is_none()
                        && let Some(packet) = first_angle_packet.take()
                    {
                        self.format_type_angle_packet(packet, writer)?;
                        member_span.start
                    } else if following_mode.is_none()
                        && let Some(gap) = leading_source
                    {
                        self.format_verbatim(gap, writer)?;
                        member_span.start
                    } else {
                        end
                    };
                    let mut item = match first_packet.take() {
                        Some((item, head)) => {
                            writer.write_fragment(head);
                            item
                        }
                        None => self.prepare_delimited_item(
                            member_span,
                            following,
                            Span::new(item_start, content.end),
                            false,
                            transition,
                            writer,
                        )?,
                    };
                    item.set_canonical_separator_spacing();
                    item.set_trailing_comma(tail);
                    let selected_child = matches!(&prepared, PreparedGenericMember::Type(node) if node.source_owner.is_some_and(|owner| owner != node.node));
                    item.set_leading_mode(if selected_child {
                        ContentMode::Formatted
                    } else {
                        mode
                    });
                    let tail = match prepared {
                        PreparedGenericMember::Type(mut node) => {
                            if item.mode == ContentMode::Verbatim {
                                node.source_owner = Some(node.node);
                            }
                            if let Some(body) = first_body.take() {
                                self.format_prepared_delimited_type(
                                    body,
                                    TypeContext::Root,
                                    layout,
                                    writer,
                                )?
                            } else {
                                self.format_prepared_type_with_layout(
                                    node,
                                    TypeContext::Root,
                                    layout,
                                    writer,
                                )?
                            }
                        }
                        PreparedGenericMember::Parameter(node, context) => {
                            self.format_type_parameter(node, context, item.mode, writer)?
                        }
                    };
                    let boundary = self.finish_delimited_item(item, tail, writer)?;
                    end = boundary.end;
                    source_end = member_span.end;
                    separator = boundary.separator;
                    transition = boundary.transition;
                    following_mode = Some(boundary.following_mode);
                }
                let dangling = self.comments.between(Span::new(end, content.end))?;
                let dangling = self.comments.take(dangling.len());
                if dangling
                    .iter()
                    .any(|comment| comment.kind == CommentKind::Line)
                {
                    writer.break_parent();
                }
                if following_mode.is_some()
                    && let Some(first) = dangling.first()
                {
                    self.comment_gap(end, first.span.start, writer);
                }
                self.format_comment_run(dangling, CommentRunLayout::Lines, writer)?;
                Ok::<_, FormatError>(dangling.last().map_or(end, |comment| comment.span.end))
            };
            let end = if hug {
                format_members(writer)?
            } else {
                writer.indent(format_members)?
            };
            self.line_constraint(end, content.end)
                .write(writer, FlatSeparator::Empty, |writer| {
                    if !hug {
                        writer.soft_line();
                    }
                });
            writer.text(">");
            Ok(())
        };
        if hug {
            format(writer)
        } else {
            writer.group(false, format)
        }
    }

    pub(super) fn prepare_type_parameter(
        &self,
        node: TsTypeParameter,
        context: TypeParameterContext,
    ) -> Result<PreparedParameter<'ast>, FormatError> {
        let ast = self.ast;
        let modifiers = TypeParameterModifiers {
            r#const: node.r#const(ast),
            r#in: node.r#in(ast),
            out: node.out(ast),
        };
        if matches!(context, TypeParameterContext::Infer)
            && (!modifiers.is_empty() || node.default(ast).is_some())
        {
            return Err(self.unsupported("infer type parameter modifiers or default", node));
        }
        let name = node.name(ast);
        let name_end = name.span(ast).end;
        let mut name_mode = ContentMode::Formatted;
        let mut constraint = node
            .constraint(ast)
            .map(|node| self.prepare_type(node, ContentMode::Formatted));
        // A default's preceding packet can select the complete constraint.
        // Resolve it before materializing the constraint's leading head.
        let default = node.default(ast).map(|type_node| {
            let mut type_node = self.prepare_type(type_node, ContentMode::Formatted);
            let left_end = constraint
                .as_ref()
                .map_or(name_end, |node| node.node.span(ast).end);
            let boundary = self.prepare_type_boundary(
                Span::new(left_end, type_node.node.span(ast).start),
                TypeBoundaryTarget::Default {
                    name: constraint.is_none().then_some(name),
                    node: &mut type_node,
                },
            );
            if boundary
                .as_ref()
                .is_ok_and(|boundary| boundary.left_mode == ContentMode::Verbatim)
            {
                if let Some(node) = &mut constraint {
                    node.source_owner = Some(node.node);
                } else {
                    name_mode = ContentMode::Verbatim;
                }
            }
            (type_node, boundary)
        });
        let constraint = constraint
            .map(|mut type_node| {
                let boundary = self.prepare_type_boundary(
                    Span::new(name_end, type_node.node.span(ast).start),
                    if matches!(context, TypeParameterContext::Infer) {
                        TypeBoundaryTarget::InferConstraint {
                            name,
                            node: &mut type_node,
                        }
                    } else {
                        TypeBoundaryTarget::Constraint {
                            name,
                            node: &mut type_node,
                        }
                    },
                )?;
                name_mode = boundary.left_mode;
                Ok::<_, FormatError>(ParameterBound {
                    node: type_node,
                    boundary,
                })
            })
            .transpose()?;
        // Preserve the original constraint-before-default error priority.
        let default = default
            .map(|(node, boundary)| boundary.map(|boundary| ParameterBound { node, boundary }))
            .transpose()?;
        let prefix = self.prepare_parameter_prefix(node, name, modifiers, &mut name_mode);
        Ok(PreparedParameter {
            prefix,
            name,
            name_mode,
            constraint,
            default,
        })
    }

    fn prepare_parameter_prefix(
        &self,
        node: TsTypeParameter,
        name: BindingIdentifier,
        modifiers: TypeParameterModifiers,
        name_mode: &mut ContentMode,
    ) -> PreparedParameterPrefix<'ast> {
        if modifiers.is_empty() || self.ast.comments().is_empty() {
            return PreparedParameterPrefix::Canonical {
                modifiers,
                name_leading: None,
            };
        }
        let name_span = name.span(self.ast);
        let span = Span::new(node.span(self.ast).start, name_span.start);
        let comments = self.comments.in_range(span);
        if self.has_suppression && self.content_mode(comments) == ContentMode::Verbatim {
            *name_mode = ContentMode::Verbatim;
        }
        let binding = self.comment_binding(comments, span, Some(name_span));
        if !binding.allows_relocation(Some(name_span)) {
            return PreparedParameterPrefix::Source(span);
        }
        let name_leading = (!comments.is_empty())
            .then(|| self.prepare_comment_boundary(comments, span.start, span.end));
        PreparedParameterPrefix::Canonical {
            modifiers,
            name_leading,
        }
    }

    fn format_type_parameter(
        &mut self,
        node: TsTypeParameter,
        context: TypeParameterContext,
        mode: ContentMode,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        if mode == ContentMode::Verbatim {
            self.format_verbatim(node.span(self.ast), writer)?;
            return Ok(None);
        }
        let parameter = self.prepare_type_parameter(node, context)?;
        self.format_prepared_type_parameter(parameter, writer)
    }

    pub(super) fn format_prepared_type_parameter(
        &mut self,
        parameter: PreparedParameter<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let ast = self.ast;
        writer.group(false, |writer| {
            self.format_parameter_prefix(parameter.prefix, writer)?;
            if parameter.name_mode == ContentMode::Verbatim {
                self.format_verbatim(parameter.name.span(ast), writer)?;
            } else {
                writer.text(ast.get_utf8(parameter.name.name(ast)));
            }
            match (parameter.constraint, parameter.default) {
                (Some(constraint), Some(default)) => {
                    self.format_parameter_bounds(constraint, default, writer)
                }
                (Some(constraint), None) => self.format_parameter_bound(
                    constraint,
                    TypeContext::TypeConstraint,
                    None,
                    writer,
                ),
                (None, Some(default)) => {
                    self.format_parameter_bound(default, TypeContext::Root, None, writer)
                }
                (None, None) => Ok(None),
            }
        })
    }

    fn format_parameter_bounds(
        &mut self,
        constraint: ParameterBound<'ast>,
        default: ParameterBound<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let movable = constraint.node.source_owner.is_none()
            && matches!(
                &constraint.boundary.layout,
                super::TypeOperatorLayout::Formatted { .. }
            );
        match default {
            ParameterBound {
                node,
                boundary:
                    TypeBoundary {
                        role,
                        layout: super::TypeOperatorLayout::Formatted { before, after },
                        ..
                    },
            } if movable && !after.is_empty() => {
                let comments = self.prepare_bound_comments(&after, node.node.span(self.ast));
                let (following, before, after, comments) = match comments {
                    PreparedBoundaryComments::Trailing { run, after, .. } => {
                        // The trailing prefix and its remaining head have disjoint
                        // source windows. Finish the head's preparation here.
                        let comments =
                            self.prepare_bound_comments(&after, node.node.span(self.ast));
                        (
                            Some(TypeBoundaryTail {
                                role,
                                before,
                                trailing: run,
                            }),
                            None,
                            after,
                            comments,
                        )
                    }
                    comments => (None, Some(before), after, comments),
                };
                let tail = self.format_parameter_bound(
                    constraint,
                    TypeContext::TypeConstraint,
                    following,
                    writer,
                )?;
                self.format_expression_tail(tail, writer)?;
                if let Some(before) = before {
                    self.format_formatted_type_boundary_operator(role, &before, writer)?;
                }
                self.format_parameter_bound_after_operator(
                    node,
                    TypeContext::Root,
                    &after,
                    comments,
                    None,
                    writer,
                )
            }
            default => {
                let tail = self.format_parameter_bound(
                    constraint,
                    TypeContext::TypeConstraint,
                    None,
                    writer,
                )?;
                self.format_expression_tail(tail, writer)?;
                self.format_parameter_bound(default, TypeContext::Root, None, writer)
            }
        }
    }

    fn format_parameter_bound(
        &mut self,
        bound: ParameterBound<'ast>,
        context: TypeContext,
        tail: Option<TypeBoundaryTail<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let (role, before, after) = match bound.boundary {
            TypeBoundary {
                role,
                layout: super::TypeOperatorLayout::Preceding { packet, after },
                ..
            } => {
                self.format_preceding_type_packet(&packet, writer)?;
                (
                    role,
                    CommentBoundary::adjacent(packet.comments[0].span.end),
                    after,
                )
            }
            TypeBoundary {
                layout: super::TypeOperatorLayout::Source(source),
                ..
            } => {
                if let Some(after) = self.prepare_type_rhs_boundary(source, bound.node) {
                    self.format_source_gap(source.prefix, writer)?;
                    let comments =
                        self.prepare_bound_comments(&after, bound.node.node.span(self.ast));
                    return self.format_parameter_bound_after_operator(
                        bound.node, context, &after, comments, tail, writer,
                    );
                }
                let pending =
                    self.format_source_type_operator_rhs(source, bound.node, context, writer)?;
                if let Some(tail) = tail {
                    self.format_expression_tail(pending, writer)?;
                    self.format_type_boundary_tail(tail, writer)?;
                    return Ok(None);
                }
                return Ok(pending);
            }
            TypeBoundary {
                role,
                layout: super::TypeOperatorLayout::Suppressed { before, head },
                ..
            } => {
                self.format_formatted_type_boundary_operator(role, &before, writer)?;
                let (opening, layout) = if head.own_line {
                    (LineConstraint::NewLine(1), TypeLayout::CallerIndented)
                } else {
                    (LineConstraint::Unconstrained, TypeLayout::Root)
                };
                let group = self.format_parameter_bound_separator(
                    None,
                    CommentGap::Source(opening),
                    writer,
                )?;
                let pending = writer.indent_if_break(group, |writer| {
                    self.format_suppressed_type_content(bound.node, &head, context, layout, writer)
                })?;
                if let Some(tail) = tail {
                    self.format_expression_tail(pending, writer)?;
                    self.format_type_boundary_tail(tail, writer)?;
                    return Ok(None);
                }
                return Ok(pending);
            }
            TypeBoundary {
                role,
                layout: super::TypeOperatorLayout::Formatted { before, after },
                ..
            } => (role, before, after),
        };
        let comments = if after.is_empty() {
            PreparedBoundaryComments::Preserved(after.gap())
        } else {
            self.prepare_bound_comments(&after, bound.node.node.span(self.ast))
        };
        self.format_formatted_type_boundary_operator(role, &before, writer)?;
        self.format_parameter_bound_after_operator(
            bound.node, context, &after, comments, tail, writer,
        )
    }

    /// Resolve and emit the final bound head without transporting an ordinary RHS.
    fn format_parameter_bound_after_operator(
        &mut self,
        node: PreparedType,
        context: TypeContext,
        after: &CommentBoundary<'ast>,
        comments: PreparedBoundaryComments<'ast>,
        tail: Option<TypeBoundaryTail<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let (gap, content) = 'prepared: {
            if let Some(projection) = node.suppressed_projection()
                && let Some(head) = self.projected_suppressed_type(projection)
                && !head.comments.is_empty()
            {
                // Record the live source frontier before the separator can claim it.
                let recorded = self.record_type_head(
                    super::compound::PreparedTypeContent::Type(node),
                    PendingTypeHead::new(after.comments(), node.node.span(self.ast).start, None)
                        .with_opening(CommentOpening::Provided),
                    context,
                    writer,
                )?;
                break 'prepared (
                    CommentGap::Source(LineConstraint::NewLine(1)),
                    PreparedBoundContent::Head { recorded, tail },
                );
            }
            let (gap, record_head) = match comments {
                PreparedBoundaryComments::Head { .. }
                    if node.source_owner.is_none()
                        && !self.diagnostic_range_is_sensitive(Span::new(
                            after.source_end(),
                            node.node.span(self.ast).end,
                        )) =>
                {
                    // This complete, unbound head enters the actual RHS syntax.
                    // Its opening is paid by that prepared head, not by its old slot.
                    (CommentGap::Source(LineConstraint::Unconstrained), true)
                }
                PreparedBoundaryComments::Head { gap }
                | PreparedBoundaryComments::Preserved(gap)
                | PreparedBoundaryComments::Trailing { gap, .. } => (gap, false),
            };
            // An empty external packet can still expose a head through removable
            // frames. Discover that frontier before choosing the ordinary direct path.
            let leading = if !record_head && tail.is_none() {
                // A transferable head needs an unclaimed comment inside this type.
                let leading = if self.comments.in_range(node.node.span(self.ast)).is_empty() {
                    None
                } else {
                    self.type_movable_head(node, context)
                };
                if leading.is_none() {
                    let group = self.format_parameter_bound_separator(Some(after), gap, writer)?;
                    return writer.indent_if_break(group, |writer| {
                        self.format_prepared_type_with_layout(
                            node,
                            context,
                            TypeLayout::Root,
                            writer,
                        )
                    });
                }
                leading
            } else {
                None
            };
            let body = self.prepare_operator_rhs_type(node, context)?;
            let (movable, leading) = match (&body.content, leading) {
                (super::compound::PreparedTypeContent::Type(_), Some(leading)) => {
                    (true, Some(leading))
                }
                _ => self.prepared_content_head(&body.content, context),
            };
            let content = if record_head || movable && after.is_empty() {
                PreparedBoundContent::Head {
                    recorded: self.record_type_head(
                        body.content,
                        PendingTypeHead::new(after.comments(), body.span.start, leading),
                        context,
                        writer,
                    )?,
                    tail,
                }
            } else {
                PreparedBoundContent::Body { body, tail }
            };
            (gap, content)
        };
        let after = match &content {
            PreparedBoundContent::Head { .. } => None,
            PreparedBoundContent::Body { .. } => Some(after),
        };
        let group = self.format_parameter_bound_separator(after, gap, writer)?;
        writer.indent_if_break(group, |writer| match content {
            PreparedBoundContent::Head { recorded, tail } => self
                .format_recorded_type_head_with_boundary_tail(
                    recorded,
                    context,
                    TypeLayout::Root,
                    tail,
                    writer,
                ),
            PreparedBoundContent::Body {
                body,
                tail: Some(tail),
            } => self.format_prepared_delimited_type_before_boundary(
                body,
                context,
                TypeLayout::Root,
                tail,
                writer,
            ),
            PreparedBoundContent::Body { body, tail: None } => {
                self.format_prepared_delimited_type(body, context, TypeLayout::Root, writer)
            }
        })
    }

    fn format_parameter_bound_separator(
        &mut self,
        after: Option<&CommentBoundary<'ast>>,
        gap: CommentGap,
        writer: &mut Writer<'ast>,
    ) -> Result<GroupId, FormatError> {
        if let Some(after) = after
            && !after.is_empty()
        {
            writer.indent(|writer| self.format_comment_boundary(after, writer))?;
        }
        let group = writer.named_group(false, |writer| {
            writer.indent(|writer| gap.write(writer, FlatSeparator::Space, Writer::line));
        });
        // A preceding boundary tail cannot remain queued across this RHS head.
        writer.line_suffix_boundary();
        Ok(group)
    }

    fn format_parameter_prefix(
        &mut self,
        prefix: PreparedParameterPrefix<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let (modifiers, name_leading) = match prefix {
            PreparedParameterPrefix::Source(span) => return self.format_verbatim(span, writer),
            PreparedParameterPrefix::Canonical {
                modifiers,
                name_leading,
            } => (modifiers, name_leading),
        };
        if modifiers.r#const {
            writer.text("const");
            if modifiers.r#in || modifiers.out {
                writer.space();
            }
        }
        if modifiers.r#in {
            writer.text("in");
            if modifiers.out {
                writer.space();
            }
        }
        if modifiers.out {
            writer.text("out");
        }
        if let Some(boundary) = name_leading {
            self.format_type_comment_gap(&boundary, FlatSeparator::Space, writer)?;
        } else if !modifiers.is_empty() {
            writer.space();
        }
        Ok(())
    }

    pub(in crate::formatter) fn call_type_arguments_are_compact(
        &self,
        node: TsTypeParameterInstantiation,
    ) -> bool {
        let ast = self.ast;
        let parameters = node.params(ast);
        if parameters.len() != 1 {
            return false;
        }
        let Some(first) = parameters.first() else {
            return false;
        };
        let first = ast.get_node_in_sub_range(first);
        if matches!(
            ast.ts_type_data(first),
            TsTypeData::TsUnionType(_)
                | TsTypeData::TsIntersectionType(_)
                | TsTypeData::TsTypeLiteral(_)
        ) {
            return false;
        }
        let mut breaks = RequiredTypeBreak {
            formatter: self,
            required: false,
        };
        first.visit_with(&mut breaks);
        !breaks.required
    }
}
