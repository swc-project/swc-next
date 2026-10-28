//! Borrowed TypeScript types, aliases, and binding annotation boundaries.

pub(super) mod callable;
mod compound;
mod conditional;
mod context;
pub(super) mod generics;
mod imports;
mod infer;
mod interfaces;
mod mapped;
pub(super) mod members;
mod postfix;
mod templates;
mod tuple;

use std::num::NonZeroU32;

use self::compound::{PendingTypeHead, PreparedTypeContent};
pub(super) use self::context::TypeContext;
use self::context::TypeEnclosure;
pub(super) use self::generics::TypeArgumentOwner;

use super::Formatter;
use super::bindings::{BindingGroup, BindingInitializerOwner, DefiniteMarker, InitializerLayout};
use super::comments::{
    CommentBinding, CommentBoundary, CommentGap, CommentOpening, CommentRunLayout, ExpressionTail,
    PreparedTrailingRun,
};
use super::diagnostics::{FlatSeparator, LineConstraint};
use super::expressions::assignment::RhsLayout;
use super::expressions::{CopiedRawOwner, ExpressionOutput, ExpressionRightEdge, TailOwner};
use super::literals::StringContext;
use super::source::is_source_whitespace;
use super::statements::DeclarationKeyword;
use super::suppression::{ContentMode, SuppressionPosition, is_suppression_comment};
use super::variables::{VariableContext, VariableDeclaratorOutput};
use crate::{
    FormatError,
    document::{Fragment, Writer},
};
use swc_next_ecma_ast::{
    Ast, BindingIdentifier, BindingPatternData, Comment, CommentKind, ExprData, GetSpan,
    IdentifierReference, Span, TsLiteralData, TsParenthesizedType, TsQualifiedName,
    TsQualifiedNameLeft, TsQualifiedNameLeftData, TsType, TsTypeAliasDeclaration, TsTypeAnnotation,
    TsTypeData, TsTypeName, TsTypeNameData, UnaryOperator, VariableDeclarator,
};

#[derive(Clone, Copy)]
enum TypeBoundaryRole {
    Alias,
    Qualified,
    Constraint(LineConstraint),
    Default,
}
enum TypeBoundaryTarget<'a> {
    ModuleReference,
    Alias(&'a mut PreparedType),
    Qualified,
    Constraint {
        name: BindingIdentifier,
        node: &'a mut PreparedType,
    },
    InferConstraint {
        name: BindingIdentifier,
        node: &'a mut PreparedType,
    },
    Default {
        name: Option<BindingIdentifier>,
        node: &'a mut PreparedType,
    },
}

#[derive(Clone, Copy)]
pub(super) struct PreparedType {
    pub(super) node: TsType,
    pub(super) data: TsTypeData,
    pub(super) source_owner: Option<TsType>,
    /// The attachment frame retained independently of an ignored content owner.
    pub(super) source_head: Option<TsType>,
    suppressed_projection: Option<SuppressedTypeProjection>,
}

#[derive(Clone, Copy)]
struct SuppressedTypeProjection {
    source: TsType,
    owner: TsType,
    retained_head: Option<TsType>,
    surviving_head: TsType,
    /// One-based start of the original, still unclaimed outer packet.
    packet_start: Option<NonZeroU32>,
}

impl PreparedType {
    fn suppressed_projection(&self) -> Option<&SuppressedTypeProjection> {
        let projection = self.suppressed_projection.as_ref()?;
        (projection.source == self.node
            && Some(projection.owner) == self.source_owner
            && projection.retained_head == self.source_head)
            .then_some(projection)
    }
}

struct ProjectedSuppressedType<'ast> {
    node: PreparedType,
    comments: &'ast [Comment],
}

/// The actual operator carries its grammar constraint for a preceding packet.
#[derive(Clone, Copy)]
pub(super) struct TypeOperatorSlot {
    span: Span,
    preceding: LineConstraint,
}

impl TypeOperatorSlot {
    pub(super) fn new(span: Span, preceding: LineConstraint) -> Self {
        Self { span, preceding }
    }
}

/// Source punctuation and its RHS keep independent indentation destinations.
#[derive(Clone, Copy)]
pub(super) struct SourceTypeOperator {
    pub(super) prefix: Span,
    pub(super) after: Span,
}

impl SourceTypeOperator {
    pub(super) fn new(gap: Span, operator: Span) -> Self {
        Self {
            prefix: Span::new(gap.start, operator.end),
            after: Span::new(operator.end, gap.end),
        }
    }

    fn span(self) -> Span {
        Span::new(self.prefix.start, self.after.end)
    }
}

#[derive(Clone, Copy)]
pub(super) enum TypeSuppressionContext {
    TypeBoundary,
    ExpressionBoundary,
}

#[derive(Clone, Copy)]
pub(super) enum TypeSuppressionTarget {
    Neither,
    Preceding,
    Following,
    Both,
}

impl TypeSuppressionTarget {
    pub(super) fn preceding(self) -> bool {
        matches!(self, Self::Preceding | Self::Both)
    }

    pub(super) fn following(self) -> bool {
        matches!(self, Self::Following | Self::Both)
    }
}

#[derive(Clone, Copy)]
pub(super) enum TypeHeadRetention {
    Unchanged,
    Source(TsType),
}

impl TypeHeadRetention {
    fn apply(self, node: &mut PreparedType, ast: &Ast<'_>) {
        if let Self::Source(owner) = self {
            node.source_head = Some(merge_type_source_owner(
                node.source_head,
                owner,
                node.node,
                ast,
            ));
        }
    }
}

pub(super) struct TypeAttachmentHead {
    node: TsType,
    union: bool,
    source_barrier: Option<TsType>,
    exterior_union_frame: Option<TsType>,
    interior_union_frame: Option<TsType>,
    prefix: TypeCommentPrefix,
    first_owner: std::cell::Cell<Option<TsType>>,
    prefix_carrier: Option<TsType>,
    parentheses_only: bool,
}

#[derive(Clone, Copy)]
struct TypeCommentPrefix {
    tie: TypeCommentTie,
    source_leads_head: bool,
    source_lines_adjacency: bool,
    preserves_union_owner: bool,
    preserves_horizontal_owner: bool,
}

#[derive(Clone, Copy)]
struct TypeCommentTie {
    next: u32,
    type_leading: bool,
    expression_leading: bool,
    union_adjacency: bool,
}

pub(super) struct TypeSuppressionRun {
    target: TypeSuppressionTarget,
    head: TypeHeadRetention,
    packet_projection: Option<PrecedingTypePacketProjection>,
    head_projection: Option<PrecedingTypeHeadProjection>,
    // A contiguous source-owner prefix, independent of the RHS frame projection.
    preceding_count: Option<usize>,
}

/// Original run selection plus a candidate partition; neither claims comments.
pub(super) struct ResolvedTypeSuppressionRun {
    pub(super) selection: TypeSuppressionRun,
    pub(super) preceding_prefix: Option<PrecedingTypePrefix>,
}

/// A source prefix and suffix have one transition from preceding to following.
/// Attachment and source-window guards still belong to the operator constructor.
pub(super) struct PrecedingTypePrefix {
    end: usize,
}

#[derive(Clone, Copy)]
enum PrecedingTypePacketProjection {
    /// Every component precedes and the type prefix keeps its ordinary owner.
    Inline,
    /// A physical trailing run still needs its operator's placement contract.
    EndOfLine(TypeSuppressionContext),
}

/// Every component in the retained head's attachment view precedes the operator.
/// The source partition must independently cover the same complete packet.
struct PrecedingTypeHeadProjection;

impl TypeSuppressionRun {
    pub(super) fn target(&self) -> TypeSuppressionTarget {
        self.target
    }

    /// Retain an attachment frame independently of the selected content owner.
    pub(super) fn retain_source_head(&self, node: &mut PreparedType, ast: &Ast<'_>) {
        self.head.apply(node, ast);
    }
}

fn merge_type_source_owner(
    previous: Option<TsType>,
    owner: TsType,
    enclosing: TsType,
    ast: &Ast<'_>,
) -> TsType {
    let Some(previous) = previous else {
        return owner;
    };
    let old_span = previous.span(ast);
    let span = owner.span(ast);
    if old_span.start <= span.start && span.end <= old_span.end {
        previous
    } else if span.start <= old_span.start && old_span.end <= span.end {
        owner
    } else {
        enclosing
    }
}

struct PreparedTypeParentheses<'ast> {
    span: Span,
    child_span: Span,
    child: PreparedType,
    retained: bool,
    movable: bool,
    leading_comments: &'ast [Comment],
    trailing: CommentBoundary<'ast>,
    trailing_unbound: bool,
    source: bool,
}

/// Only an unbound caller packet or an escaped type packet can supply this opening.
pub(super) struct BinaryTypeHead<'ast>(BinaryTypeHeadLayout<'ast>);

enum BinaryTypeHeadLayout<'ast> {
    Direct,
    /// Block packets with no separator breaks need no transferable type frontier.
    InlineNamed,
    /// A named RHS owns its complete block head without a continuation scope.
    Named,
    /// An atomic name can hand its ordinary operator-line packet to its caller.
    TrailingNamed {
        name: IdentifierReference,
        tail: ExpressionTail<'ast>,
    },
    Caller {
        leading: Option<compound::MovableTypeHead<'ast>>,
        own_line: Option<LineConstraint>,
    },
}

pub(super) struct RecordedTypeHead<'ast> {
    head: Fragment<'ast>,
    content: PreparedTypeContent<'ast>,
    riding: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum TypeLayout {
    Root,
    OpeningProvided,
    CallerIndented,
    AliasRhs,
    SignatureParameter,
    MultipleTupleElement,
    ConditionalConsequent,
    ConditionalAlternate,
}

/// A complete leading packet whose ignored owner survives either line layout.
/// The caller formats its operator and indentation; the selected raw type stays separate.
pub(super) struct SuppressedTypeHead<'ast> {
    comments: &'ast [Comment],
    next_start: u32,
    own_line: bool,
}

#[derive(Clone, Copy)]
pub(super) enum SuppressedTypeHeadRole {
    TypeBoundary,
    ExpressionBoundary,
    ConditionalExtends,
    MappedBoundary,
}

/// A complete block packet whose preceding owner and placement have been proved.
pub(super) struct PreparedPrecedingTypePacket<'ast> {
    comments: &'ast [Comment],
}

enum TypeBoundaryLayout<'ast> {
    Source(Span),
    Formatted {
        before: CommentBoundary<'ast>,
        after: CommentBoundary<'ast>,
    },
}

/// Operator boundaries can separate a proven suppressed head from source geometry.
/// Other type punctuation still carries an ordinary or source-bound gap.
enum TypeOperatorLayout<'ast> {
    Source(SourceTypeOperator),
    Preceding {
        packet: PreparedPrecedingTypePacket<'ast>,
        after: CommentBoundary<'ast>,
    },
    Suppressed {
        before: CommentBoundary<'ast>,
        head: SuppressedTypeHead<'ast>,
    },
    Formatted {
        before: CommentBoundary<'ast>,
        after: CommentBoundary<'ast>,
    },
}
pub(super) struct TypeBoundary<'ast> {
    role: TypeBoundaryRole,
    pub(super) left_mode: ContentMode,
    pub(super) right_mode: ContentMode,
    layout: TypeOperatorLayout<'ast>,
}

pub(super) enum TypeGap<'ast> {
    Source(Span),
    Formatted(CommentBoundary<'ast>),
}
pub(super) struct PreparedAnnotation<'ast> {
    content: AnnotationContent,
    prefix: TypeBoundaryLayout<'ast>,
}

pub(super) struct PreparedDelimitedType<'ast> {
    span: Span,
    content: PreparedTypeContent<'ast>,
}

/// A delimited type either owns its prepared head or keeps the caller's boundary.
pub(super) enum PreparedDelimitedTypeHead<'ast> {
    Recorded(RecordedTypeHead<'ast>),
    Boundary(PreparedDelimitedType<'ast>, CommentBoundary<'ast>),
}
impl PreparedDelimitedType<'_> {
    pub(super) fn starts_with_type_parameters(&self, ast: &Ast<'_>) -> bool {
        matches!(&self.content, PreparedTypeContent::Type(node)
            if matches!(node.data, TsTypeData::TsFunctionType(function)
                if function.type_parameters(ast).is_some()))
    }

    pub(super) fn is_object_type(&self) -> bool {
        self.content.is_object_type()
    }
}

struct TypeBoundaryTail<'ast> {
    role: TypeBoundaryRole,
    before: CommentBoundary<'ast>,
    trailing: PreparedTrailingRun<'ast>,
}

pub(super) struct PreparedBindingAnnotation<'ast> {
    leading: &'ast [Comment],
    before_leading: LineConstraint,
    leading_source_gap: Option<LineConstraint>,
    colon: u32,
    prefix: BindingAnnotationPrefix<'ast>,
    content: AnnotationContent,
    type_argument_owner: TypeArgumentOwner,
}
enum BindingAnnotationPrefix<'ast> {
    Source(Span),
    Formatted {
        colon: Span,
        before: LineConstraint,
        comments: &'ast [Comment],
        end: u32,
    },
}
struct AnnotationGeometry<'ast> {
    gap: Span,
    colon: Span,
    before: &'ast [Comment],
    after: &'ast [Comment],
    before_mode: ContentMode,
}
enum AnnotationContent {
    Source(TsTypeAnnotation),
    Formatted(PreparedType),
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_annotation_content(
        &mut self,
        content: PreparedAnnotation<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_annotation_content_with_layout(content, TypeLayout::Root, writer)
    }

    fn prepare_annotation_content(
        &self,
        preceding_end: u32,
        annotation: TsTypeAnnotation,
    ) -> Result<PreparedAnnotation<'ast>, FormatError> {
        self.prepare_annotation_content_mode(preceding_end, annotation, ContentMode::Formatted)
    }

    pub(super) fn prepare_annotation_content_mode(
        &self,
        preceding_end: u32,
        annotation: TsTypeAnnotation,
        mode: ContentMode,
    ) -> Result<PreparedAnnotation<'ast>, FormatError> {
        let ast = self.ast;
        if mode == ContentMode::Verbatim {
            return Ok(PreparedAnnotation {
                content: AnnotationContent::Source(annotation),
                prefix: TypeBoundaryLayout::Source(Span::new(
                    preceding_end,
                    annotation.span(ast).start,
                )),
            });
        }
        let node = annotation.type_annotation(ast);
        let gap = Span::new(preceding_end, node.span(ast).start);
        let comments = self.comments.in_range(gap);
        let Some(colon_start) = self.find_token(gap, comments, ":") else {
            return Err(self.unsupported("type annotation colon", annotation));
        };
        let colon = Span::new(colon_start, colon_start + 1);
        let geometry = self.annotation_geometry(gap, colon, comments);
        let content = if geometry.before_mode == ContentMode::Verbatim {
            AnnotationContent::Source(annotation)
        } else {
            let mut prepared = self.prepare_type(node, ContentMode::Formatted);
            self.select_type_head_comments(&mut prepared, geometry.after, geometry.gap.end);
            AnnotationContent::Formatted(prepared)
        };
        let prefix =
            match self.annotation_source_prefix(&content, geometry.gap.end, geometry.gap.start) {
                Some(span) => TypeBoundaryLayout::Source(span),
                None => TypeBoundaryLayout::Formatted {
                    before: self.prepare_comment_boundary(
                        geometry.before,
                        geometry.gap.start,
                        geometry.colon.start,
                    ),
                    after: self.prepare_comment_boundary(
                        geometry.after,
                        geometry.colon.end,
                        geometry.gap.end,
                    ),
                },
            };
        Ok(PreparedAnnotation { content, prefix })
    }

    fn format_annotation_content_with_layout(
        &mut self,
        content: PreparedAnnotation<'ast>,
        mut layout: TypeLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        match content.prefix {
            TypeBoundaryLayout::Source(gap) => {
                self.format_source_gap(gap, writer)?;
                if layout == TypeLayout::Root {
                    layout = TypeLayout::OpeningProvided;
                }
            }
            TypeBoundaryLayout::Formatted { before, after } => {
                self.format_comment_boundary(&before, writer)?;
                before.gap().write(writer, FlatSeparator::Empty, |_| {});
                writer.text(":");
                if let AnnotationContent::Formatted(node) = content.content {
                    return self.format_prepared_type_after_comments(
                        node,
                        TypeContext::Root,
                        &after,
                        layout,
                        writer,
                    );
                }
                self.format_comment_boundary(&after, writer)?;
                after
                    .gap()
                    .write(writer, FlatSeparator::Space, Writer::space);
            }
        }
        self.format_annotation_type_content_with_layout(content.content, layout, writer)
    }

    fn format_prepared_parentheses_after_comments(
        &mut self,
        parentheses: PreparedTypeParentheses<'ast>,
        after: &CommentBoundary<'ast>,
        context: TypeContext,
        layout: TypeLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        if parentheses.movable
            && !context.preserves_union_head(parentheses.child.node, self.ast)
            && matches!(
                self.comment_boundary_binding(after, parentheses.span),
                CommentBinding::Unbound
            )
        {
            writer.space();
            let start = parentheses.span.start;
            let recorded = self.record_type_head(
                compound::PreparedTypeContent::Parentheses(parentheses),
                PendingTypeHead::new(after.comments(), start, None),
                context,
                writer,
            )?;
            return self.format_recorded_type_head(recorded, context, layout, writer);
        }
        self.format_type_comment_gap(after, FlatSeparator::Space, writer)?;
        self.format_prepared_type_parentheses(parentheses, context, layout, writer)
    }

    pub(super) fn format_prepared_type_after_comments(
        &mut self,
        node: PreparedType,
        context: TypeContext,
        after: &CommentBoundary<'ast>,
        layout: TypeLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let compound_head = matches!(
            node.data,
            TsTypeData::TsParenthesizedType(_)
                | TsTypeData::TsUnionType(_)
                | TsTypeData::TsIntersectionType(_)
                | TsTypeData::TsArrayType(_)
                | TsTypeData::TsIndexedAccessType(_)
        );
        let leading_head = if compound_head {
            self.type_movable_head(node, context)
        } else {
            None
        };
        if node.source_owner.is_none()
            && compound_head
            && (!after.is_empty() || leading_head.is_some())
            && matches!(
                self.comment_boundary_binding(after, node.node.span(self.ast)),
                CommentBinding::Unbound
            )
        {
            writer.space();
            return self.format_prepared_type_with_caller_head(
                node,
                leading_head,
                context,
                after,
                layout,
                writer,
            );
        }
        self.format_type_comment_gap(after, FlatSeparator::Space, writer)?;
        self.format_prepared_type_with_layout(node, context, layout, writer)
    }

    pub(super) fn format_type_gap(
        &mut self,
        gap: TypeGap<'ast>,
        separator: FlatSeparator,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        match gap {
            TypeGap::Source(span) => self.format_verbatim(span, writer),
            TypeGap::Formatted(boundary) => {
                self.format_type_comment_gap(&boundary, separator, writer)
            }
        }
    }

    pub(super) fn prepare_type_gap(&self, gap: Span, source: bool) -> TypeGap<'ast> {
        if source {
            return TypeGap::Source(gap);
        }
        let comments = self.comments.in_range(gap);
        TypeGap::Formatted(self.prepare_comment_boundary(comments, gap.start, gap.end))
    }

    fn binding_type_content(&self, mut node: TsType) -> TsTypeData {
        loop {
            let data = self.ast.ts_type_data(node);
            let members = match data {
                TsTypeData::TsParenthesizedType(parentheses) => {
                    node = parentheses.type_annotation(self.ast);
                    continue;
                }
                TsTypeData::TsUnionType(union) => union.types(self.ast),
                TsTypeData::TsIntersectionType(intersection) => intersection.types(self.ast),
                _ => return data,
            };
            if members.len() != 1 {
                return data;
            }
            let Some(member) = members.first() else {
                return data;
            };
            node = self.ast.get_node_in_sub_range(member);
        }
    }

    pub(super) fn binding_annotation_breaks_left(&self, annotation: TsTypeAnnotation) -> bool {
        let ast = self.ast;
        let TsTypeData::TsTypeReference(reference) =
            self.binding_type_content(annotation.type_annotation(ast))
        else {
            return false;
        };
        let Some(arguments) = reference.type_arguments(ast) else {
            return false;
        };
        let params = arguments.params(ast);
        params.len() > 1
            && params.iter().any(|id| {
                let argument = ast.get_node_in_sub_range(id);
                match self.binding_type_content(argument) {
                    TsTypeData::TsConditionalType(_) => true,
                    TsTypeData::TsTypeReference(reference) => reference
                        .type_arguments(ast)
                        .is_some_and(|arguments| !arguments.params(ast).is_empty()),
                    _ => false,
                }
            })
    }

    fn format_annotation_type_content_with_layout(
        &mut self,
        content: AnnotationContent,
        layout: TypeLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        match content {
            AnnotationContent::Source(annotation) => {
                self.format_verbatim(annotation.span(self.ast), writer)?;
                Ok(None)
            }
            AnnotationContent::Formatted(type_node) => {
                self.format_prepared_type_with_layout(type_node, TypeContext::Root, layout, writer)
            }
        }
    }

    fn format_binding_annotation_root(
        &mut self,
        content: compound::PreparedTypeContent<'ast>,
        owner: TypeArgumentOwner,
        layout: TypeLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        if let compound::PreparedTypeContent::Type(node) = &content
            && node.source_owner.is_none()
            && let TsTypeData::TsTypeReference(reference) = node.data
        {
            return self.format_type_reference_with_owner(reference, owner, writer);
        }
        self.format_prepared_type_content(content, TypeContext::Root, layout, writer)
    }

    fn format_recorded_binding_annotation_head(
        &mut self,
        recorded: RecordedTypeHead<'ast>,
        owner: TypeArgumentOwner,
        layout: TypeLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        if matches!(&recorded.content, compound::PreparedTypeContent::Type(node)
            if node.source_owner.is_none() && matches!(node.data, TsTypeData::TsTypeReference(_)))
        {
            return writer.group(false, |writer| {
                writer.write_fragment(recorded.head);
                self.format_binding_annotation_root(recorded.content, owner, layout, writer)
            });
        }
        self.format_recorded_type_head(recorded, TypeContext::Root, layout, writer)
    }

    fn format_binding_annotation_type_content(
        &mut self,
        content: AnnotationContent,
        owner: TypeArgumentOwner,
        layout: TypeLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        if owner == TypeArgumentOwner::ArrowVariableAnnotation
            && let AnnotationContent::Formatted(node) = content
            && node.source_owner.is_none()
            && match node.data {
                TsTypeData::TsTypeReference(_) | TsTypeData::TsParenthesizedType(_) => true,
                TsTypeData::TsUnionType(union) => union.types(self.ast).len() == 1,
                TsTypeData::TsIntersectionType(intersection) => {
                    intersection.types(self.ast).len() == 1
                }
                _ => false,
            }
        {
            let content = if matches!(node.data, TsTypeData::TsTypeReference(_)) {
                compound::PreparedTypeContent::Type(node)
            } else {
                self.format_movable_type_head(
                    node,
                    &mut PendingTypeHead::new(&[], node.node.span(self.ast).start, None),
                    TypeContext::Root,
                    writer,
                )?
            };
            return self.format_binding_annotation_root(content, owner, layout, writer);
        }
        self.format_annotation_type_content_with_layout(content, layout, writer)
    }

    fn format_type_content_with_boundary_tail(
        &mut self,
        content: compound::PreparedTypeContent<'ast>,
        head: Option<Fragment<'ast>>,
        context: TypeContext,
        layout: TypeLayout,
        tail: Option<TypeBoundaryTail<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        if let compound::PreparedTypeContent::Union(union) = content {
            return match tail {
                Some(tail) => self.format_prepared_union_before_boundary(
                    union, context, layout, head, tail, writer,
                ),
                None => self.format_prepared_union(union, context, layout, head, writer),
            };
        }
        if let Some(head) = head {
            writer.write_fragment(head);
        }
        let pending = self.format_prepared_type_content(content, context, layout, writer)?;
        if let Some(tail) = tail {
            self.format_expression_tail(pending, writer)?;
            self.format_type_boundary_tail(tail, writer)?;
            return Ok(None);
        }
        Ok(pending)
    }

    fn format_recorded_type_head_with_boundary_tail(
        &mut self,
        recorded: RecordedTypeHead<'ast>,
        context: TypeContext,
        layout: TypeLayout,
        tail: Option<TypeBoundaryTail<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_type_content_with_boundary_tail(
            recorded.content,
            Some(recorded.head),
            context,
            layout,
            tail,
            writer,
        )
    }

    fn format_prepared_delimited_type_before_boundary(
        &mut self,
        body: PreparedDelimitedType<'ast>,
        context: TypeContext,
        layout: TypeLayout,
        tail: TypeBoundaryTail<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let content = match body.content {
            compound::PreparedTypeContent::Compound(single) if single.movable() => {
                self.format_single_compound_head(single, context, writer)?
            }
            content => content,
        };
        self.format_type_content_with_boundary_tail(
            content,
            None,
            context,
            layout,
            Some(tail),
            writer,
        )
    }

    fn format_prepared_type_content_output(
        &mut self,
        content: PreparedTypeContent<'ast>,
        context: TypeContext,
        layout: TypeLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<ExpressionOutput<'ast>, FormatError> {
        match content {
            PreparedTypeContent::Type(node) => {
                self.format_prepared_type_output(node, context, layout, writer)
            }
            content => self
                .format_prepared_type_content(content, context, layout, writer)
                .map(ExpressionOutput::unreported),
        }
    }

    pub(super) fn format_prepared_delimited_type(
        &mut self,
        body: PreparedDelimitedType<'ast>,
        context: TypeContext,
        layout: TypeLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_prepared_delimited_type_output(body, context, layout, writer)
            .map(|output| output.pending)
    }

    pub(super) fn format_prepared_delimited_type_output(
        &mut self,
        body: PreparedDelimitedType<'ast>,
        context: TypeContext,
        layout: TypeLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<ExpressionOutput<'ast>, FormatError> {
        let content = match body.content {
            compound::PreparedTypeContent::Compound(single) if single.movable() => {
                self.format_single_compound_head(single, context, writer)?
            }
            content => content,
        };
        self.format_prepared_type_content_output(content, context, layout, writer)
    }

    pub(super) fn prepare_delimited_type(
        &self,
        mut node: PreparedType,
    ) -> Result<PreparedDelimitedType<'ast>, FormatError> {
        let span = node.node.span(self.ast);
        let content = loop {
            if node.source_owner.is_some() || node.source_head.is_some() {
                break compound::PreparedTypeContent::Type(node);
            }
            match node.data {
                TsTypeData::TsParenthesizedType(parentheses) => {
                    let parentheses =
                        self.prepare_type_parentheses(parentheses, None, None, TypeContext::Root);
                    if parentheses.retained {
                        break compound::PreparedTypeContent::Parentheses(parentheses);
                    }
                    node = parentheses.child;
                }
                TsTypeData::TsUnionType(union) => {
                    let types = union.types(self.ast);
                    if types.len() > 1 {
                        break compound::PreparedTypeContent::Union(
                            self.prepare_union(node, types)?,
                        );
                    }
                    match self.prepare_single_compound(node)?.into_headless_type() {
                        Ok(child) => node = child,
                        Err(single) => break compound::PreparedTypeContent::Compound(single),
                    }
                }
                TsTypeData::TsIntersectionType(intersection)
                    if intersection.types(self.ast).len() == 1 =>
                {
                    match self.prepare_single_compound(node)?.into_headless_type() {
                        Ok(child) => node = child,
                        Err(single) => break compound::PreparedTypeContent::Compound(single),
                    }
                }
                _ => break compound::PreparedTypeContent::Type(node),
            }
        };
        Ok(PreparedDelimitedType { span, content })
    }

    /// Retaining a lexical head does not change the syntax printer's layout scope.
    /// Materialize that same syntax once, without releasing its source owner.
    fn prepare_operator_rhs_type(
        &self,
        node: PreparedType,
        context: TypeContext,
    ) -> Result<PreparedDelimitedType<'ast>, FormatError> {
        let mut body = self.prepare_delimited_type(node)?;
        let PreparedTypeContent::Type(node) = body.content else {
            return Ok(body);
        };
        if node.source_owner.is_some() || node.source_head.is_none() {
            return Ok(body);
        }
        body.content = match node.data {
            TsTypeData::TsParenthesizedType(parentheses) => PreparedTypeContent::Parentheses(
                self.prepare_type_parentheses(parentheses, None, node.source_head, context),
            ),
            TsTypeData::TsUnionType(union) if union.types(self.ast).len() > 1 => {
                PreparedTypeContent::Union(self.prepare_union(node, union.types(self.ast))?)
            }
            TsTypeData::TsUnionType(_) => {
                PreparedTypeContent::Compound(self.prepare_single_compound(node)?)
            }
            TsTypeData::TsIntersectionType(intersection)
                if intersection.types(self.ast).len() > 1 =>
            {
                PreparedTypeContent::Intersection(
                    self.prepare_intersection(node, intersection.types(self.ast))?,
                )
            }
            TsTypeData::TsIntersectionType(_) => {
                PreparedTypeContent::Compound(self.prepare_single_compound(node)?)
            }
            _ => PreparedTypeContent::Type(node),
        };
        Ok(body)
    }

    fn format_type_boundary_tail(
        &mut self,
        tail: TypeBoundaryTail<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        self.format_formatted_type_boundary_operator(tail.role, &tail.before, writer)?;
        self.claim_prepared_trailing_run(tail.trailing, writer)
    }

    fn format_type_boundary_operator<'boundary>(
        &mut self,
        boundary: &'boundary TypeBoundary<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<(&'boundary CommentBoundary<'ast>, FlatSeparator)>, FormatError> {
        let space = match boundary.role {
            TypeBoundaryRole::Qualified => FlatSeparator::Empty,
            _ => FlatSeparator::Space,
        };
        let (before, after) = match &boundary.layout {
            TypeOperatorLayout::Source(source) => {
                self.format_source_gap(source.span(), writer)?;
                return Ok(None);
            }
            TypeOperatorLayout::Preceding { packet, after } => {
                self.format_preceding_type_packet(packet, writer)?;
                self.format_formatted_type_boundary_operator(
                    boundary.role,
                    &CommentBoundary::adjacent(packet.comments[0].span.end),
                    writer,
                )?;
                return Ok(Some((after, space)));
            }
            TypeOperatorLayout::Suppressed { before, head } => {
                self.format_formatted_type_boundary_operator(boundary.role, before, writer)?;
                writer.space();
                self.format_suppressed_type_head(head, writer)?;
                return Ok(None);
            }
            TypeOperatorLayout::Formatted { before, after } => (before, after),
        };
        self.format_formatted_type_boundary_operator(boundary.role, before, writer)?;
        Ok(Some((after, space)))
    }

    pub(super) fn format_type_alias(
        &mut self,
        alias: TsTypeAliasDeclaration,
        keyword: Option<DeclarationKeyword>,
        writer: &mut Writer<'ast>,
    ) -> Result<(u32, Option<ExpressionTail<'ast>>), FormatError> {
        let ast = self.ast;
        let span = alias.span(ast);
        let identifier = alias.id(ast);
        let identifier_span = identifier.span(ast);
        let annotation = alias.type_annotation(ast);
        let annotation_span = annotation.span(ast);
        let mut type_node = self.prepare_type(annotation, ContentMode::Formatted);
        let keyword_end = keyword.map_or(span.start + "type".len() as u32, |keyword| {
            keyword.name_prefix_start()
        });
        let parameters = alias.type_parameters(ast).map(|node| {
            let prefix = self.prepare_type_list_prefix(
                Span::new(identifier_span.end, node.span(ast).start),
                generics::TypeListKind::Parameters {
                    has_left_owner: true,
                },
            );
            (node, prefix)
        });
        let rhs_layout = parameters
            .as_ref()
            .map_or(RhsLayout::Fluid, |(parameters, _)| {
                let parameters = parameters.params(ast);
                let complex = parameters.len() > 1
                    && parameters.iter().any(|parameter| {
                        let parameter = ast.get_node_in_sub_range(parameter);
                        parameter.constraint(ast).is_some() || parameter.default(ast).is_some()
                    });
                if complex {
                    RhsLayout::NeverBreakAfterOperator
                } else {
                    RhsLayout::Fluid
                }
            });
        let head_end = parameters
            .as_ref()
            .map_or(identifier_span.end, |(node, _)| node.span(ast).end);
        let mut name_mode = parameters
            .as_ref()
            .map_or(ContentMode::Formatted, |(_, prefix)| prefix.left_mode);
        if keyword.is_some_and(|keyword| keyword.suppresses_name(identifier)) {
            name_mode = ContentMode::Verbatim;
        }
        if self.has_suppression {
            let header = self
                .comments
                .in_range(Span::new(keyword_end, identifier_span.start));
            if self.content_mode(header) == ContentMode::Verbatim {
                name_mode = ContentMode::Verbatim;
            }
            let tail = self
                .comments
                .in_range(Span::new(annotation_span.end, span.end));
            if self.content_mode(tail) == ContentMode::Verbatim {
                type_node.source_owner = Some(annotation);
            }
        }
        let boundary = self.prepare_type_boundary(
            Span::new(head_end, annotation_span.start),
            TypeBoundaryTarget::Alias(&mut type_node),
        )?;
        if parameters.is_none() && boundary.left_mode == ContentMode::Verbatim {
            name_mode = ContentMode::Verbatim;
        }
        writer.text("type");
        self.format_source_token_gap(keyword_end, identifier_span.start, true, writer)?;
        if name_mode == ContentMode::Verbatim {
            self.format_verbatim(identifier_span, writer)?;
        } else {
            writer.text(ast.get_utf8(identifier.name(ast)));
        }
        if let Some((parameters, ref prefix)) = parameters {
            let mode = if boundary.left_mode == ContentMode::Verbatim {
                ContentMode::Verbatim
            } else {
                prefix.right_mode
            };
            self.format_type_list_prefix(prefix, writer)?;
            self.format_type_parameters(
                parameters,
                generics::TypeParameterContext::TypeDeclaration,
                mode,
                writer,
            )?;
        }
        let trailing_comments = self
            .comments
            .in_range(Span::new(annotation_span.end, span.end));
        let compound_tail = match type_node.data {
            TsTypeData::TsUnionType(node) => node.types(ast).len() > 1,
            TsTypeData::TsIntersectionType(node) => node.types(ast).len() > 1,
            _ => false,
        };
        let trailing = (compound_tail
            && !trailing_comments.is_empty()
            && trailing_comments.iter().all(|comment| {
                comment.kind == CommentKind::Block
                    && self.line_breaks(annotation_span.end, comment.span.start) == 0
            })
            && matches!(
                self.comment_binding(trailing_comments, span, Some(annotation_span)),
                CommentBinding::Unbound
            ))
        .then(|| self.prepare_comment_boundary(trailing_comments, annotation_span.end, span.end));
        let tail = self.format_type_alias_rhs(&boundary, type_node, rhs_layout, writer)?;
        if let Some(trailing) = trailing {
            self.format_expression_tail(tail, writer)?;
            self.format_comment_boundary(&trailing, writer)?;
            Ok((trailing.end(), None))
        } else {
            Ok((annotation_span.end, tail))
        }
    }

    fn format_type_alias_rhs(
        &mut self,
        boundary: &TypeBoundary<'ast>,
        node: PreparedType,
        rhs_layout: RhsLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let packet_before;
        let (before, after) = match &boundary.layout {
            TypeOperatorLayout::Source(source) => {
                self.format_source_gap(source.span(), writer)?;
                return self.format_prepared_type_with_layout(
                    node,
                    TypeContext::Root,
                    TypeLayout::OpeningProvided,
                    writer,
                );
            }
            TypeOperatorLayout::Preceding { packet, after } => {
                self.format_preceding_type_packet(packet, writer)?;
                packet_before = CommentBoundary::adjacent(packet.comments[0].span.end);
                (&packet_before, after)
            }
            TypeOperatorLayout::Suppressed { before, head } => {
                self.format_formatted_type_boundary_operator(boundary.role, before, writer)?;
                if head.own_line {
                    let recorded = self.record_type_head(
                        PreparedTypeContent::Type(node),
                        PendingTypeHead::new(head.comments, head.next_start, None),
                        TypeContext::Root,
                        writer,
                    )?;
                    return self.format_type_alias_head(recorded.head, recorded.content, writer);
                }
                return writer.group(false, |writer| {
                    rhs_layout.write(
                        writer,
                        |writer| match rhs_layout {
                            RhsLayout::NeverBreakAfterOperator => writer.space(),
                            RhsLayout::Fluid | RhsLayout::BreakAfterOperator => writer.line(),
                        },
                        |writer| {
                            self.format_suppressed_type_content(
                                node,
                                head,
                                TypeContext::Root,
                                TypeLayout::OpeningProvided,
                                writer,
                            )
                        },
                    )
                });
            }
            TypeOperatorLayout::Formatted { before, after } => (before, after),
        };
        if node.source_owner == Some(node.node) {
            self.format_type_boundary(boundary, writer)?;
            return self.format_prepared_type(node, TypeContext::Root, writer);
        }
        let node = self.normalize_type_head(node)?;
        if let TsTypeData::TsParenthesizedType(parentheses) = node.data {
            let parentheses = self.prepare_type_parentheses(
                parentheses,
                node.source_owner,
                node.source_head,
                TypeContext::Root,
            );
            if !parentheses.retained {
                return self.format_type_alias_rhs(boundary, parentheses.child, rhs_layout, writer);
            }
            if parentheses.movable {
                self.format_comment_boundary(before, writer)?;
                before
                    .gap()
                    .write(writer, FlatSeparator::Space, Writer::space);
                writer.text("=");
                let (head, content) = writer.record_with_value(|writer| {
                    self.format_comment_boundary_with_layout(
                        after,
                        CommentRunLayout::GroupedLeading(
                            parentheses
                                .leading_comments
                                .first()
                                .map_or(parentheses.child_span.start, |comment| comment.span.start),
                        ),
                        writer,
                    )?;
                    self.format_movable_type_head(
                        parentheses.child,
                        &mut PendingTypeHead::new(
                            parentheses.leading_comments,
                            parentheses.child_span.start,
                            self.type_movable_head(parentheses.child, TypeContext::Root),
                        ),
                        TypeContext::Root,
                        writer,
                    )
                })?;
                return self.format_type_alias_head(head, content, writer);
            }
            self.format_type_boundary(boundary, writer)?;
            return self.format_prepared_type_parentheses(
                parentheses,
                TypeContext::Root,
                TypeLayout::Root,
                writer,
            );
        }
        let single_compound = match node.data {
            TsTypeData::TsUnionType(union) => union.types(self.ast).len() == 1,
            TsTypeData::TsIntersectionType(intersection) => intersection.types(self.ast).len() == 1,
            _ => false,
        };
        if single_compound && node.source_owner.is_none() {
            let single = self.prepare_single_compound(node)?;
            if single.movable() {
                self.format_comment_boundary(before, writer)?;
                before
                    .gap()
                    .write(writer, FlatSeparator::Space, Writer::space);
                writer.text("=");
                let (head, content) = writer.record_with_value(|writer| {
                    if matches!(
                        self.comment_boundary_binding(after, node.node.span(self.ast)),
                        CommentBinding::Unbound
                    ) {
                        return self.format_prepared_type_content_head(
                            compound::PreparedTypeContent::Compound(single),
                            &mut PendingTypeHead::new(
                                after.comments(),
                                node.node.span(self.ast).start,
                                None,
                            ),
                            TypeContext::Root,
                            writer,
                        );
                    }
                    self.format_comment_boundary_with_layout(
                        after,
                        CommentRunLayout::Leading(single.start()),
                        writer,
                    )?;
                    self.format_single_compound_head(single, TypeContext::Root, writer)
                })?;
                return self.format_type_alias_head(head, content, writer);
            }
            if matches!(node.data, TsTypeData::TsUnionType(_)) {
                self.format_comment_boundary(before, writer)?;
                before
                    .gap()
                    .write(writer, FlatSeparator::Space, Writer::space);
                writer.text("=");
                return writer.group(false, |writer| {
                    writer.indent(|writer| {
                        self.format_comment_boundary(after, writer)?;
                        after
                            .gap()
                            .write(writer, FlatSeparator::Space, Writer::line);
                        self.format_prepared_type_content(
                            compound::PreparedTypeContent::Compound(single),
                            TypeContext::Root,
                            TypeLayout::AliasRhs,
                            writer,
                        )
                    })
                });
            }
            self.format_type_boundary(boundary, writer)?;
            return self.format_prepared_type_content(
                compound::PreparedTypeContent::Compound(single),
                TypeContext::Root,
                TypeLayout::Root,
                writer,
            );
        }
        let leading_head = if matches!(
            node.data,
            TsTypeData::TsArrayType(_)
                | TsTypeData::TsIndexedAccessType(_)
                | TsTypeData::TsConditionalType(_)
        ) {
            self.type_movable_head(node, TypeContext::Root)
        } else {
            None
        };
        if node.source_owner.is_none()
            && matches!(
                node.data,
                TsTypeData::TsArrayType(_)
                    | TsTypeData::TsIndexedAccessType(_)
                    | TsTypeData::TsConditionalType(_)
            )
            && (!after.is_empty() || leading_head.is_some())
            && matches!(
                self.comment_boundary_binding(after, node.node.span(self.ast)),
                CommentBinding::Unbound
            )
        {
            self.format_comment_boundary(before, writer)?;
            before
                .gap()
                .write(writer, FlatSeparator::Space, Writer::space);
            writer.text("=");
            let (head, content) = writer.record_with_value(|writer| {
                self.format_movable_type_head(
                    node,
                    &mut PendingTypeHead::new(
                        after.comments(),
                        node.node.span(self.ast).start,
                        leading_head,
                    ),
                    TypeContext::Root,
                    writer,
                )
            })?;
            return self.format_type_alias_head(head, content, writer);
        }
        let mut union = match node.data {
            TsTypeData::TsUnionType(union) => {
                Some(self.prepare_union(node, union.types(self.ast))?)
            }
            _ => None,
        };
        let intersection = match node.data {
            TsTypeData::TsIntersectionType(intersection) => {
                Some(self.prepare_intersection(node, intersection.types(self.ast))?)
            }
            _ => None,
        };
        let head_start = intersection
            .as_ref()
            .and_then(compound::PreparedIntersection::movable_head_start);
        let intersection_head = intersection
            .as_ref()
            .and_then(|intersection| self.intersection_movable_head(intersection));
        if node.source_owner.is_none()
            && (!after.is_empty() || intersection_head.is_some())
            && matches!(
                self.comment_boundary_binding(after, node.node.span(self.ast)),
                super::comments::CommentBinding::Unbound
            )
        {
            self.format_comment_boundary(before, writer)?;
            before
                .gap()
                .write(writer, FlatSeparator::Space, Writer::space);
            writer.text("=");
            if self.union_head_is_inline(after.comments(), node.node.span(self.ast).start)
                && let Some(mut prepared) = union.take()
            {
                if prepared.accepts_inline_head() {
                    let packet = self.prepare_comment_boundary(
                        after.comments(),
                        after.end(),
                        node.node.span(self.ast).start,
                    );
                    prepared.set_inline_head(packet, node.node.span(self.ast).start);
                    return writer.group(false, |writer| {
                        writer.indent(|writer| {
                            writer.line();
                            self.format_prepared_union(
                                prepared,
                                TypeContext::Root,
                                TypeLayout::AliasRhs,
                                None,
                                writer,
                            )
                        })
                    });
                }
                union = Some(prepared);
            }
            let (head, content) = writer.record_with_value(|writer| {
                self.format_comment_boundary_with_layout(
                    after,
                    CommentRunLayout::GroupedLeading(
                        head_start.unwrap_or_else(|| node.node.span(self.ast).start),
                    ),
                    writer,
                )?;
                let content = if let Some(intersection) = intersection {
                    compound::PreparedTypeContent::Intersection(self.format_intersection_head(
                        intersection,
                        intersection_head,
                        writer,
                    )?)
                } else if let Some(union) = union {
                    compound::PreparedTypeContent::Union(union)
                } else {
                    compound::PreparedTypeContent::Type(node)
                };
                Ok::<_, FormatError>(content)
            })?;
            return self.format_type_alias_head(head, content, writer);
        }
        if let Some(intersection) = intersection {
            if before.is_empty()
                && after.is_empty()
                && !before.gap().requires_line()
                && !after.gap().requires_line()
            {
                self.format_formatted_type_boundary_operator(boundary.role, before, writer)?;
                return writer.group(false, |writer| {
                    rhs_layout.write(
                        writer,
                        |writer| match rhs_layout {
                            RhsLayout::NeverBreakAfterOperator => {
                                after
                                    .gap()
                                    .write(writer, FlatSeparator::Space, Writer::space)
                            }
                            RhsLayout::Fluid | RhsLayout::BreakAfterOperator => {
                                after
                                    .gap()
                                    .write(writer, FlatSeparator::Space, Writer::line)
                            }
                        },
                        |writer| {
                            self.format_prepared_intersection_with_head(
                                intersection,
                                intersection_head,
                                TypeContext::Root,
                                TypeLayout::Root,
                                writer,
                            )
                        },
                    )
                });
            }
            self.format_type_boundary(boundary, writer)?;
            return self.format_prepared_intersection_with_head(
                intersection,
                intersection_head,
                TypeContext::Root,
                TypeLayout::Root,
                writer,
            );
        }
        let Some(union) = union else {
            if before.is_empty()
                && after.is_empty()
                && !before.gap().requires_line()
                && !after.gap().requires_line()
            {
                let rhs_layout = match node.data {
                    TsTypeData::TsConditionalType(conditional) => self
                        .conditional_type_rhs_layout(conditional)
                        .unwrap_or(rhs_layout),
                    _ => rhs_layout,
                };
                self.format_formatted_type_boundary_operator(boundary.role, before, writer)?;
                return writer.group(false, |writer| {
                    rhs_layout.write(
                        writer,
                        |writer| match rhs_layout {
                            RhsLayout::NeverBreakAfterOperator => {
                                after
                                    .gap()
                                    .write(writer, FlatSeparator::Space, Writer::space)
                            }
                            RhsLayout::Fluid | RhsLayout::BreakAfterOperator => {
                                after
                                    .gap()
                                    .write(writer, FlatSeparator::Space, Writer::line)
                            }
                        },
                        |writer| self.format_prepared_type(node, TypeContext::Root, writer),
                    )
                });
            }
            self.format_type_boundary(boundary, writer)?;
            return self.format_prepared_type(node, TypeContext::Root, writer);
        };
        if union.alias_layout() == TypeLayout::Root {
            self.format_type_boundary(boundary, writer)?;
            return self.format_prepared_union(
                union,
                TypeContext::Root,
                TypeLayout::Root,
                None,
                writer,
            );
        }
        self.format_comment_boundary(before, writer)?;
        before
            .gap()
            .write(writer, FlatSeparator::Space, Writer::space);
        writer.text("=");
        writer.group(false, |writer| {
            writer.indent(|writer| {
                self.format_comment_boundary(after, writer)?;
                after
                    .gap()
                    .write(writer, FlatSeparator::Space, Writer::line);
                self.format_prepared_union(
                    union,
                    TypeContext::Root,
                    TypeLayout::AliasRhs,
                    None,
                    writer,
                )
            })
        })
    }

    fn format_type_alias_head(
        &mut self,
        head: Fragment<'ast>,
        content: compound::PreparedTypeContent<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let layout = content.alias_layout();
        let expand = head.preferred_breaks() || !content.is_object_type() && head.can_break();
        if expand || layout == TypeLayout::AliasRhs {
            writer.group(false, |writer| {
                writer.indent(|writer| {
                    writer.line();
                    writer.write_fragment(head);
                    self.format_prepared_type_content(content, TypeContext::Root, layout, writer)
                })
            })
        } else {
            writer.space();
            writer.group(false, |writer| writer.write_fragment(head));
            self.format_prepared_type_content(content, TypeContext::Root, layout, writer)
        }
    }

    fn prepare_type_boundary(
        &self,
        gap: Span,
        mut target: TypeBoundaryTarget<'_>,
    ) -> Result<TypeBoundary<'ast>, FormatError> {
        let (token, mut role) = match &target {
            TypeBoundaryTarget::Alias(_) | TypeBoundaryTarget::ModuleReference => {
                ("=", TypeBoundaryRole::Alias)
            }
            TypeBoundaryTarget::Qualified => (".", TypeBoundaryRole::Qualified),
            TypeBoundaryTarget::Constraint { name, .. } => {
                // A modifier-token name before `extends` needs a line terminator
                // to remain the name. Read the real identifier, including escapes.
                let leading = if matches!(
                    self.ast.get_utf8(name.name(self.ast)),
                    "const" | "in" | "out"
                ) {
                    LineConstraint::NewLine(1)
                } else {
                    LineConstraint::Unconstrained
                };
                ("extends", TypeBoundaryRole::Constraint(leading))
            }
            TypeBoundaryTarget::InferConstraint { .. } => (
                "extends",
                TypeBoundaryRole::Constraint(LineConstraint::Unconstrained),
            ),
            TypeBoundaryTarget::Default { .. } => ("=", TypeBoundaryRole::Default),
        };
        let comments = self.comments.in_range(gap);
        let Some(start) = self.find_token(gap, comments, token) else {
            return Err(FormatError::unsupported(
                "type punctuation boundary",
                None,
                gap,
            ));
        };
        let operator = Span::new(start, start + token.len() as u32);
        let split = comments.partition_point(|comment| comment.span.end <= operator.start);
        let (mut before, mut after) = comments.split_at(split);
        if let TypeBoundaryTarget::Alias(node) = &target
            && matches!(node.data, TsTypeData::TsUnionType(union) if union.types(self.ast).len() > 1)
            && before
                .first()
                .is_some_and(|comment| self.source_line_starts_line(comment.span.start))
            && matches!(
                self.comment_binding(comments, gap, Some(node.node.span(self.ast))),
                CommentBinding::Unbound
            )
        {
            before = &comments[..0];
            after = comments;
        }
        let mut left_mode = ContentMode::Formatted;
        let mut right_mode = ContentMode::Formatted;
        let layout = 'layout: {
            if !self.has_suppression || self.content_mode(comments) != ContentMode::Verbatim {
                if matches!(
                    role,
                    TypeBoundaryRole::Constraint(LineConstraint::NewLine(1))
                ) && before
                    .last()
                    .is_some_and(|comment| self.line_breaks(gap.start, comment.span.end) != 0)
                {
                    // SourceAfter prints this preceding run's existing line once.
                    role = TypeBoundaryRole::Constraint(LineConstraint::Unconstrained);
                }
                break 'layout TypeOperatorLayout::Formatted {
                    before: self.prepare_comment_boundary(before, gap.start, operator.start),
                    after: self.prepare_comment_boundary(after, operator.end, gap.end),
                };
            }
            let packet_owner = match &target {
                TypeBoundaryTarget::Constraint { name, .. }
                | TypeBoundaryTarget::InferConstraint { name, .. }
                    if matches!(
                        role,
                        TypeBoundaryRole::Constraint(LineConstraint::Unconstrained)
                    ) =>
                {
                    Some(name.span(self.ast))
                }
                TypeBoundaryTarget::Default {
                    name: Some(name), ..
                } => Some(name.span(self.ast)),
                _ => None,
            };
            if before.is_empty()
                && let Some(owner) = packet_owner
                && let TypeBoundaryTarget::Constraint { node, .. }
                | TypeBoundaryTarget::InferConstraint { node, .. }
                | TypeBoundaryTarget::Default { node, .. } = &target
                && self.can_format_preceding_line_type_boundary(after, owner, node)
            {
                left_mode = ContentMode::Verbatim;
                break 'layout TypeOperatorLayout::Formatted {
                    before: self.prepare_comment_boundary(before, gap.start, operator.start),
                    after: self.prepare_comment_boundary(after, operator.end, gap.end),
                };
            }
            let attachment = match &target {
                TypeBoundaryTarget::Alias(node)
                | TypeBoundaryTarget::Constraint { node, .. }
                | TypeBoundaryTarget::InferConstraint { node, .. }
                | TypeBoundaryTarget::Default { node, .. } => {
                    Some(self.type_attachment_head(node.node))
                }
                _ => None,
            };
            let before_head = attachment
                .as_ref()
                .map_or(TypeHeadRetention::Unchanged, |head| {
                    self.prepare_type_suppression_head(head, before)
                });
            for (run, position) in self.suppressed_comment_runs(before, operator.start) {
                let selection = attachment.as_ref().map(|attachment| {
                    self.prepare_type_suppression_run(
                        attachment,
                        before_head,
                        run,
                        position,
                        TypeSuppressionContext::TypeBoundary,
                    )
                });
                if let Some(selection) = &selection
                    && let TypeBoundaryTarget::Constraint { node, .. }
                    | TypeBoundaryTarget::InferConstraint { node, .. }
                    | TypeBoundaryTarget::Default { node, .. } = &mut target
                {
                    selection.retain_source_head(node, self.ast);
                }
                if !matches!(position, SuppressionPosition::OwnLine)
                    && let Some(owner) = packet_owner
                    && let Some(selection) = &selection
                    && let TypeBoundaryTarget::Constraint { node, .. }
                    | TypeBoundaryTarget::InferConstraint { node, .. }
                    | TypeBoundaryTarget::Default { node, .. } = &mut target
                    && let Some((packet, after)) = self.prepare_type_operator_boundary(
                        comments,
                        run,
                        owner,
                        node,
                        selection,
                        TypeOperatorSlot::new(operator, LineConstraint::Unconstrained),
                    )
                {
                    left_mode = ContentMode::Verbatim;
                    break 'layout TypeOperatorLayout::Preceding { packet, after };
                }
                for &comment in run {
                    if !is_suppression_comment(self.ast, comment) {
                        continue;
                    }
                    let preceding = match role {
                        TypeBoundaryRole::Qualified => {
                            !matches!(position, SuppressionPosition::OwnLine)
                        }
                        TypeBoundaryRole::Constraint(_) | TypeBoundaryRole::Default => {
                            !matches!(position, SuppressionPosition::OwnLine)
                        }
                        TypeBoundaryRole::Alias => {
                            matches!(position, SuppressionPosition::Inline)
                                || matches!(position, SuppressionPosition::EndOfLine)
                                    && comment.kind == CommentKind::Line
                        }
                    };
                    if preceding {
                        left_mode = ContentMode::Verbatim;
                    } else {
                        right_mode = ContentMode::Verbatim;
                        if let Some(attachment) = &attachment
                            && let TypeBoundaryTarget::Alias(node)
                            | TypeBoundaryTarget::Constraint { node, .. }
                            | TypeBoundaryTarget::InferConstraint { node, .. }
                            | TypeBoundaryTarget::Default { node, .. } = &mut target
                        {
                            self.select_type_head_with_retention(
                                node,
                                attachment,
                                before_head,
                                position,
                            );
                        }
                    }
                }
            }
            let after_head = attachment
                .as_ref()
                .map_or(TypeHeadRetention::Unchanged, |head| {
                    self.prepare_type_suppression_head(head, after)
                });
            for (run, position) in self.suppressed_comment_runs(after, gap.end) {
                let selected = match (&attachment, &mut target) {
                    (Some(_), TypeBoundaryTarget::Alias(node)) => {
                        after_head.apply(node, self.ast);
                        TypeSuppressionTarget::Following
                    }
                    (
                        Some(attachment),
                        TypeBoundaryTarget::Constraint { node, .. }
                        | TypeBoundaryTarget::InferConstraint { node, .. }
                        | TypeBoundaryTarget::Default { node, .. },
                    ) => {
                        let selection = self.prepare_type_suppression_run(
                            attachment,
                            after_head,
                            run,
                            position,
                            TypeSuppressionContext::TypeBoundary,
                        );
                        let selected = selection.target();
                        selection.retain_source_head(node, self.ast);
                        if let Some(owner) = packet_owner
                            && let Some((packet, after)) = self.prepare_type_operator_boundary(
                                comments,
                                run,
                                owner,
                                node,
                                &selection,
                                TypeOperatorSlot::new(operator, LineConstraint::Unconstrained),
                            )
                        {
                            left_mode = ContentMode::Verbatim;
                            break 'layout TypeOperatorLayout::Preceding { packet, after };
                        }
                        selected
                    }
                    (_, TypeBoundaryTarget::Qualified)
                        if matches!(position, SuppressionPosition::EndOfLine) =>
                    {
                        TypeSuppressionTarget::Preceding
                    }
                    _ => TypeSuppressionTarget::Following,
                };
                if selected.preceding() {
                    left_mode = ContentMode::Verbatim;
                }
                if !selected.following() {
                    continue;
                }
                right_mode = ContentMode::Verbatim;
                if let Some(attachment) = &attachment
                    && let TypeBoundaryTarget::Alias(node)
                    | TypeBoundaryTarget::Constraint { node, .. }
                    | TypeBoundaryTarget::InferConstraint { node, .. }
                    | TypeBoundaryTarget::Default { node, .. } = &mut target
                {
                    self.select_type_head_with_attachment(node, attachment, position);
                }
            }
            if let Some(attachment) = &attachment
                && let TypeBoundaryTarget::Alias(node)
                | TypeBoundaryTarget::Constraint { node, .. }
                | TypeBoundaryTarget::InferConstraint { node, .. }
                | TypeBoundaryTarget::Default { node, .. } = &mut target
            {
                self.finalize_type_head(node, attachment, comments);
            }
            if left_mode == ContentMode::Formatted
                && right_mode == ContentMode::Verbatim
                && let Some(attachment) = &attachment
                && let TypeBoundaryTarget::Alias(node)
                | TypeBoundaryTarget::Constraint { node, .. }
                | TypeBoundaryTarget::InferConstraint { node, .. }
                | TypeBoundaryTarget::Default { node, .. } = &target
                && let Some(head) = self.prepare_suppressed_type_head(
                    **node,
                    attachment,
                    gap,
                    before,
                    after,
                    SuppressedTypeHeadRole::TypeBoundary,
                )
            {
                break 'layout TypeOperatorLayout::Suppressed {
                    before: self.prepare_comment_boundary(before, gap.start, operator.start),
                    head,
                };
            }
            // A suppressed packet without a proven canonical placement retains
            // its boundary independently of which operand is copied verbatim.
            TypeOperatorLayout::Source(SourceTypeOperator::new(gap, operator))
        };
        Ok(TypeBoundary {
            role,
            left_mode,
            right_mode,
            layout,
        })
    }

    pub(super) fn format_type_boundary(
        &mut self,
        boundary: &TypeBoundary<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let Some((after, space)) = self.format_type_boundary_operator(boundary, writer)? else {
            return Ok(());
        };
        self.format_comment_boundary(after, writer)?;
        after.gap().write(writer, space, |writer| {
            if matches!(
                boundary.role,
                TypeBoundaryRole::Constraint(_) | TypeBoundaryRole::Default
            ) {
                writer.line();
            } else if matches!(space, FlatSeparator::Space) {
                writer.space();
            }
        });
        Ok(())
    }

    fn format_formatted_type_boundary_operator(
        &mut self,
        role: TypeBoundaryRole,
        before: &CommentBoundary<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let (token, space) = match role {
            TypeBoundaryRole::Alias | TypeBoundaryRole::Default => ("=", FlatSeparator::Space),
            TypeBoundaryRole::Qualified => (".", FlatSeparator::Empty),
            TypeBoundaryRole::Constraint(_) => ("extends", FlatSeparator::Space),
        };
        self.format_comment_boundary(before, writer)?;
        if matches!(
            role,
            TypeBoundaryRole::Constraint(LineConstraint::NewLine(1))
        ) && !before.gap().requires_line()
        {
            writer.hard_line();
        } else {
            before.gap().write(writer, space, |writer| {
                if matches!(space, FlatSeparator::Space) {
                    writer.space();
                }
            });
        }
        writer.text(token);
        Ok(())
    }

    pub(super) fn prepare_type(&self, node: TsType, mode: ContentMode) -> PreparedType {
        PreparedType {
            node,
            data: self.ast.ts_type_data(node),
            source_owner: (mode == ContentMode::Verbatim).then_some(node),
            source_head: None,
            suppressed_projection: None,
        }
    }

    fn select_delimited_type_head(
        &self,
        node: &mut PreparedType,
        comments: &[Comment],
        end: u32,
        first: bool,
    ) {
        let mut attachment = None;
        for (run, position) in self.suppressed_comment_runs(comments, end) {
            let preceding = !first
                && !matches!(position, SuppressionPosition::OwnLine)
                && (matches!(position, SuppressionPosition::EndOfLine)
                    || run.last().is_some_and(|last| {
                        self.find_comma(Span::new(last.span.end, end), comments)
                            .is_some()
                    }));
            if !preceding {
                let (attachment, head) = attachment.get_or_insert_with(|| {
                    let attachment = self.type_attachment_head(node.node);
                    let head = self.prepare_type_suppression_head(&attachment, comments);
                    (attachment, head)
                });
                self.select_type_head_with_retention(node, attachment, *head, position);
            }
        }
        if let Some((attachment, _)) = attachment {
            self.finalize_type_head(node, &attachment, comments);
        }
    }

    fn format_type_with_context(
        &mut self,
        annotation: TsType,
        mode: ContentMode,
        context: TypeContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let node = self.prepare_type(annotation, mode);
        self.format_prepared_type(node, context, writer)
    }

    /// Follow the same removable frames used by the type's comment attachment view.
    pub(super) fn type_attachment_head(&self, mut node: TsType) -> TypeAttachmentHead {
        let source = node;
        let mut source_barrier = None;
        let mut interior_frame = None;
        let mut prefix_carrier = None;
        let mut parentheses_only = true;
        let data = loop {
            let data = self.ast.ts_type_data(node);
            let (types, token) = match data {
                TsTypeData::TsParenthesizedType(parentheses) => {
                    let child = parentheses.type_annotation(self.ast);
                    if !self
                        .comments
                        .in_range(Span::new(
                            parentheses.span(self.ast).start + 1,
                            child.span(self.ast).start,
                        ))
                        .is_empty()
                    {
                        prefix_carrier = Some(node);
                    }
                    if matches!(
                        self.ast.ts_type_data(child),
                        TsTypeData::TsParenthesizedType(_)
                    ) {
                        let comments = self.comments.in_range(Span::new(
                            parentheses.span(self.ast).start + 1,
                            child.span(self.ast).start,
                        ));
                        if self.type_union_frame_has_comments(child, comments) {
                            interior_frame = Some(child);
                        }
                    }
                    node = child;
                    continue;
                }
                TsTypeData::TsUnionType(union) => {
                    parentheses_only = false;
                    (union.types(self.ast), "|")
                }
                TsTypeData::TsIntersectionType(intersection) => {
                    parentheses_only = false;
                    (intersection.types(self.ast), "&")
                }
                _ => break data,
            };
            let Some(first) = types.first() else {
                break data;
            };
            let first: TsType = self.ast.get_node_in_sub_range(first);
            let prefix = Span::new(node.span(self.ast).start, first.span(self.ast).start);
            if self
                .find_token(prefix, self.comments.in_range(prefix), token)
                .is_some()
            {
                source_barrier = Some(node);
            }
            if types.len() != 1 {
                break data;
            }
            node = first;
        };
        let union = matches!(data, TsTypeData::TsUnionType(_));
        TypeAttachmentHead {
            node,
            union,
            source_barrier,
            exterior_union_frame: (source != node && union).then_some(source),
            interior_union_frame: interior_frame.filter(|_| union),
            first_owner: std::cell::Cell::new(None),
            prefix_carrier,
            parentheses_only: parentheses_only && source != node,
            prefix: self.prepare_type_comment_tie(
                node,
                union,
                self.comments.in_range(Span::new(
                    source.span(self.ast).start,
                    node.span(self.ast).start,
                )),
            ),
        }
    }

    /// A trailing line marker ignores the name, not its bound's head or layout.
    /// Keep the real after-operator run so its mandatory line remains in the
    /// normal bound separator group.
    fn can_format_preceding_line_type_boundary(
        &self,
        comments: &[Comment],
        owner: Span,
        right: &PreparedType,
    ) -> bool {
        let [comment] = comments else {
            return false;
        };
        let right_span = right.node.span(self.ast);
        comment.kind == CommentKind::Line
            && is_suppression_comment(self.ast, *comment)
            && right.source_owner.is_none()
            && right.source_head.is_none()
            && self.line_breaks(owner.start, comment.span.end) == 0
            && self.line_breaks(comment.span.end, right_span.start) != 0
            && self.comments.in_range(right_span).is_empty()
            && !self.diagnostic_range_is_sensitive(Span::new(owner.start, right_span.end))
    }

    /// Root expression operators try the complete packet before their prefix constructor.
    pub(super) fn prepare_preceding_type_operator(
        &self,
        comments: &'ast [Comment],
        run: &[Comment],
        owner: Span,
        right: &mut PreparedType,
        selection: &TypeSuppressionRun,
        operator: TypeOperatorSlot,
    ) -> Option<(PreparedPrecedingTypePacket<'ast>, CommentBoundary<'ast>)> {
        if selection.preceding_count != Some(run.len()) {
            return None;
        }
        self.prepare_type_operator_boundary(comments, run, owner, right, selection, operator)
    }

    /// Prepare a type operator with independent preceding and RHS source slots.
    pub(super) fn prepare_type_operator_boundary(
        &self,
        comments: &'ast [Comment],
        run: &[Comment],
        owner: Span,
        right: &mut PreparedType,
        selection: &TypeSuppressionRun,
        operator: TypeOperatorSlot,
    ) -> Option<(PreparedPrecedingTypePacket<'ast>, CommentBoundary<'ast>)> {
        let mut projected = *right;
        selection.head.apply(&mut projected, self.ast);
        // A complete preceding packet can release its lexical head through
        // either inline attachment or its physical end-of-line owner proof.
        // Independent RHS comments still retain their own source head.
        if let TypeHeadRetention::Source(head) = selection.head
            && projected.source_head == Some(head)
            && selection.preceding_count == Some(run.len())
            && run.len() == comments.len()
            && (selection.head_projection.is_some()
                || matches!(
                    selection.packet_projection,
                    Some(PrecedingTypePacketProjection::EndOfLine(_))
                ))
            && self
                .comments
                .in_range(projected.node.span(self.ast))
                .is_empty()
        {
            projected.source_head = None;
        }
        let packet = self.prepare_preceding_type_operator_packet(
            comments, run, owner, &projected, selection, operator,
        )?;
        *right = projected;
        Some(packet)
    }

    fn prepare_preceding_type_operator_packet(
        &self,
        comments: &'ast [Comment],
        run: &[Comment],
        owner: Span,
        right: &PreparedType,
        selection: &TypeSuppressionRun,
        operator: TypeOperatorSlot,
    ) -> Option<(PreparedPrecedingTypePacket<'ast>, CommentBoundary<'ast>)> {
        let TypeOperatorSlot {
            span: operator,
            preceding,
        } = operator;
        if !matches!(selection.target, TypeSuppressionTarget::Preceding) || comments.is_empty() {
            return None;
        }
        let right_span = right.node.span(self.ast);
        // An operator envelope carries the RHS opening independently of this
        // preceding packet.
        if right.source_owner.is_some() {
            return None;
        }
        let inner = self.comments.in_range(right_span);
        if !inner.is_empty() && selection.packet_projection.is_none()
            || self.diagnostic_range_is_sensitive(Span::new(owner.start, right_span.end))
        {
            return None;
        }
        // A partial packet can keep the RHS's actual compound head. Its two
        // owners are certified below; the head itself is neither moved nor freed.
        if let Some(head) = right.source_head
            && (!matches!(selection.head, TypeHeadRetention::Source(selected) if selected == head)
                || !matches!(
                    self.ast.ts_type_data(head),
                    TsTypeData::TsUnionType(_) | TsTypeData::TsIntersectionType(_)
                )
                || !inner.is_empty())
        {
            return None;
        }
        let preceding_count = selection.preceding_count?;
        let preceding_run = run.get(..preceding_count)?;
        if preceding == LineConstraint::SameLine
            && preceding_run.first().is_some_and(|first| {
                preceding_run
                    .last()
                    .is_some_and(|last| self.line_breaks(first.span.start, last.span.end) != 0)
                    || preceding_run
                        .iter()
                        .any(|comment| comment.kind == CommentKind::Line)
            })
        {
            return None;
        }
        if preceding_count < run.len() {
            // A source packet has one preceding prefix and one independent RHS
            // suffix. Moving the whole run would change an ordinary head's owner.
            if run.len() != comments.len() || !comments.starts_with(run) {
                return None;
            }
            let (prefix, following) = comments.split_at(preceding_count);
            let first = prefix.first()?;
            let last = prefix.last()?;
            let first_following = following.first()?;
            if self.line_breaks(owner.start, first.span.start) != 0
                || prefix
                    .iter()
                    .any(|comment| comment.kind != CommentKind::Block)
                || first_following.span.start < operator.end
                || last.span.end > first_following.span.start
                || !self
                    .comment_binding(prefix, Span::new(owner.start, right_span.end), Some(owner))
                    .allows_relocation(Some(owner))
                || !matches!(
                    self.comment_binding(
                        following,
                        Span::new(operator.end, right_span.start),
                        Some(right_span),
                    ),
                    CommentBinding::Unbound
                )
            {
                return None;
            }
            return Some((
                PreparedPrecedingTypePacket { comments: prefix },
                self.prepare_comment_boundary(following, operator.end, right_span.start),
            ));
        }
        if run.len() != comments.len() {
            if !matches!(
                selection.packet_projection,
                Some(PrecedingTypePacketProjection::Inline)
            ) || run.is_empty()
                || !comments.starts_with(run)
            {
                return None;
            }
            let (preceding, following) = comments.split_at(run.len());
            let first_following = following.first()?;
            let last_preceding = preceding.last()?;
            // The complete source packet has two independent owners. Preserve
            // its prefix on the preceding owner and pass the RHS head through
            // the same operator boundary on both original and normalized input.
            if last_preceding.span.end > operator.start
                || first_following.span.start < operator.end
                || self.line_breaks(owner.start, last_preceding.span.end) != 0
                || preceding
                    .iter()
                    .any(|comment| comment.kind != CommentKind::Block)
                || !self
                    .comment_binding(
                        preceding,
                        Span::new(owner.start, right_span.end),
                        Some(owner),
                    )
                    .allows_relocation(Some(owner))
                || !matches!(
                    self.comment_binding(
                        following,
                        Span::new(operator.end, right_span.start),
                        Some(right_span),
                    ),
                    CommentBinding::Unbound
                )
            {
                return None;
            }
            return Some((
                PreparedPrecedingTypePacket {
                    comments: preceding,
                },
                self.prepare_comment_boundary(following, operator.end, right_span.start),
            ));
        }
        // Only the two-owner paths above can carry a retained RHS head.
        if right.source_head.is_some() {
            return None;
        }
        let empty_after = || self.prepare_comment_boundary(&[], operator.end, right_span.start);
        if let [comment] = comments
            && comment.kind == CommentKind::Block
            && is_suppression_comment(self.ast, *comment)
            && self.line_breaks(owner.start, comment.span.end) == 0
        {
            // Preserve the old single-marker EOL gap as well as its raw owner.
            return Some((PreparedPrecedingTypePacket { comments }, empty_after()));
        }
        let first = comments[0];
        if self.line_breaks(owner.start, first.span.start) != 0 {
            return None;
        }
        if matches!(
            selection.packet_projection,
            Some(PrecedingTypePacketProjection::Inline)
        ) {
            let last = comments[comments.len() - 1];
            if comments
                .iter()
                .any(|comment| comment.kind != CommentKind::Block)
                || self.line_breaks(last.span.end, right_span.start) != 0
            {
                return None;
            }
            if self.line_breaks(owner.start, right_span.start) == 0 {
                return Some((PreparedPrecedingTypePacket { comments }, empty_after()));
            }
        } else {
            let Some(PrecedingTypePacketProjection::EndOfLine(context)) =
                selection.packet_projection
            else {
                return None;
            };
            let (last, prefix) = comments.split_last()?;
            // This envelope owns the complete physical slot after its operator.
            // A preceding EOL run keeps its source boundary instead of crossing it.
            if first.span.start < operator.end || last.span.end > right_span.start {
                return None;
            }
            if last.kind == CommentKind::Line {
                // A cast Line can attach to the enclosing declaration instead.
                // Type-boundary callers must also consume the mandatory after gap.
                if prefix.is_empty()
                    || !matches!(context, TypeSuppressionContext::TypeBoundary)
                    || prefix.iter().any(|comment| {
                        comment.kind != CommentKind::Block
                            || self.line_breaks(comment.span.start, comment.span.end) != 0
                    })
                    || !self
                        .comment_binding(
                            comments,
                            Span::new(owner.start, right_span.end),
                            Some(owner),
                        )
                        .allows_relocation(Some(owner))
                {
                    return None;
                }
                let after = self.prepare_comment_boundary(
                    &comments[prefix.len()..],
                    operator.end,
                    right_span.start,
                );
                return Some((PreparedPrecedingTypePacket { comments: prefix }, after));
            }
            if comments.iter().any(|comment| {
                comment.kind != CommentKind::Block
                    || self.line_breaks(comment.span.start, comment.span.end) != 0
            }) {
                return None;
            }
        }
        if !self
            .comment_binding(
                comments,
                Span::new(owner.start, right_span.end),
                Some(owner),
            )
            .allows_relocation(Some(owner))
        {
            return None;
        }
        // The packet's own lines move with its preceding owner. They are not an
        // additional source gap between the printed operator and formatted RHS.
        Some((PreparedPrecedingTypePacket { comments }, empty_after()))
    }

    /// The original after-operator run can have two independently attached sides.
    /// Keep its raw owner and final RHS source gap distinct from the real operator.
    pub(super) fn prepare_preceding_type_prefix(
        &self,
        comments: &'ast [Comment],
        run: &[Comment],
        owner: Span,
        right: &PreparedType,
        prefix: PrecedingTypePrefix,
        operator: Span,
    ) -> Option<(PreparedPrecedingTypePacket<'ast>, CommentBoundary<'ast>)> {
        let right_span = right.node.span(self.ast);
        if run.len() != comments.len()
            || !comments.starts_with(run)
            || owner.end > operator.start
            || operator.start >= operator.end
            || operator.end > right_span.start
            || comments.first()?.span.start < operator.end
            || comments.last()?.span.end > right_span.start
            || right.source_owner.is_some()
            || right.source_head.is_some()
            || right.suppressed_projection.is_some()
            || !self.comments.in_range(right_span).is_empty()
            || self.diagnostic_range_is_sensitive(Span::new(owner.start, right_span.end))
        {
            return None;
        }
        let (preceding, following) = comments.split_at(prefix.end);
        let last_preceding = preceding.last()?;
        let first_following = following.first()?;
        if self.line_breaks(owner.start, last_preceding.span.end) != 0
            || self.line_breaks(operator.end, first_following.span.start) != 0
            || preceding
                .iter()
                .any(|comment| comment.kind != CommentKind::Block)
            || !self
                .comment_binding(
                    preceding,
                    Span::new(owner.start, right_span.end),
                    Some(owner),
                )
                .allows_relocation(Some(owner))
            || !matches!(
                self.comment_binding(
                    following,
                    Span::new(last_preceding.span.end, right_span.start),
                    Some(right_span),
                ),
                CommentBinding::Unbound
            )
        {
            return None;
        }
        Some((
            PreparedPrecedingTypePacket {
                comments: preceding,
            },
            self.prepare_comment_boundary(following, last_preceding.span.end, right_span.start),
        ))
    }

    pub(super) fn prepare_type_rhs_boundary(
        &self,
        source: SourceTypeOperator,
        node: PreparedType,
    ) -> Option<CommentBoundary<'ast>> {
        if self.diagnostic_range_is_sensitive(Span::new(
            source.prefix.start,
            node.node.span(self.ast).end,
        )) {
            return None;
        }
        let comments = self.comments.in_range(source.after);
        let mut start = source.after.start;
        for comment in comments {
            if comment.span.start < start
                || comment.span.end > source.after.end
                || !self
                    .source(Span::new(start, comment.span.start))
                    .chars()
                    .all(is_source_whitespace)
            {
                return None;
            }
            start = comment.span.end;
        }
        if !self
            .source(Span::new(start, source.after.end))
            .chars()
            .all(is_source_whitespace)
        {
            return None;
        }
        let after = self.prepare_comment_boundary(comments, source.after.start, source.after.end);
        matches!(
            self.comment_boundary_binding(&after, node.node.span(self.ast)),
            CommentBinding::Unbound
        )
        .then_some(after)
    }

    /// A source head has its own entry scope, separate from the opaque RHS body.
    pub(super) fn format_source_type_operator_rhs(
        &mut self,
        source: SourceTypeOperator,
        node: PreparedType,
        context: TypeContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_source_type_operator_rhs_output(source, node, context, writer)
            .map(|output| output.pending)
    }

    pub(super) fn format_source_type_operator_rhs_output(
        &mut self,
        source: SourceTypeOperator,
        node: PreparedType,
        context: TypeContext,
        writer: &mut Writer<'ast>,
    ) -> Result<ExpressionOutput<'ast>, FormatError> {
        self.format_source_gap(source.prefix, writer)?;
        let body = self.prepare_operator_rhs_type(node, context)?;
        let entry = self
            .comments
            .in_range(source.after)
            .first()
            .map_or(source.after.end, |comment| comment.span.start);
        let head_entry = self.line_constraint(source.after.start, entry);
        let (head, ()) =
            writer.record_with_value(|writer| self.format_source_gap(source.after, writer))?;
        if matches!(head_entry, LineConstraint::NewLine(_)) {
            writer.indent(|writer| {
                writer.write_fragment(head);
                self.format_prepared_delimited_type_output(
                    body,
                    context,
                    TypeLayout::CallerIndented,
                    writer,
                )
            })
        } else {
            writer.write_fragment(head);
            writer.indent(|writer| {
                self.format_prepared_delimited_type_output(
                    body,
                    context,
                    TypeLayout::CallerIndented,
                    writer,
                )
            })
        }
    }

    pub(super) fn format_preceding_type_packet(
        &mut self,
        packet: &PreparedPrecedingTypePacket<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        self.comments.claim(packet.comments)?;
        self.format_inline_trailing_comments(packet.comments, writer)
    }

    /// Keep the formatted head separate from its already selected raw owner.
    fn prepare_suppressed_type_projection(
        &self,
        node: &PreparedType,
        attachment: &TypeAttachmentHead,
        first_owner: Option<TsType>,
        packet: &[Comment],
    ) -> Option<SuppressedTypeProjection> {
        let owner = node.source_owner?;
        let span = node.node.span(self.ast);
        if node.source_head != attachment.source_barrier || self.diagnostic_range_is_sensitive(span)
        {
            return None;
        }
        let comments = self.comments.in_range(span);
        let packet_start = if attachment.union {
            if first_owner != Some(owner) {
                return None;
            }
            let first = packet.first()?;
            let prefix_span = Span::new(span.start, attachment.node.span(self.ast).start);
            let prefix = self.comments.in_range(prefix_span);
            if !self.source_position_starts_line(first.span.start)
                || comments.len() != prefix.len()
                || !matches!(
                    self.comment_binding(prefix, prefix_span, None),
                    CommentBinding::Unbound
                )
                || !self
                    .comment_binding(
                        packet,
                        Span::new(first.span.start, span.end),
                        Some(owner.span(self.ast)),
                    )
                    .allows_relocation(Some(owner.span(self.ast)))
            {
                return None;
            }
            let mut prefix = attachment.prefix;
            let mut next = span.start;
            for &comment in packet.iter().rev() {
                // The whole caller packet must be the actual type frontier,
                // with no intervening operator or earlier syntax owner.
                if comment.span.end > next
                    || !self
                        .source(Span::new(comment.span.end, next))
                        .chars()
                        .all(is_source_whitespace)
                {
                    return None;
                }
                self.advance_type_comment_prefix(&mut prefix, comment, true);
                next = comment.span.start;
            }
            if !prefix.preserves_union_owner {
                return None;
            }
            // A nonempty comment starts below u32::MAX; preserve offset zero.
            Some(NonZeroU32::new(first.span.start + 1).expect("comment start has a successor"))
        } else {
            if owner != attachment.node
                || node.node == attachment.node
                || node.source_head.is_none()
                || !comments.is_empty()
            {
                return None;
            }
            None
        };
        Some(SuppressedTypeProjection {
            source: node.node,
            owner,
            retained_head: node.source_head,
            surviving_head: attachment.node,
            packet_start,
        })
    }

    /// The same source identities and unclaimed packet authorize parent and body.
    fn projected_suppressed_type(
        &self,
        projection: &SuppressedTypeProjection,
    ) -> Option<ProjectedSuppressedType<'ast>> {
        let comments = if let Some(start) = projection.packet_start {
            let start = start.get() - 1;
            let comments = self.comments.in_range(Span::new(
                start,
                projection.surviving_head.span(self.ast).start,
            ));
            if comments
                .first()
                .is_none_or(|comment| comment.span.start != start)
            {
                return None;
            }
            comments
        } else {
            &[]
        };
        let mut head = self.prepare_type(projection.surviving_head, ContentMode::Formatted);
        head.source_owner = Some(projection.owner);
        Some(ProjectedSuppressedType {
            node: head,
            comments,
        })
    }

    /// Admit only a head whose attachment is independent of its printed line.
    /// Every comment must lead the same complete RHS when the packet is inline
    /// or on its own line. The caller has already selected the original raw owner.
    pub(super) fn prepare_suppressed_type_head(
        &self,
        node: PreparedType,
        attachment: &TypeAttachmentHead,
        gap: Span,
        before: &[Comment],
        after: &'ast [Comment],
        role: SuppressedTypeHeadRole,
    ) -> Option<SuppressedTypeHead<'ast>> {
        let first = after.first()?;
        let source_span = node.node.span(self.ast);
        if !before.is_empty()
            || self.diagnostic_range_is_sensitive(Span::new(gap.start, source_span.end))
        {
            return None;
        }
        if attachment.union
            && let Some(projection) = node.suppressed_projection()
            && let Some(head) = self.projected_suppressed_type(projection)
            && head.node.source_owner != Some(head.node.node)
            && self.source_position_starts_line(first.span.start)
            && after
                .iter()
                .any(|&comment| is_suppression_comment(self.ast, comment))
            && matches!(
                self.comment_run_binding(after.iter().copied(), |comment| {
                    let semantics = self.comment_semantics(comment);
                    if semantics.diagnostic || semantics.file_directive {
                        CommentBinding::SourceBound
                    } else {
                        CommentBinding::Unbound
                    }
                }),
                CommentBinding::Unbound
            )
        {
            return Some(SuppressedTypeHead {
                comments: head.comments,
                next_start: head.node.node.span(self.ast).start,
                own_line: true,
            });
        }
        if node.source_owner != Some(attachment.node)
            || self.line_breaks(first.span.start, source_span.start) != 0
        {
            return None;
        }
        let context = match role {
            SuppressedTypeHeadRole::ExpressionBoundary => {
                TypeSuppressionContext::ExpressionBoundary
            }
            _ => TypeSuppressionContext::TypeBoundary,
        };
        let mut tie = attachment.prefix.tie;
        if let Some(projection) = node.suppressed_projection()
            && self.projected_suppressed_type(projection).is_some()
        {
            tie.next = source_span.start;
        }
        let mut markers = 0;
        let mut all_following = true;
        let mut marker_types_following = true;
        for &comment in after.iter().rev() {
            if comment.kind != CommentKind::Block
                || self.line_breaks(comment.span.start, comment.span.end) != 0
            {
                return None;
            }
            all_following &= self.advance_type_comment_tie(&mut tie, comment, true, context);
            if is_suppression_comment(self.ast, comment) {
                markers += 1;
                // Markers do not attach independently. This is the original
                // type-boundary projection, including its use by binary callers.
                marker_types_following &= tie.type_leading;
            }
        }
        if markers == 0 {
            return None;
        }
        // Keep the existing marker-only contract, including independently retained
        // source frames. Broader packets must also account for a removable prefix.
        let marker_only = markers == after.len();
        let existing_marker_head = !attachment.union
            && marker_only
            && marker_types_following
            && matches!(
                role,
                SuppressedTypeHeadRole::TypeBoundary | SuppressedTypeHeadRole::ExpressionBoundary
            );
        if !existing_marker_head {
            let attachment_start = attachment.node.span(self.ast).start;
            let source_breaks = self.line_breaks(first.span.start, attachment_start);
            if !all_following
                || source_breaks != 0
                    && !(matches!(
                        role,
                        SuppressedTypeHeadRole::ConditionalExtends
                            | SuppressedTypeHeadRole::MappedBoundary
                    ) && attachment.parentheses_only
                        && node.source_head.is_none())
                || attachment.union
                    && !(matches!(role, SuppressedTypeHeadRole::ConditionalExtends)
                        && marker_only
                        && markers == 1)
            {
                return None;
            }
            let inner = self.comments.in_range(source_span);
            // The attachment walk identified every frame. Empty parentheses
            // without a retained head disappear before the raw child.
            if source_breaks != 0 && !inner.is_empty() {
                return None;
            }
            // Removing transparent pairs can expose ordinary prefix comments as
            // part of this packet. Admit that same layout on the first pass;
            // the existing body emitter still consumes the internal prefix.
            // For a nonunion, same-line block comments have no independent
            // attachment, so the shared leading chain proves their whole owner.
            if !inner.is_empty()
                && (attachment.union
                    || inner.iter().any(|&comment| {
                        comment.span.end > attachment_start
                            || comment.kind != CommentKind::Block
                            || is_suppression_comment(self.ast, comment)
                    }))
            {
                return None;
            }
        }
        Some(SuppressedTypeHead {
            comments: after,
            next_start: source_span.start,
            own_line: false,
        })
    }

    pub(super) fn format_suppressed_type_head(
        &mut self,
        head: &SuppressedTypeHead<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        self.comments.claim(head.comments)?;
        self.format_comment_run(
            head.comments,
            CommentRunLayout::Leading(head.next_start),
            writer,
        )
    }

    pub(super) fn format_suppressed_type_content(
        &mut self,
        node: PreparedType,
        head: &SuppressedTypeHead<'ast>,
        context: TypeContext,
        layout: TypeLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_suppressed_type_content_output(node, head, context, layout, writer)
            .map(|output| output.pending)
    }

    pub(super) fn format_suppressed_type_content_output(
        &mut self,
        node: PreparedType,
        head: &SuppressedTypeHead<'ast>,
        context: TypeContext,
        layout: TypeLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<ExpressionOutput<'ast>, FormatError> {
        if !head.own_line {
            self.format_suppressed_type_head(head, writer)?;
            return self.format_prepared_type_output(node, context, layout, writer);
        }
        let recorded = self.record_type_head(
            PreparedTypeContent::Type(node),
            PendingTypeHead::new(head.comments, head.next_start, None),
            context,
            writer,
        )?;
        let layout = if layout == TypeLayout::OpeningProvided {
            TypeLayout::Root
        } else {
            layout
        };
        self.format_recorded_type_head_output(recorded, context, layout, writer)
    }

    pub(super) fn prepare_type_suppression_head(
        &self,
        attachment: &TypeAttachmentHead,
        comments: &[Comment],
    ) -> TypeHeadRetention {
        let frame = attachment.interior_union_frame.or_else(|| {
            attachment
                .exterior_union_frame
                .filter(|&frame| self.type_union_frame_has_comments(frame, comments))
        });
        let owner = match (frame, attachment.source_barrier) {
            (Some(frame), Some(barrier)) => Some(
                if frame.span(self.ast).start < barrier.span(self.ast).start {
                    frame
                } else {
                    barrier
                },
            ),
            (frame, barrier) => frame.or(barrier),
        };
        owner.map_or(TypeHeadRetention::Unchanged, TypeHeadRetention::Source)
    }

    fn type_union_frame_has_comments(&self, frame: TsType, comments: &[Comment]) -> bool {
        let mut next = frame.span(self.ast).start;
        for &comment in comments.iter().rev() {
            if comment.span.end > next
                || !self
                    .source(Span::new(comment.span.end, next))
                    .chars()
                    .all(|character| {
                        is_source_whitespace(character)
                            && !matches!(character, '\n' | '\r' | '\u{2028}' | '\u{2029}')
                    })
            {
                break;
            }
            if comment.kind == CommentKind::Block
                && !is_suppression_comment(self.ast, comment)
                && self.line_breaks(comment.span.start, comment.span.end) == 0
            {
                return true;
            }
            next = comment.span.start;
        }
        false
    }

    fn prepare_type_comment_tie(
        &self,
        node: TsType,
        union: bool,
        comments: &[Comment],
    ) -> TypeCommentPrefix {
        let mut prefix = TypeCommentPrefix {
            tie: TypeCommentTie {
                next: node.span(self.ast).start,
                type_leading: true,
                expression_leading: true,
                union_adjacency: union,
            },
            source_leads_head: true,
            source_lines_adjacency: union,
            preserves_union_owner: true,
            preserves_horizontal_owner: true,
        };
        // Resolve the shared prefix once, before processing boundary runs.
        // Markers inside these frames still belong to their own suppression runs.
        let mut following_start = prefix.tie.next;
        for run in comments
            .chunk_by(|left, right| {
                self.source(Span::new(left.span.end, right.span.start))
                    .chars()
                    .all(|character| {
                        is_source_whitespace(character)
                            && !matches!(character, '\n' | '\r' | '\u{2028}' | '\u{2029}')
                    })
            })
            .rev()
        {
            let first = run[0];
            let last = run[run.len() - 1];
            let tied = matches!(
                self.source_comment_run_position(first, last, following_start),
                SuppressionPosition::Inline
            );
            for &comment in run.iter().rev() {
                self.advance_type_comment_prefix(&mut prefix, comment, tied);
            }
            following_start = first.span.start;
        }
        prefix
    }

    /// Compare real source owners with the packet's physical SourceLines layout.
    fn advance_type_comment_prefix(
        &self,
        prefix: &mut TypeCommentPrefix,
        comment: Comment,
        tied: bool,
    ) {
        let gap = self.source(Span::new(comment.span.end, prefix.tie.next));
        prefix.source_leads_head &= gap
            .chars()
            .all(|character| is_source_whitespace(character) || character == '(');
        prefix.source_lines_adjacency &= self.line_breaks(comment.span.end, prefix.tie.next) == 0;
        self.advance_type_comment_tie(
            &mut prefix.tie,
            comment,
            tied,
            TypeSuppressionContext::TypeBoundary,
        );
        let single_line = self.line_breaks(comment.span.start, comment.span.end) == 0;
        if self.comment_semantics(comment).is_ordinary() {
            // A removed operator can expose a different owner even when both
            // layouts are nonadjacent. Ordinary comments must lead this head.
            prefix.preserves_union_owner &= prefix.source_leads_head;
            if comment.kind == CommentKind::Block && single_line {
                let same_owner = prefix.tie.union_adjacency == prefix.source_lines_adjacency;
                prefix.preserves_union_owner &= same_owner;
                if !same_owner
                    && gap
                        .chars()
                        .all(super::source::is_source_horizontal_whitespace)
                {
                    prefix.preserves_horizontal_owner = false;
                }
            }
        }
        prefix.source_lines_adjacency &= single_line;
    }

    fn advance_type_comment_tie(
        &self,
        tie: &mut TypeCommentTie,
        comment: Comment,
        tied: bool,
        context: TypeSuppressionContext,
    ) -> bool {
        let gap = self.source(Span::new(comment.span.end, tie.next));
        let leading = tied
            && gap
                .chars()
                .all(|character| is_source_whitespace(character) || character == '(');
        tie.type_leading &= leading;
        tie.expression_leading &= leading;
        tie.union_adjacency &= gap.chars().all(|character| matches!(character, ' ' | '\t'));
        let single_line = self.line_breaks(comment.span.start, comment.span.end) == 0;
        let block = comment.kind == CommentKind::Block;
        let type_attached = block
            && tie.union_adjacency
            && single_line
            && !is_suppression_comment(self.ast, comment);
        let expression_attached = type_attached || block && !single_line;
        let following = match context {
            TypeSuppressionContext::TypeBoundary => type_attached || tie.type_leading,
            TypeSuppressionContext::ExpressionBoundary => {
                expression_attached || tie.expression_leading
            }
        };
        // Independently attached comments cannot form another comment's tie chain.
        tie.type_leading &= !type_attached;
        tie.expression_leading &= !expression_attached;
        tie.union_adjacency &= single_line;
        tie.next = comment.span.start;
        following
    }

    /// Resolve the protected operands separately from their source attachment frame.
    pub(super) fn prepare_type_suppression_run(
        &self,
        attachment: &TypeAttachmentHead,
        head: TypeHeadRetention,
        comments: &[Comment],
        position: SuppressionPosition,
        context: TypeSuppressionContext,
    ) -> TypeSuppressionRun {
        self.resolve_type_suppression_run(attachment, head, comments, position, context)
            .selection
    }

    pub(super) fn resolve_type_suppression_run(
        &self,
        attachment: &TypeAttachmentHead,
        head: TypeHeadRetention,
        comments: &[Comment],
        position: SuppressionPosition,
        context: TypeSuppressionContext,
    ) -> ResolvedTypeSuppressionRun {
        if matches!(position, SuppressionPosition::OwnLine) {
            return ResolvedTypeSuppressionRun {
                selection: TypeSuppressionRun {
                    target: TypeSuppressionTarget::Following,
                    head,
                    packet_projection: None,
                    head_projection: None,
                    preceding_count: None,
                },
                preceding_prefix: None,
            };
        }
        let end_of_line = matches!(position, SuppressionPosition::EndOfLine);
        let mut selection_tie = attachment.prefix.tie;
        let mut source_tie = attachment.prefix.tie;
        let source_tied = matches!(position, SuppressionPosition::Inline);
        let mut preceding = false;
        let mut following = false;
        let mut all_preceding = true;
        let mut prefix_end = comments.len();
        let mut preceding_seen = false;
        let mut contiguous = true;
        for (index, &comment) in comments.iter().enumerate().rev() {
            let selected_following =
                self.advance_type_comment_tie(&mut selection_tie, comment, true, context);
            let source_following =
                self.advance_type_comment_tie(&mut source_tie, comment, source_tied, context);
            let suppression = is_suppression_comment(self.ast, comment);
            // The marker keeps its already selected target. Ordinary attachment
            // observes the original physical run, not a simulated inline packet.
            let component_following = if suppression {
                selected_following && !end_of_line
            } else {
                source_following
            };
            if component_following {
                contiguous &= !preceding_seen;
                prefix_end = index;
            } else {
                preceding_seen = true;
            }
            all_preceding &= !selected_following;
            if suppression {
                if selected_following {
                    following = true;
                } else {
                    preceding = true;
                }
            }
        }
        let target = if end_of_line {
            TypeSuppressionTarget::Preceding
        } else {
            match (preceding, following) {
                (true, true) => TypeSuppressionTarget::Both,
                (true, false) => TypeSuppressionTarget::Preceding,
                (false, true) => TypeSuppressionTarget::Following,
                (false, false) => TypeSuppressionTarget::Neither,
            }
        };
        let head_projection = (all_preceding
            && matches!(target, TypeSuppressionTarget::Preceding)
            && attachment.prefix.preserves_union_owner)
            .then_some(PrecedingTypeHeadProjection);
        let packet_projection = if end_of_line {
            attachment
                .prefix
                .preserves_union_owner
                .then_some(PrecedingTypePacketProjection::EndOfLine(context))
        } else {
            (all_preceding
                && matches!(target, TypeSuppressionTarget::Preceding)
                && attachment.prefix.preserves_union_owner)
                .then_some(PrecedingTypePacketProjection::Inline)
        };
        let preceding_count = (contiguous
            && prefix_end != 0
            && matches!(target, TypeSuppressionTarget::Preceding)
            && attachment.prefix.preserves_union_owner)
            .then_some(prefix_end);
        ResolvedTypeSuppressionRun {
            selection: TypeSuppressionRun {
                target,
                head,
                packet_projection,
                head_projection,
                preceding_count,
            },
            // Root's constructor additionally requires its original inline,
            // horizontal-owner proof and a nonempty following suffix.
            preceding_prefix: preceding_count
                .filter(|&end| {
                    !end_of_line
                        && end < comments.len()
                        && attachment.prefix.preserves_horizontal_owner
                })
                .map(|end| PrecedingTypePrefix { end }),
        }
    }

    pub(super) fn select_type_head_with_attachment(
        &self,
        node: &mut PreparedType,
        attachment: &TypeAttachmentHead,
        position: SuppressionPosition,
    ) {
        if node.source_owner == Some(node.node) {
            return;
        }
        let owner = if matches!(position, SuppressionPosition::OwnLine) && attachment.union {
            match attachment.first_owner.get() {
                Some(owner) => owner,
                None => {
                    let TsTypeData::TsUnionType(union) = self.ast.ts_type_data(attachment.node)
                    else {
                        unreachable!("prepared union attachment retains its union node");
                    };
                    let first = union
                        .types(self.ast)
                        .first()
                        .map_or(attachment.node, |id| self.ast.get_node_in_sub_range(id));
                    let owner = self.type_attachment_head(first).node;
                    attachment.first_owner.set(Some(owner));
                    owner
                }
            }
        } else {
            attachment.node
        };
        self.select_type_source_owner(node, owner, attachment);
    }

    fn select_type_source_owner(
        &self,
        node: &mut PreparedType,
        owner: TsType,
        attachment: &TypeAttachmentHead,
    ) {
        node.source_owner = Some(merge_type_source_owner(
            node.source_owner,
            owner,
            attachment.node,
            self.ast,
        ));
        node.suppressed_projection = None;
    }

    pub(super) fn select_type_head_with_retention(
        &self,
        node: &mut PreparedType,
        attachment: &TypeAttachmentHead,
        head: TypeHeadRetention,
        position: SuppressionPosition,
    ) {
        head.apply(node, self.ast);
        self.select_type_head_with_attachment(node, attachment, position);
    }

    /// Resolve every owner and retained frame before preparing one emission view.
    pub(super) fn finalize_type_head(
        &self,
        node: &mut PreparedType,
        attachment: &TypeAttachmentHead,
        packet: &[Comment],
    ) {
        node.suppressed_projection = None;
        if node.source_owner.is_none() {
            return;
        }
        if attachment.union
            && attachment.first_owner.get() == node.source_owner
            && node.source_head == attachment.source_barrier
            && !attachment.prefix.preserves_horizontal_owner
            && let Some(carrier) = attachment.prefix_carrier
        {
            TypeHeadRetention::Source(carrier).apply(node, self.ast);
            return;
        }
        node.suppressed_projection = self.prepare_suppressed_type_projection(
            node,
            attachment,
            attachment.first_owner.get(),
            packet,
        );
    }

    fn resolve_type_head_comments(
        &self,
        node: &mut PreparedType,
        attachment: &TypeAttachmentHead,
        comments: &[Comment],
        end: u32,
    ) {
        let head = self.prepare_type_suppression_head(attachment, comments);
        for (_, position) in self.suppressed_comment_runs(comments, end) {
            self.select_type_head_with_retention(node, attachment, head, position);
        }
    }

    fn select_type_head_comments(&self, node: &mut PreparedType, comments: &[Comment], end: u32) {
        if !self.has_suppression || self.content_mode(comments) != ContentMode::Verbatim {
            return;
        }
        let attachment = self.type_attachment_head(node.node);
        self.resolve_type_head_comments(node, &attachment, comments, end);
        self.finalize_type_head(node, &attachment, comments);
    }
    fn format_prepared_type(
        &mut self,
        node: PreparedType,
        context: TypeContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_prepared_type_with_layout(node, context, TypeLayout::Root, writer)
    }

    pub(super) fn format_prepared_type_with_layout(
        &mut self,
        node: PreparedType,
        context: TypeContext,
        layout: TypeLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_prepared_type_completed(node, context, layout, writer, |pending, _| pending)
    }

    fn format_prepared_type_output(
        &mut self,
        node: PreparedType,
        context: TypeContext,
        layout: TypeLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<ExpressionOutput<'ast>, FormatError> {
        self.format_prepared_type_completed(node, context, layout, writer, |pending, right_edge| {
            ExpressionOutput {
                pending,
                right_edge,
            }
        })
    }

    fn format_prepared_type_completed<R>(
        &mut self,
        mut node: PreparedType,
        context: TypeContext,
        layout: TypeLayout,
        writer: &mut Writer<'ast>,
        finish: impl FnOnce(Option<ExpressionTail<'ast>>, ExpressionRightEdge) -> R,
    ) -> Result<R, FormatError> {
        if let Some(projection) = node.suppressed_projection()
            && let Some(head) = self.projected_suppressed_type(projection)
        {
            if !head.comments.is_empty() {
                let recorded = self.record_type_head(
                    PreparedTypeContent::Type(node),
                    PendingTypeHead::new(&[], node.node.span(self.ast).start, None),
                    context,
                    writer,
                )?;
                let output =
                    self.format_recorded_type_head_output(recorded, context, layout, writer)?;
                return Ok(finish(output.pending, output.right_edge));
            }
            node = head.node;
        }
        if node.source_owner != Some(node.node)
            && let TsTypeData::TsParenthesizedType(parentheses) = node.data
        {
            let parentheses = self.prepare_type_parentheses(
                parentheses,
                node.source_owner,
                node.source_head,
                context,
            );
            return self
                .format_prepared_type_parentheses(parentheses, context, layout, writer)
                .map(|pending| finish(pending, ExpressionRightEdge::Unreported));
        }
        if node.source_owner != Some(node.node)
            && let TsTypeData::TsIntersectionType(intersection) = node.data
            && intersection.types(self.ast).len() > 1
        {
            let intersection = self.prepare_intersection(node, intersection.types(self.ast))?;
            return self
                .format_prepared_intersection(intersection, context, layout, writer)
                .map(|pending| finish(pending, ExpressionRightEdge::Unreported));
        }
        if node.source_owner.is_none() {
            let single = match node.data {
                TsTypeData::TsUnionType(union) => {
                    let types = union.types(self.ast);
                    (types.len() == 1).then_some((types, compound::CompoundTypeKind::Union))
                }
                TsTypeData::TsIntersectionType(intersection) => {
                    let types = intersection.types(self.ast);
                    (types.len() == 1).then_some((types, compound::CompoundTypeKind::Intersection))
                }
                _ => None,
            };
            if let Some((types, kind)) = single {
                return self
                    .format_compound_type(node, types, kind, context, layout, writer)
                    .map(|pending| finish(pending, ExpressionRightEdge::Unreported));
            }
        }
        if matches!(context, TypeContext::TypeExpressionRhs) && node.source_owner == Some(node.node)
        {
            let pending = self.format_type_data(node, layout, TypeEnclosure::None, writer)?;
            return Ok(finish(
                pending,
                ExpressionRightEdge::SourceFenceCandidate(CopiedRawOwner::Type(node.node)),
            ));
        }
        let parenthesize = context.needs_parentheses(node.data, self.ast);
        self.format_type_in_context_with_enclosure(
            node.data,
            context,
            layout,
            parenthesize,
            writer,
            |formatter, layout, enclosure, writer| {
                formatter.format_type_data(node, layout, enclosure, writer)
            },
        )
        .map(|pending| finish(pending, ExpressionRightEdge::Unreported))
    }

    fn format_type_in_context(
        &mut self,
        data: TsTypeData,
        context: TypeContext,
        layout: TypeLayout,
        writer: &mut Writer<'ast>,
        print: impl FnOnce(
            &mut Self,
            TypeLayout,
            &mut Writer<'ast>,
        ) -> Result<Option<ExpressionTail<'ast>>, FormatError>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_type_in_context_with_enclosure(
            data,
            context,
            layout,
            context.needs_parentheses(data, self.ast),
            writer,
            |formatter, layout, _, writer| print(formatter, layout, writer),
        )
    }

    fn format_type_in_context_with_enclosure(
        &mut self,
        data: TsTypeData,
        context: TypeContext,
        layout: TypeLayout,
        parenthesize: bool,
        writer: &mut Writer<'ast>,
        print: impl FnOnce(
            &mut Self,
            TypeLayout,
            TypeEnclosure,
            &mut Writer<'ast>,
        ) -> Result<Option<ExpressionTail<'ast>>, FormatError>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        if parenthesize {
            // Unions and nested conditional tests own a block frame.
            // Other types keep their own breaks inside tight parentheses.
            let block = matches!(
                (data, context),
                (TsTypeData::TsUnionType(_), _)
                    | (
                        TsTypeData::TsConditionalType(_),
                        TypeContext::ConditionalCheck | TypeContext::ConditionalExtends
                    )
            );
            if !block {
                if matches!(data, TsTypeData::TsIntersectionType(node)
                    if !self.comments.in_range(node.span(self.ast)).is_empty())
                {
                    let (content, tail) = writer.record_with_value(|writer| {
                        print(self, TypeLayout::Root, TypeEnclosure::Parentheses, writer)
                    })?;
                    let own_line = tail.as_ref().is_some_and(|tail| {
                        self.comment_run_has_own_line_tail(tail.comments, tail.source_end)
                    });
                    if own_line {
                        return writer.group(false, |writer| {
                            writer.text("(");
                            let gap = writer.indent(|writer| {
                                writer.soft_line();
                                writer.write_fragment(content);
                                let gap = self.type_tail_closing_gap(tail.as_ref());
                                self.format_expression_tail(tail, writer)?;
                                Ok::<_, FormatError>(gap)
                            })?;
                            if let Some(gap) = gap {
                                gap.write(writer, FlatSeparator::Empty, |_| {});
                            } else {
                                writer.soft_line();
                            }
                            writer.text(")");
                            Ok(None)
                        });
                    }
                    writer.text("(");
                    writer.write_fragment(content);
                    self.format_type_tail_before_closer(tail, writer)?;
                    writer.text(")");
                    return Ok(None);
                }
                writer.text("(");
                let tail = print(self, TypeLayout::Root, TypeEnclosure::Parentheses, writer)?;
                self.format_type_tail_before_closer(tail, writer)?;
                writer.text(")");
                return Ok(None);
            }
            return writer.group(false, |writer| {
                writer.text("(");
                let gap = writer.indent(|writer| {
                    writer.soft_line();
                    let tail = print(
                        self,
                        TypeLayout::CallerIndented,
                        TypeEnclosure::Parentheses,
                        writer,
                    )?;
                    let gap = self.type_tail_closing_gap(tail.as_ref());
                    self.format_expression_tail(tail, writer)?;
                    Ok::<_, FormatError>(gap)
                })?;
                if let Some(gap) = gap {
                    gap.write(writer, FlatSeparator::Empty, |_| {});
                } else {
                    writer.soft_line();
                }
                writer.text(")");
                Ok(None)
            });
        }
        print(self, layout, context.enclosure(), writer)
    }

    fn type_tail_closing_gap(&self, tail: Option<&ExpressionTail<'ast>>) -> Option<CommentGap> {
        tail.and_then(|tail| {
            self.comment_run_has_own_line_tail(tail.comments, tail.source_end)
                .then(|| {
                    let end = tail
                        .comments
                        .last()
                        .map_or(tail.source_end, |comment| comment.span.end);
                    self.prepare_comment_boundary(tail.comments, tail.source_end, end)
                        .gap()
                })
        })
    }

    /// Finish a directly emitted line before a following type delimiter. An
    /// inline line suffix stays queued so its punctuation owner can precede it.
    fn format_type_tail_before_closer(
        &mut self,
        tail: Option<ExpressionTail<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let gap = self.type_tail_closing_gap(tail.as_ref());
        self.format_expression_tail(tail, writer)?;
        if let Some(gap) = gap {
            gap.write(writer, FlatSeparator::Empty, |_| {});
        }
        Ok(())
    }

    fn format_type_data(
        &mut self,
        node: PreparedType,
        layout: TypeLayout,
        enclosure: TypeEnclosure,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        if node.source_owner == Some(node.node) {
            self.format_verbatim(node.node.span(self.ast), writer)?;
            return Ok(None);
        }
        let keyword = match node.data {
            TsTypeData::TsAnyKeyword(_) => Some("any"),
            TsTypeData::TsUnknownKeyword(_) => Some("unknown"),
            TsTypeData::TsNeverKeyword(_) => Some("never"),
            TsTypeData::TsVoidKeyword(_) => Some("void"),
            TsTypeData::TsNullKeyword(_) => Some("null"),
            TsTypeData::TsUndefinedKeyword(_) => Some("undefined"),
            TsTypeData::TsStringKeyword(_) => Some("string"),
            TsTypeData::TsNumberKeyword(_) => Some("number"),
            TsTypeData::TsBigIntKeyword(_) => Some("bigint"),
            TsTypeData::TsBooleanKeyword(_) => Some("boolean"),
            TsTypeData::TsSymbolKeyword(_) => Some("symbol"),
            TsTypeData::TsObjectKeyword(_) => Some("object"),
            TsTypeData::TsIntrinsicKeyword(_) => Some("intrinsic"),
            TsTypeData::TsThisType(_) => Some("this"),
            _ => None,
        };
        if let Some(keyword) = keyword {
            writer.text(keyword);
            return Ok(None);
        }
        match node.data {
            TsTypeData::TsJsDocUnknownType(_) => {
                writer.text("?");
                Ok(None)
            }
            TsTypeData::TsJsDocNullableType(owner) => self.format_jsdoc_type(
                postfix::JsDocTypeOwner::Nullable(owner),
                node.source_owner,
                enclosure,
                writer,
            ),
            TsTypeData::TsJsDocNonNullableType(owner) => self.format_jsdoc_type(
                postfix::JsDocTypeOwner::NonNullable(owner),
                node.source_owner,
                enclosure,
                writer,
            ),
            TsTypeData::TsImportType(node) => self.format_import_type(node, writer),
            TsTypeData::TsMappedType(node) => self.format_mapped_type(node, writer),
            TsTypeData::TsInferType(node) => self.format_infer_type(node, writer),
            TsTypeData::TsConditionalType(node) => {
                self.format_conditional_type(node, layout, writer)
            }
            TsTypeData::TsTypeLiteral(node) => self.format_type_literal(node, layout, writer),
            TsTypeData::TsFunctionType(node) => {
                self.format_callable_type(callable::CallableType::Function(node), writer)
            }
            TsTypeData::TsConstructorType(node) => {
                self.format_callable_type(callable::CallableType::Constructor(node), writer)
            }
            TsTypeData::TsTypePredicate(node) => self.format_type_predicate(node, writer),

            TsTypeData::TsTupleType(node) => self.format_tuple_type(node, writer),
            TsTypeData::TsOptionalType(node) => self.format_optional_type(node, writer),
            TsTypeData::TsRestType(node) => self.format_rest_type(node, writer),

            TsTypeData::TsUnionType(union) => self.format_compound_type(
                node,
                union.types(self.ast),
                compound::CompoundTypeKind::Union,
                TypeContext::Root,
                layout,
                writer,
            ),
            TsTypeData::TsIntersectionType(intersection) => self.format_compound_type(
                node,
                intersection.types(self.ast),
                compound::CompoundTypeKind::Intersection,
                TypeContext::Root,
                layout,
                writer,
            ),

            TsTypeData::TsArrayType(node) => self.format_array_type(node, writer),
            TsTypeData::TsIndexedAccessType(node) => self.format_indexed_type(node, writer),
            TsTypeData::TsTypeOperator(node) => self.format_type_operator(node, writer),
            TsTypeData::TsTypeQuery(node) => self.format_type_query(node, writer),

            TsTypeData::TsTypeReference(reference) => self.format_type_reference(reference, writer),
            TsTypeData::TsTemplateLiteralType(template) => {
                self.format_template_literal_type(template, writer)?;
                Ok(None)
            }
            TsTypeData::TsLiteralType(literal) => {
                match self.ast.ts_literal_data(literal.literal(self.ast)) {
                    TsLiteralData::TemplateLiteral(template) => {
                        self.format_zero_slot_template(template, writer)?;
                    }
                    TsLiteralData::StringLiteral(node) => {
                        self.format_string_literal(node, StringContext::Expression, writer)
                    }
                    TsLiteralData::NumericLiteral(node) => {
                        self.format_numeric_literal(node, writer)
                    }
                    TsLiteralData::BigIntLiteral(node) => self.format_bigint_literal(node, writer),
                    TsLiteralData::BooleanLiteral(node) => writer.text(if node.value(self.ast) {
                        "true"
                    } else {
                        "false"
                    }),
                    TsLiteralData::UnaryExpression(node) => {
                        if node.operator(self.ast) != UnaryOperator::Negate {
                            return Err(self.unsupported("type literal operator", node));
                        }
                        let argument = node.argument(self.ast);
                        let span =
                            Span::new(node.span(self.ast).start + 1, argument.span(self.ast).start);
                        let mode = if self.has_suppression {
                            self.content_mode(self.comments.in_range(span))
                        } else {
                            ContentMode::Formatted
                        };
                        writer.text("-");
                        if mode == ContentMode::Verbatim {
                            self.format_verbatim(span, writer)?;
                            self.format_verbatim(argument.span(self.ast), writer)?;
                        } else {
                            self.format_source_token_gap(span.start, span.end, false, writer)?;
                            match self.ast.expr_data(argument) {
                                ExprData::NumericLiteral(node) => {
                                    self.format_numeric_literal(node, writer)
                                }
                                ExprData::BigIntLiteral(node) => {
                                    self.format_bigint_literal(node, writer)
                                }
                                _ => return Err(self.unsupported("type literal operand", argument)),
                            }
                        }
                    }
                }
                Ok(None)
            }
            _ => Err(self.unsupported("TypeScript type", node.node)),
        }
    }

    pub(super) fn format_type_name(
        &mut self,
        name: TsTypeName,
        mode: ContentMode,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        if mode == ContentMode::Verbatim {
            return self.format_verbatim(name.span(self.ast), writer);
        }
        match self.ast.ts_type_name_data(name) {
            TsTypeNameData::IdentifierReference(identifier) => {
                writer.text(self.ast.get_utf8(identifier.name(self.ast)));
                Ok(())
            }
            TsTypeNameData::TsQualifiedName(name) => self.format_qualified_type_name(name, writer),
        }
    }

    pub(super) fn format_qualified_type_name(
        &mut self,
        name: TsQualifiedName,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
        let left = name.left(ast);
        let right = name.right(ast);
        let boundary = self.prepare_type_boundary(
            Span::new(left.span(ast).end, right.span(ast).start),
            TypeBoundaryTarget::Qualified,
        )?;
        self.format_qualified_type_left(left, boundary.left_mode, writer)?;
        self.format_type_boundary(&boundary, writer)?;
        if boundary.right_mode == ContentMode::Verbatim {
            self.format_verbatim(right.span(ast), writer)
        } else {
            writer.text(ast.get_utf8(right.name(ast)));
            Ok(())
        }
    }

    fn format_qualified_type_left(
        &mut self,
        left: TsQualifiedNameLeft,
        mode: ContentMode,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        if mode == ContentMode::Verbatim {
            return self.format_verbatim(left.span(self.ast), writer);
        }
        match self.ast.ts_qualified_name_left_data(left) {
            TsQualifiedNameLeftData::IdentifierReference(node) => {
                writer.text(self.ast.get_utf8(node.name(self.ast)))
            }
            TsQualifiedNameLeftData::IdentifierName(node) => {
                writer.text(self.ast.get_utf8(node.name(self.ast)))
            }
            TsQualifiedNameLeftData::BindingIdentifier(node) => {
                writer.text(self.ast.get_utf8(node.name(self.ast)))
            }
            TsQualifiedNameLeftData::ThisExpression(_) => writer.text("this"),
            TsQualifiedNameLeftData::TsQualifiedName(node) => {
                return self.format_qualified_type_name(node, writer);
            }
        }
        Ok(())
    }

    fn prepare_type_parentheses(
        &self,
        node: TsParenthesizedType,
        source_owner: Option<TsType>,
        source_head: Option<TsType>,
        context: TypeContext,
    ) -> PreparedTypeParentheses<'ast> {
        let ast = self.ast;
        let span = node.span(ast);
        let child = node.type_annotation(ast);
        let child_span = child.span(ast);
        let mut child_type = self.prepare_type(child, ContentMode::Formatted);
        child_type.source_owner = source_owner;
        child_type.source_head =
            source_head.filter(|&owner| owner != TsType::TsParenthesizedType(node));
        let before = self
            .comments
            .in_range(Span::new(span.start + 1, child_span.start));
        let after = self
            .comments
            .in_range(Span::new(child_span.end, span.end - 1));
        let sensitive = self.diagnostic_range_is_sensitive(span);
        // Retained attachment pairs keep their source-envelope spine. A retained
        // singleton operator keeps its leading comment carrier because attachment
        // projects through that operator. Raw content ownership stays separate.
        let source_frame = source_head.is_some_and(|head| match self.ast.ts_type_data(head) {
            TsTypeData::TsParenthesizedType(_) => true,
            TsTypeData::TsUnionType(union) => !before.is_empty() && union.types(ast).len() == 1,
            TsTypeData::TsIntersectionType(intersection) => {
                !before.is_empty() && intersection.types(ast).len() == 1
            }
            _ => false,
        });
        let raw_frame = matches!(context, TypeContext::TypeExpressionRhs)
            && source_owner == Some(child)
            && raw_type_has_open_end(child, ast);
        let retained =
            source_frame || raw_frame || !before.is_empty() || !after.is_empty() || sensitive;
        let mut selected_attachment = None;
        let (source, movable, trailing_unbound) = if retained {
            if self.has_suppression && self.content_mode(before) == ContentMode::Verbatim {
                if matches!(context, TypeContext::UnionMember) {
                    child_type.source_owner = Some(child);
                } else {
                    let attachment = self.type_attachment_head(child_type.node);
                    self.resolve_type_head_comments(
                        &mut child_type,
                        &attachment,
                        before,
                        child_span.start,
                    );
                    selected_attachment = Some(attachment);
                }
            }
            if self.content_mode(after) == ContentMode::Verbatim {
                let member_tail = self
                    .suppressed_comment_runs(after, span.end - 1)
                    .all(|(_, position)| matches!(position, SuppressionPosition::EndOfLine));
                let owner = if matches!(
                    context,
                    TypeContext::CompoundMember
                        | TypeContext::UnionMember
                        | TypeContext::IntersectionMember { .. }
                ) && member_tail
                    && let TsTypeData::TsUnionType(union) = child_type.data
                {
                    union
                        .types(ast)
                        .last()
                        .map_or(child, |id| ast.get_node_in_sub_range(id))
                } else {
                    child
                };
                child_type.source_owner = Some(match child_type.source_owner {
                    Some(previous) if previous != owner => child,
                    _ => owner,
                });
            }
            if let Some(attachment) = selected_attachment {
                self.finalize_type_head(&mut child_type, &attachment, before);
            }
            let target = child_type
                .source_owner
                .filter(|&owner| owner != child)
                .map(|owner| owner.span(ast));
            let leading_target = target.filter(|_| {
                before.iter().all(|&comment| {
                    !is_suppression_comment(ast, comment) || comment.kind == CommentKind::Line
                })
            });
            let trailing_target = target.filter(|_| {
                after.iter().all(|&comment| {
                    !is_suppression_comment(ast, comment) || comment.kind == CommentKind::Line
                })
            });
            let leading_binding = self.comment_binding(before, span, leading_target);
            let source = source_frame
                || raw_frame
                || source_owner == Some(TsType::TsParenthesizedType(node))
                || sensitive
                || !leading_binding.allows_relocation(leading_target);
            let trailing_binding = if source {
                CommentBinding::SourceBound
            } else {
                self.comment_binding(after, span, trailing_target)
            };
            let source = source || !trailing_binding.allows_relocation(trailing_target);
            let movable =
                after.is_empty() && !source && matches!(leading_binding, CommentBinding::Unbound);
            (
                source,
                movable,
                matches!(trailing_binding, CommentBinding::Unbound),
            )
        } else {
            (
                source_owner == Some(TsType::TsParenthesizedType(node)),
                false,
                true,
            )
        };
        PreparedTypeParentheses {
            span,
            child_span,
            child: child_type,
            retained,
            movable,
            leading_comments: before,
            trailing: self.prepare_comment_boundary(after, child_span.end, span.end - 1),
            trailing_unbound,
            source,
        }
    }

    fn format_prepared_type_parentheses(
        &mut self,
        parentheses: PreparedTypeParentheses<'ast>,
        context: TypeContext,
        layout: TypeLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let PreparedTypeParentheses {
            span,
            child_span,
            child,
            retained,
            movable,
            leading_comments,
            trailing,
            trailing_unbound,
            source,
        } = parentheses;
        if movable && !context.preserves_union_head(child.node, self.ast) {
            let content = self.format_movable_type_head(
                child,
                &mut PendingTypeHead::new(
                    leading_comments,
                    child_span.start,
                    self.type_movable_head(child, context),
                ),
                context,
                writer,
            )?;
            return self.format_prepared_type_content(content, context, layout, writer);
        }
        if !retained {
            return self.format_prepared_type_with_layout(child, context, layout, writer);
        }
        // A trailing packet keeps its delimiter fence until a compound separator
        // supplies the next owner. Leaf pairs keep their existing compact frame;
        // compound envelopes prepare the member packet and its closing line.
        let transfers_tail = matches!(context, TypeContext::IntersectionMember { .. });
        let fenced_tail = !trailing.is_empty()
            && (!matches!(
                context,
                TypeContext::CompoundMember
                    | TypeContext::UnionMember
                    | TypeContext::IntersectionMember { .. }
            ) || transfers_tail && !trailing_unbound);
        let compact_tail = fenced_tail
            && !matches!(
                child.data,
                TsTypeData::TsUnionType(_) | TsTypeData::TsIntersectionType(_)
            );
        if source || compact_tail {
            if source && child.source_owner == Some(child.node) {
                self.format_verbatim(span, writer)?;
                return Ok(None);
            }
            return writer.group(false, |writer| {
                writer.text("(");
                self.format_verbatim(Span::new(span.start + 1, child_span.start), writer)?;
                let tail = self.format_prepared_type_with_layout(
                    child,
                    TypeContext::Parenthesized,
                    TypeLayout::OpeningProvided,
                    writer,
                )?;
                self.format_expression_tail(tail, writer)?;
                self.format_verbatim(Span::new(child_span.end, span.end - 1), writer)?;
                writer.text(")");
                Ok(None)
            });
        }
        let leading =
            self.prepare_comment_boundary(leading_comments, span.start + 1, child_span.start);
        if matches!(
            context,
            TypeContext::IntersectionMember {
                line_continuation: true,
                ..
            }
        ) && trailing.is_empty()
        {
            self.format_comment_boundary_with_layout(
                &leading,
                CommentRunLayout::GroupedLeading(child_span.start),
                writer,
            )?;
            return self.format_prepared_type_with_layout(child, context, layout, writer);
        }
        let enclosed_head = !leading_comments.is_empty() && !movable;
        if !fenced_tail && !enclosed_head && !context.needs_parentheses(child.data, self.ast) {
            self.format_comment_boundary_with_layout(
                &leading,
                CommentRunLayout::GroupedLeading(child_span.start),
                writer,
            )?;
            if transfers_tail && !trailing.is_empty() {
                let pending =
                    self.format_prepared_type_with_layout(child, context, layout, writer)?;
                let source_end = pending
                    .as_ref()
                    .map_or(trailing.source_end(), |tail| tail.source_end);
                let comments = if let Some(pending) = pending {
                    let comments = self
                        .comments
                        .in_range(Span::new(source_end, trailing.end()));
                    debug_assert!(comments.starts_with(pending.comments));
                    debug_assert!(comments.ends_with(trailing.comments()));
                    comments
                } else {
                    trailing.comments()
                };
                return Ok(Some(ExpressionTail {
                    source_end,
                    comments,
                }));
            }
            return self
                .format_type_before_trailing_comments(child, context, layout, trailing, writer);
        }
        writer.group(false, |writer| {
            writer.text("(");
            writer.indent(|writer| {
                writer.soft_line();
                self.format_comment_boundary_with_layout(
                    &leading,
                    CommentRunLayout::GroupedLeading(child_span.start),
                    writer,
                )?;
                self.format_type_before_trailing_comments(
                    child,
                    TypeContext::Parenthesized,
                    TypeLayout::CallerIndented,
                    trailing,
                    writer,
                )
            })?;
            writer.soft_line();
            writer.text(")");
            Ok(None)
        })
    }

    fn format_type_before_trailing_comments(
        &mut self,
        child: PreparedType,
        context: TypeContext,
        layout: TypeLayout,
        trailing: CommentBoundary<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let tail = if let TsTypeData::TsUnionType(union) = child.data
            && child.source_owner != Some(child.node)
            && !trailing.is_empty()
        {
            let union = self.prepare_union(child, union.types(self.ast))?;
            self.format_prepared_union(
                union.with_trailing_comments(trailing),
                context,
                layout,
                None,
                writer,
            )?
        } else {
            let tail = self.format_prepared_type_with_layout(child, context, layout, writer)?;
            self.format_expression_tail(tail, writer)?;
            self.format_comment_boundary(&trailing, writer)?;
            None
        };
        self.format_expression_tail(tail, writer)?;
        Ok(None)
    }

    fn record_type_head(
        &mut self,
        content: compound::PreparedTypeContent<'ast>,
        mut pending: PendingTypeHead<'ast>,
        context: TypeContext,
        writer: &mut Writer<'ast>,
    ) -> Result<RecordedTypeHead<'ast>, FormatError> {
        let (head, content) = writer.record_with_value(|writer| {
            self.format_prepared_type_content_head(content, &mut pending, context, writer)
        })?;
        Ok(RecordedTypeHead {
            head,
            content,
            riding: pending.has_riding_prefix(),
        })
    }

    pub(super) fn format_recorded_type_head(
        &mut self,
        recorded: RecordedTypeHead<'ast>,
        context: TypeContext,
        layout: TypeLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_recorded_type_head_output(recorded, context, layout, writer)
            .map(|output| output.pending)
    }

    pub(super) fn format_recorded_type_head_output(
        &mut self,
        recorded: RecordedTypeHead<'ast>,
        context: TypeContext,
        layout: TypeLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<ExpressionOutput<'ast>, FormatError> {
        let RecordedTypeHead { head, content, .. } = recorded;
        if let compound::PreparedTypeContent::Union(union) = content {
            return self
                .format_prepared_union(union, context, layout, Some(head), writer)
                .map(ExpressionOutput::unreported);
        }
        writer.group(false, |writer| {
            writer.write_fragment(head);
            self.format_prepared_type_content_output(content, context, layout, writer)
        })
    }

    fn format_prepared_type_with_caller_head(
        &mut self,
        node: PreparedType,
        leading_head: Option<compound::MovableTypeHead<'ast>>,
        context: TypeContext,
        after: &CommentBoundary<'ast>,
        layout: TypeLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let content = if node.source_head.is_some() {
            self.prepare_operator_rhs_type(node, context)?.content
        } else {
            compound::PreparedTypeContent::Type(node)
        };
        let recorded = self.record_type_head(
            content,
            PendingTypeHead::new(
                after.comments(),
                node.node.span(self.ast).start,
                leading_head,
            ),
            context,
            writer,
        )?;
        self.format_recorded_type_head(recorded, context, layout, writer)
    }

    fn annotation_geometry(
        &self,
        gap: Span,
        colon: Span,
        comments: &'ast [Comment],
    ) -> AnnotationGeometry<'ast> {
        let split = comments.partition_point(|comment| comment.span.end <= colon.start);
        let (before, after) = comments.split_at(split);
        AnnotationGeometry {
            gap,
            colon,
            before,
            after,
            before_mode: if self.has_suppression {
                self.content_mode(before)
            } else {
                ContentMode::Formatted
            },
        }
    }

    fn annotation_source_prefix(
        &self,
        content: &AnnotationContent,
        gap_end: u32,
        start: u32,
    ) -> Option<Span> {
        match content {
            AnnotationContent::Source(annotation) => {
                Some(Span::new(start, annotation.span(self.ast).start))
            }
            AnnotationContent::Formatted(node) if node.source_owner.is_some() => {
                Some(Span::new(start, gap_end))
            }
            AnnotationContent::Formatted(_) => None,
        }
    }

    pub(super) fn prepare_binding_annotation_content(
        &self,
        preceding_end: u32,
        annotation: TsTypeAnnotation,
        type_argument_owner: TypeArgumentOwner,
    ) -> Result<PreparedBindingAnnotation<'ast>, FormatError> {
        let node = annotation.type_annotation(self.ast);
        let gap = Span::new(preceding_end, node.span(self.ast).start);
        let comments = self.comments.in_range(gap);
        let Some(colon_start) = self.find_token(gap, comments, ":") else {
            return Err(self.unsupported("type annotation colon", annotation));
        };
        let colon_span = Span::new(colon_start, colon_start + 1);
        let geometry = self.annotation_geometry(gap, colon_span, comments);
        let leading = geometry.before;
        let colon = geometry.colon.start;
        let before_leading = self.line_constraint(
            preceding_end,
            leading.first().map_or(colon, |comment| comment.span.start),
        );
        let leading_source_gap = if geometry.before_mode == ContentMode::Verbatim {
            Some(
                self.source_line_constraint(
                    leading
                        .last()
                        .map_or(preceding_end, |comment| comment.span.end),
                    colon,
                ),
            )
        } else {
            None
        };
        let content = if geometry.before_mode == ContentMode::Verbatim {
            AnnotationContent::Source(annotation)
        } else {
            let mut prepared = self.prepare_type(node, ContentMode::Formatted);
            self.select_type_head_comments(&mut prepared, geometry.after, geometry.gap.end);
            if prepared.source_owner == Some(prepared.node) {
                let mut selected = prepared.node;
                let mut data = prepared.data;
                while let TsTypeData::TsParenthesizedType(parentheses) = data {
                    selected = parentheses.type_annotation(self.ast);
                    data = self.ast.ts_type_data(selected);
                }
                prepared.source_owner = Some(selected);
            }
            AnnotationContent::Formatted(prepared)
        };
        let prefix = match self.annotation_source_prefix(&content, geometry.gap.end, colon) {
            Some(span) => BindingAnnotationPrefix::Source(span),
            None => BindingAnnotationPrefix::Formatted {
                colon: geometry.colon,
                before: self.line_constraint(
                    geometry.colon.end,
                    geometry
                        .after
                        .first()
                        .map_or(geometry.gap.end, |comment| comment.span.start),
                ),
                comments: geometry.after,
                end: geometry.gap.end,
            },
        };
        Ok(PreparedBindingAnnotation {
            leading,
            before_leading,
            leading_source_gap,
            colon,
            prefix,
            content,
            type_argument_owner,
        })
    }

    pub(super) fn format_binding_annotation_content(
        &mut self,
        annotation: PreparedBindingAnnotation<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        if !annotation.leading.is_empty() {
            annotation
                .before_leading
                .write(writer, FlatSeparator::Space, Writer::space);
            self.comments.claim(annotation.leading)?;
            let layout = if annotation.leading_source_gap.is_some() {
                CommentRunLayout::Source
            } else {
                CommentRunLayout::FlatLeading(annotation.colon)
            };
            self.format_comment_run(annotation.leading, layout, writer)?;
            if let Some(gap) = annotation.leading_source_gap {
                gap.write(writer, FlatSeparator::Space, Writer::space);
            }
        }
        let layout = match annotation.prefix {
            BindingAnnotationPrefix::Source(span) => {
                self.format_verbatim(span, writer)?;
                TypeLayout::OpeningProvided
            }
            BindingAnnotationPrefix::Formatted {
                colon,
                before,
                comments,
                end,
            } => {
                writer.text(self.source(colon));
                let candidate = match &annotation.content {
                    AnnotationContent::Formatted(node)
                        if node.source_owner.is_none()
                            && matches!(
                                node.data,
                                TsTypeData::TsParenthesizedType(_)
                                    | TsTypeData::TsUnionType(_)
                                    | TsTypeData::TsIntersectionType(_)
                                    | TsTypeData::TsArrayType(_)
                                    | TsTypeData::TsIndexedAccessType(_)
                            ) =>
                    {
                        Some(*node)
                    }
                    _ => None,
                };
                let leading_head = if comments.is_empty() {
                    candidate.and_then(|node| self.type_movable_head(node, TypeContext::Root))
                } else {
                    None
                };
                if let Some(node) = candidate
                    && (!comments.is_empty() || leading_head.is_some())
                {
                    let boundary = self.prepare_comment_boundary(comments, colon.end, end);
                    if matches!(
                        self.comment_boundary_binding(&boundary, node.node.span(self.ast)),
                        CommentBinding::Unbound
                    ) {
                        let leading_head = if comments.is_empty() {
                            leading_head
                        } else {
                            self.type_movable_head(node, TypeContext::Root)
                        };
                        before.write(writer, FlatSeparator::Space, Writer::space);
                        if annotation.type_argument_owner
                            == TypeArgumentOwner::ArrowVariableAnnotation
                        {
                            let recorded = self.record_type_head(
                                compound::PreparedTypeContent::Type(node),
                                PendingTypeHead::new(
                                    boundary.comments(),
                                    node.node.span(self.ast).start,
                                    leading_head,
                                ),
                                TypeContext::Root,
                                writer,
                            )?;
                            return self.format_recorded_binding_annotation_head(
                                recorded,
                                annotation.type_argument_owner,
                                TypeLayout::Root,
                                writer,
                            );
                        }
                        return self.format_prepared_type_with_caller_head(
                            node,
                            leading_head,
                            TypeContext::Root,
                            &boundary,
                            TypeLayout::Root,
                            writer,
                        );
                    }
                }
                before.write(writer, FlatSeparator::Space, Writer::space);
                self.comments.claim(comments)?;
                self.format_comment_run(comments, CommentRunLayout::FlatLeading(end), writer)?;
                TypeLayout::Root
            }
        };
        self.format_binding_annotation_type_content(
            annotation.content,
            annotation.type_argument_owner,
            layout,
            writer,
        )
    }

    pub(super) fn format_typed_variable_declarator(
        &mut self,
        declarator: VariableDeclarator,
        annotation: Option<TsTypeAnnotation>,
        context: VariableContext,
        definition_owner: Option<VariableDeclarator>,
        writer: &mut Writer<'ast>,
    ) -> Result<VariableDeclaratorOutput<'ast>, FormatError> {
        let ast = self.ast;
        let pattern = declarator.id(ast);
        let pattern_span = pattern.span(ast);
        let is_definite = declarator.definite(ast);
        let mut definite = if is_definite && annotation.is_none() {
            Some(self.variable_definite_marker(declarator, annotation))
        } else {
            None
        };
        let marker_end = definite
            .as_ref()
            .and_then(|marker| marker.source)
            .map_or(pattern_span.end, |span| span.end);
        let left_end = annotation.map_or(marker_end, |annotation| annotation.span(ast).end);
        let owner = Span::new(pattern_span.start, left_end);
        let Some(right) = declarator.init(ast) else {
            if is_definite && definite.is_none() {
                definite = Some(self.variable_definite_marker(declarator, annotation));
            }
            let binding = self.prepare_annotated_binding(
                pattern,
                owner,
                definite,
                annotation,
                TypeArgumentOwner::Ordinary,
                ContentMode::Formatted,
            )?;
            let group = self.declaration_binding_group(pattern);
            return self
                .format_annotated_binding(binding, group, writer)
                .map(VariableDeclaratorOutput::formatted);
        };
        let gap = Span::new(left_end, right.span(ast).start);
        let comments = self.comments.in_range(gap);
        let Some(operator) = self.find_token(gap, comments, "=") else {
            return Err(self.unsupported("typed variable initializer boundary", declarator));
        };
        let split = comments.partition_point(|comment| comment.span.end <= operator);
        let (before, after) = comments.split_at(split);
        let mut left_mode = ContentMode::Formatted;
        let mut right_mode = ContentMode::Formatted;
        if self.has_suppression && self.content_mode(comments) == ContentMode::Verbatim {
            for (run, position) in self.suppressed_comment_runs(before, operator) {
                for &comment in run {
                    if !is_suppression_comment(ast, comment) {
                        continue;
                    }
                    if matches!(position, SuppressionPosition::Inline)
                        || matches!(position, SuppressionPosition::EndOfLine)
                            && comment.kind == CommentKind::Line
                    {
                        left_mode = ContentMode::Verbatim;
                    } else {
                        right_mode = ContentMode::Verbatim;
                    }
                }
            }
            for (run, position) in self.suppressed_comment_runs(after, gap.end) {
                for &comment in run {
                    if !is_suppression_comment(ast, comment) {
                        continue;
                    }
                    if matches!(position, SuppressionPosition::EndOfLine)
                        && comment.kind == CommentKind::Line
                    {
                        left_mode = ContentMode::Verbatim;
                    } else {
                        right_mode = ContentMode::Verbatim;
                    }
                }
            }
        }
        let mut context = self.variable_initializer_context(right, context);
        let is_identifier_annotation = annotation.is_some()
            && matches!(
                ast.binding_pattern_data(pattern),
                BindingPatternData::BindingIdentifier(_)
            );
        let annotation_owner = |data| {
            if is_identifier_annotation && matches!(data, ExprData::ArrowFunctionExpression(_)) {
                TypeArgumentOwner::ArrowVariableAnnotation
            } else {
                TypeArgumentOwner::Ordinary
            }
        };
        let (group, layout, type_argument_owner) = if left_mode == ContentMode::Verbatim
            || right_mode == ContentMode::Verbatim
        {
            let group = if left_mode == ContentMode::Formatted {
                self.bound_initializer_binding_group(pattern)
            } else {
                BindingGroup::Auto
            };
            let content = (left_mode == ContentMode::Formatted && is_identifier_annotation
                || right_mode == ContentMode::Formatted && definition_owner == Some(declarator))
            .then(|| self.expression_content(right));
            if right_mode == ContentMode::Formatted && definition_owner == Some(declarator) {
                context = context.with_decorated_function_call(
                    content
                        .as_ref()
                        .and_then(|content| self.decorated_function_call_content(content)),
                );
            }
            let type_argument_owner = match content.as_ref() {
                Some(content)
                    if left_mode == ContentMode::Formatted && is_identifier_annotation =>
                {
                    let data = self.source_parentheses_rhs_content(
                        content,
                        context.with_content_mode(right_mode),
                    );
                    annotation_owner(data)
                }
                _ => TypeArgumentOwner::Ordinary,
            };
            (
                group,
                InitializerLayout::Bound { gap, right_mode },
                type_argument_owner,
            )
        } else {
            let content = self.expression_content(right);
            if definition_owner == Some(declarator) {
                context = context
                    .with_decorated_function_call(self.decorated_function_call_content(&content));
            }
            let rhs = self.assignment_rhs(&content);
            let boundary = self.assignment_boundary(
                left_end,
                &rhs,
                Span::new(operator, operator + 1),
                right.span(self.ast).end,
                context,
            )?;
            let data = if is_identifier_annotation {
                self.source_parentheses_rhs_content(
                    &content,
                    context.with_content_mode(boundary.mode),
                )
            } else {
                content.data
            };
            let type_argument_owner = annotation_owner(data);
            let rhs_layout = self.ordinary_rhs_layout(&rhs, context, &boundary);
            let (group, rhs_layout) = self.binding_initializer_layout(
                pattern,
                BindingInitializerOwner::Variable { annotation },
                content.data,
                &boundary,
                rhs_layout,
            );
            (
                group,
                InitializerLayout::Formatted {
                    boundary,
                    rhs_layout,
                },
                type_argument_owner,
            )
        };
        group.with_initializer_group(writer, |writer| {
            if left_mode == ContentMode::Verbatim {
                self.format_verbatim(owner, writer)?;
            } else {
                if is_definite && definite.is_none() {
                    definite = Some(self.variable_definite_marker(declarator, annotation));
                }
                let binding = self.prepare_annotated_binding(
                    pattern,
                    owner,
                    definite,
                    annotation,
                    type_argument_owner,
                    ContentMode::Formatted,
                )?;
                let tail = self.format_annotated_binding(binding, group, writer)?;
                self.format_expression_tail(tail, writer)?;
            }
            self.format_initializer_right_with_raw_end(left_end, right, layout, context, writer)
                .map(VariableDeclaratorOutput::initializer)
        })
    }

    /// Resolve and claim a movable head before the delimiter chooses its opening.
    pub(super) fn prepare_delimited_type_head(
        &mut self,
        body: PreparedDelimitedType<'ast>,
        after: CommentBoundary<'ast>,
        context: TypeContext,
        writer: &mut Writer<'ast>,
    ) -> Result<PreparedDelimitedTypeHead<'ast>, FormatError> {
        let (movable_head, leading) = self.prepared_content_head(&body.content, context);
        if ((!after.is_empty() && self.prepared_content_accepts_caller_head(&body.content))
            || movable_head)
            && matches!(
                self.comment_boundary_binding(&after, body.span),
                CommentBinding::Unbound
            )
        {
            let recorded = self.record_type_head(
                body.content,
                PendingTypeHead::new(after.comments(), body.span.start, leading),
                context,
                writer,
            )?;
            return Ok(PreparedDelimitedTypeHead::Recorded(recorded));
        }
        Ok(PreparedDelimitedTypeHead::Boundary(body, after))
    }

    pub(super) fn prepare_binary_type_head(
        &self,
        node: PreparedType,
        context: TypeContext,
        after: &CommentBoundary<'ast>,
        operator_end: u32,
        tail_owner: TailOwner,
    ) -> BinaryTypeHead<'ast> {
        let projected = node
            .suppressed_projection()
            .and_then(|projection| self.projected_suppressed_type(projection));
        let node = match projected {
            Some(head) => head.node,
            None if node.source_owner.is_some() => {
                return BinaryTypeHead(BinaryTypeHeadLayout::Direct);
            }
            None => node,
        };
        if node.source_owner == Some(node.node) {
            return BinaryTypeHead(BinaryTypeHeadLayout::Direct);
        }
        let compound = matches!(
            node.data,
            TsTypeData::TsParenthesizedType(_)
                | TsTypeData::TsUnionType(_)
                | TsTypeData::TsIntersectionType(_)
                | TsTypeData::TsArrayType(_)
                | TsTypeData::TsIndexedAccessType(_)
                | TsTypeData::TsConditionalType(_)
        );
        if !compound && after.is_empty() {
            return BinaryTypeHead(BinaryTypeHeadLayout::Direct);
        }
        let target = node.node.span(self.ast);
        if !matches!(
            self.comment_boundary_binding(after, target),
            CommentBinding::Unbound
        ) {
            return BinaryTypeHead(BinaryTypeHeadLayout::Direct);
        }
        let leading = if compound {
            self.type_movable_head(node, context)
        } else {
            None
        };
        // The opening ends at the first physical RHS boundary, independently
        // of any comment head exposed through removed frames or operators.
        let opening_end = after
            .comments()
            .first()
            .map_or(target.start, |first| first.span.start);
        let empty_opening = match self.comment_gap_constraint(operator_end, opening_end) {
            LineConstraint::NewLine(2) => Some(LineConstraint::NewLine(2)),
            _ => None,
        };
        let (comments, next_start) = if after.is_empty() {
            let Some(head) = leading else {
                let Some(opening) = empty_opening else {
                    return BinaryTypeHead(BinaryTypeHeadLayout::Direct);
                };
                // A compound can keep its own first-member packet, rather than
                // lend it to the caller. Only that real frontier needs this
                // external opening; comments on later members do not qualify.
                let first = match node.data {
                    TsTypeData::TsUnionType(union) => union.types(self.ast).first(),
                    TsTypeData::TsIntersectionType(intersection) => {
                        intersection.types(self.ast).first()
                    }
                    _ => None,
                };
                let owns_head = first.is_some_and(|first| {
                    let first: TsType = self.ast.get_node_in_sub_range(first);
                    !self
                        .comments
                        .in_range(Span::new(target.start, first.span(self.ast).start))
                        .is_empty()
                });
                return BinaryTypeHead(if owns_head {
                    BinaryTypeHeadLayout::Caller {
                        leading: None,
                        own_line: Some(opening),
                    }
                } else {
                    BinaryTypeHeadLayout::Direct
                });
            };
            (head.comments, head.next_start)
        } else {
            (after.comments(), target.start)
        };
        if let TsTypeData::TsTypeReference(reference) = node.data {
            let blocks = comments
                .iter()
                .all(|comment| comment.kind == CommentKind::Block);
            if blocks && empty_opening.is_none() {
                let continuous = comments
                    .windows(2)
                    .all(|pair| self.line_breaks(pair[0].span.end, pair[1].span.start) == 0)
                    && comments
                        .last()
                        .is_some_and(|last| self.line_breaks(last.span.end, next_start) == 0);
                return BinaryTypeHead(if continuous {
                    BinaryTypeHeadLayout::InlineNamed
                } else {
                    BinaryTypeHeadLayout::Named
                });
            }
            // A continuing operation owns another token/comment boundary. Keep
            // this complete head here rather than leasing a queued line across it.
            if tail_owner == TailOwner::Expression
                && reference.type_arguments(self.ast).is_none()
                && let TsTypeNameData::IdentifierReference(name) =
                    self.ast.ts_type_name_data(reference.type_name(self.ast))
                && comments
                    .first()
                    .is_some_and(|first| self.line_breaks(operator_end, first.span.start) == 0)
            {
                return BinaryTypeHead(BinaryTypeHeadLayout::TrailingNamed {
                    name,
                    tail: ExpressionTail {
                        source_end: operator_end,
                        comments,
                    },
                });
            }
        }
        let own_line = empty_opening.or_else(|| {
            let first = comments.first()?;
            let owns_opening = if after.is_empty() {
                // This head is borrowed through a removable syntax frame, not
                // the actual external operator boundary. Keep its proven entry.
                let starts_line = self.source_line_starts_line(first.span.start)
                    || self.line_breaks(first.span.start, first.span.end) != 0;
                starts_line
                    && comments.iter().enumerate().any(|(index, comment)| {
                        let next = comments
                            .get(index + 1)
                            .map_or(next_start, |next| next.span.start);
                        self.line_breaks(comment.span.end, next) != 0
                    })
            } else {
                // Entry belongs to the actual operator-to-head boundary. A
                // generated union bar cannot change that external opening.
                self.line_breaks(operator_end, first.span.start) != 0
            };
            owns_opening.then(|| self.line_constraint(operator_end, first.span.start))
        });
        if own_line.is_none()
            && !compound
            && comments
                .iter()
                .all(|comment| comment.kind == CommentKind::Block)
        {
            return BinaryTypeHead(BinaryTypeHeadLayout::Direct);
        }
        BinaryTypeHead(BinaryTypeHeadLayout::Caller { leading, own_line })
    }

    /// The operator owns the opening space; the head retains required line breaks.
    pub(super) fn format_binary_type_rhs_head(
        &mut self,
        node: PreparedType,
        context: TypeContext,
        after: &CommentBoundary<'ast>,
        head: BinaryTypeHead<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_binary_type_rhs_head_output(node, context, after, head, writer)
            .map(|output| output.pending)
    }

    pub(super) fn format_binary_type_rhs_head_output(
        &mut self,
        node: PreparedType,
        context: TypeContext,
        after: &CommentBoundary<'ast>,
        head: BinaryTypeHead<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<ExpressionOutput<'ast>, FormatError> {
        if matches!(head.0, BinaryTypeHeadLayout::InlineNamed) {
            self.format_comment_boundary_with_layout(
                after,
                CommentRunLayout::FlatLeading(node.node.span(self.ast).start),
                writer,
            )?;
            return self.format_prepared_type_output(node, context, TypeLayout::Root, writer);
        }
        if matches!(head.0, BinaryTypeHeadLayout::Named) {
            let comments = after.comments();
            self.format_comment_boundary_with_layout(after, CommentRunLayout::Source, writer)?;
            if let Some(last) = comments.last() {
                self.comment_gap(last.span.end, node.node.span(self.ast).start, writer);
            }
            return self.format_prepared_type_output(node, context, TypeLayout::Root, writer);
        }
        if let BinaryTypeHeadLayout::TrailingNamed { name, tail } = head.0 {
            // This name has no nested token boundary to consume the leased packet.
            writer.text(self.ast.get_utf8(name.name(self.ast)));
            return Ok(ExpressionOutput::closed(Some(tail)));
        }
        let BinaryTypeHead(BinaryTypeHeadLayout::Caller { leading, own_line }) = head else {
            if after.is_empty() {
                after.gap().write(writer, FlatSeparator::Empty, |_| {});
            } else {
                self.format_comment_boundary_after_separator(after, writer)?;
                after
                    .gap()
                    .write(writer, FlatSeparator::Space, Writer::space);
            }
            return self.format_prepared_type_output(node, context, TypeLayout::Root, writer);
        };
        let mut pending =
            PendingTypeHead::new(after.comments(), node.node.span(self.ast).start, leading)
                .with_opening(CommentOpening::Provided);
        if own_line.is_none() {
            pending = pending.with_inline_prefix();
        }
        // The normal head walk keeps its inline union-head optimization. A
        // retained lexical head stops that walk, but still owns the same actual
        // syntax scope for its external head and intrinsic opening.
        let content = if node.source_head.is_some() {
            self.prepare_operator_rhs_type(node, context)?.content
        } else {
            compound::PreparedTypeContent::Type(node)
        };
        let recorded = self.record_type_head(content, pending, context, writer)?;
        let opening = match own_line {
            _ if recorded.riding => None,
            Some(opening) => Some(opening),
            _ => {
                return self.format_recorded_type_head_output(
                    recorded,
                    context,
                    TypeLayout::Root,
                    writer,
                );
            }
        };
        writer.indent(|writer| {
            if let Some(opening) = opening {
                // This boundary supplied the complete source opening. The RHS
                // keeps its indentation without emitting another opening gap.
                opening.write(writer, FlatSeparator::Empty, Writer::hard_line);
            }
            self.format_recorded_type_head_output(
                recorded,
                context,
                TypeLayout::CallerIndented,
                writer,
            )
        })
    }

    pub(super) fn prepare_qualified_name_boundary(
        &self,
        gap: Span,
    ) -> Result<TypeBoundary<'ast>, FormatError> {
        self.prepare_type_boundary(gap, TypeBoundaryTarget::Qualified)
    }

    pub(super) fn prepare_module_reference_boundary(
        &self,
        gap: Span,
    ) -> Result<TypeBoundary<'ast>, FormatError> {
        self.prepare_type_boundary(gap, TypeBoundaryTarget::ModuleReference)
    }

    fn variable_definite_marker(
        &self,
        declarator: VariableDeclarator,
        annotation: Option<TsTypeAnnotation>,
    ) -> DefiniteMarker {
        if annotation.is_some() {
            return DefiniteMarker {
                source: None,
                movable: false,
            };
        }
        let ast = self.ast;
        let end = declarator
            .init(ast)
            .map_or(declarator.span(ast).end, |right| right.span(ast).start);
        let gap = Span::new(declarator.id(ast).span(ast).end, end);
        let source = self
            .find_token(gap, self.comments.in_range(gap), "!")
            .map(|marker| Span::new(marker, marker + 1));
        let prefix = Span::new(gap.start, source.map_or(gap.start, |span| span.start));
        let comments = self.comments.in_range(prefix);
        let movable = !comments.is_empty()
            && self
                .comment_binding(comments, prefix, None)
                .allows_relocation(None);
        DefiniteMarker { source, movable }
    }
}

/// A trailing JSDoc question mark can consume a type on the following source line.
pub(super) fn type_requires_terminator(node: TsType, ast: &Ast<'_>) -> bool {
    type_data_requires_terminator(ast.ts_type_data(node), ast)
}

fn type_data_requires_terminator(mut data: TsTypeData, ast: &Ast<'_>) -> bool {
    let mut context = TypeContext::Root;
    loop {
        if context.needs_parentheses(data, ast) {
            return false;
        }
        let (next, next_context) = match data {
            TsTypeData::TsJsDocUnknownType(_) | TsTypeData::TsOptionalType(_) => return true,
            TsTypeData::TsJsDocNullableType(node) => {
                if node.postfix(ast) {
                    return true;
                }
                (node.type_annotation(ast), TypeContext::JsDocNullableOperand)
            }
            TsTypeData::TsJsDocNonNullableType(node) if !node.postfix(ast) => {
                (node.type_annotation(ast), TypeContext::JsDocPrimaryOperand)
            }
            TsTypeData::TsParenthesizedType(node) => (node.type_annotation(ast), context),
            TsTypeData::TsUnionType(node) => {
                let Some(last) = node.types(ast).last() else {
                    return false;
                };
                (ast.get_node_in_sub_range(last), TypeContext::UnionMember)
            }
            TsTypeData::TsIntersectionType(node) => {
                let Some(last) = node.types(ast).last() else {
                    return false;
                };
                (ast.get_node_in_sub_range(last), TypeContext::CompoundMember)
            }
            TsTypeData::TsConditionalType(node) => (node.false_type(ast), TypeContext::Root),
            TsTypeData::TsFunctionType(node) => (
                node.return_type(ast).type_annotation(ast),
                TypeContext::Root,
            ),
            TsTypeData::TsConstructorType(node) => (
                node.return_type(ast).type_annotation(ast),
                TypeContext::Root,
            ),
            TsTypeData::TsTypePredicate(node) => {
                let Some(annotation) = node.type_annotation(ast) else {
                    return false;
                };
                (annotation.type_annotation(ast), TypeContext::Root)
            }
            TsTypeData::TsTypeOperator(node) => {
                (node.type_annotation(ast), TypeContext::ModifierOperand)
            }
            _ => return false,
        };
        data = ast.ts_type_data(next);
        context = next_context;
    }
}

/// Unlike formatted types, raw compound members do not acquire generated pairs.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum RawTypeTerminal {
    Closed,
    PostfixQuestion,
    TypeContinuation,
}

pub(super) fn raw_type_has_open_end(node: TsType, ast: &Ast<'_>) -> bool {
    raw_type_terminal(node, ast) != RawTypeTerminal::Closed
}

pub(super) fn raw_type_terminal(mut node: TsType, ast: &Ast<'_>) -> RawTypeTerminal {
    loop {
        node = match ast.ts_type_data(node) {
            TsTypeData::TsJsDocUnknownType(_) => return RawTypeTerminal::TypeContinuation,
            TsTypeData::TsOptionalType(_) => return RawTypeTerminal::PostfixQuestion,
            TsTypeData::TsJsDocNullableType(node) if node.postfix(ast) => {
                return RawTypeTerminal::PostfixQuestion;
            }
            TsTypeData::TsJsDocNullableType(node) => node.type_annotation(ast),
            TsTypeData::TsJsDocNonNullableType(node) if !node.postfix(ast) => {
                node.type_annotation(ast)
            }
            TsTypeData::TsUnionType(node) => {
                let Some(last) = node.types(ast).last() else {
                    return RawTypeTerminal::Closed;
                };
                ast.get_node_in_sub_range(last)
            }
            TsTypeData::TsIntersectionType(node) => {
                let Some(last) = node.types(ast).last() else {
                    return RawTypeTerminal::Closed;
                };
                ast.get_node_in_sub_range(last)
            }
            TsTypeData::TsConditionalType(node) => node.false_type(ast),
            TsTypeData::TsFunctionType(node) => node.return_type(ast).type_annotation(ast),
            TsTypeData::TsConstructorType(node) => node.return_type(ast).type_annotation(ast),
            TsTypeData::TsTypePredicate(node) => {
                let Some(annotation) = node.type_annotation(ast) else {
                    return RawTypeTerminal::Closed;
                };
                annotation.type_annotation(ast)
            }
            TsTypeData::TsTypeOperator(node) => node.type_annotation(ast),
            TsTypeData::TsInferType(node) => {
                let parameter = node.type_parameter(ast);
                let Some(annotation) = parameter.default(ast).or_else(|| parameter.constraint(ast))
                else {
                    return RawTypeTerminal::Closed;
                };
                annotation
            }
            TsTypeData::TsRestType(node) => {
                match ast.ts_tuple_element_data(node.type_annotation(ast)) {
                    swc_next_ecma_ast::TsTupleElementData::TsType(node) => node,
                    swc_next_ecma_ast::TsTupleElementData::TsNamedTupleMember(node) => {
                        node.element_type(ast)
                    }
                }
            }
            // These productions end in an actual atom, postfix token, or delimiter.
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
            | TsTypeData::TsTypeReference(_)
            | TsTypeData::TsTypeQuery(_)
            | TsTypeData::TsImportType(_)
            | TsTypeData::TsLiteralType(_)
            | TsTypeData::TsTemplateLiteralType(_)
            | TsTypeData::TsArrayType(_)
            | TsTypeData::TsIndexedAccessType(_)
            | TsTypeData::TsTupleType(_)
            | TsTypeData::TsJsDocNonNullableType(_)
            | TsTypeData::TsParenthesizedType(_)
            | TsTypeData::TsTypeLiteral(_)
            | TsTypeData::TsMappedType(_) => return RawTypeTerminal::Closed,
        };
    }
}
