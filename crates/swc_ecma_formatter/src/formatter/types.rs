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

use self::compound::{PendingTypeHead, PreparedTypeContent};
pub(super) use self::context::TypeContext;
use self::context::TypeEnclosure;
pub(super) use self::generics::TypeArgumentOwner;

use super::Formatter;
use super::bindings::{BindingGroup, BindingInitializerOwner, DefiniteMarker, InitializerLayout};
use super::comments::{
    CommentBinding, CommentBoundary, CommentRunLayout, ExpressionTail, PreparedTrailingRun,
};
use super::diagnostics::{FlatSeparator, LineConstraint};
use super::expressions::TailOwner;
use super::expressions::assignment::RhsLayout;
use super::literals::StringContext;
use super::statements::DeclarationKeyword;
use super::suppression::{ContentMode, SuppressionPosition, is_suppression_comment};
use super::variables::VariableContext;
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
    InferConstraint(&'a mut PreparedType),
    Default(&'a mut PreparedType),
}

#[derive(Clone, Copy)]
pub(super) struct PreparedType {
    pub(super) node: TsType,
    pub(super) data: TsTypeData,
    pub(super) source_owner: Option<TsType>,
}

struct PreparedTypeParentheses<'ast> {
    span: Span,
    child_span: Span,
    child: PreparedType,
    retained: bool,
    movable: bool,
    leading_comments: &'ast [Comment],
    trailing: CommentBoundary<'ast>,
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

struct RecordedTypeHead<'ast> {
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

enum TypeBoundaryLayout<'ast> {
    Source(Span),
    Formatted {
        before: CommentBoundary<'ast>,
        after: CommentBoundary<'ast>,
    },
}
pub(super) struct TypeBoundary<'ast> {
    role: TypeBoundaryRole,
    pub(super) left_mode: ContentMode,
    pub(super) right_mode: ContentMode,
    layout: TypeBoundaryLayout<'ast>,
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
impl PreparedDelimitedType<'_> {
    pub(super) fn starts_with_type_parameters(&self, ast: &Ast<'_>) -> bool {
        matches!(&self.content, PreparedTypeContent::Type(node)
            if matches!(node.data, TsTypeData::TsFunctionType(function)
                if function.type_parameters(ast).is_some()))
    }

    pub(super) fn is_object_type(&self) -> bool {
        matches!(&self.content, compound::PreparedTypeContent::Type(node)
            if node.source_owner.is_none()
                && matches!(node.data, TsTypeData::TsTypeLiteral(_) | TsTypeData::TsMappedType(_)))
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
                self.format_verbatim(gap, writer)?;
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

    pub(super) fn format_prepared_delimited_type(
        &mut self,
        body: PreparedDelimitedType<'ast>,
        context: TypeContext,
        layout: TypeLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let content = match body.content {
            compound::PreparedTypeContent::Compound(single) if single.movable() => {
                self.format_single_compound_head(single, context, writer)?
            }
            content => content,
        };
        self.format_prepared_type_content(content, context, layout, writer)
    }

    pub(super) fn prepare_delimited_type(
        &self,
        mut node: PreparedType,
    ) -> Result<PreparedDelimitedType<'ast>, FormatError> {
        let span = node.node.span(self.ast);
        let content = loop {
            if node.source_owner.is_some() {
                break compound::PreparedTypeContent::Type(node);
            }
            match node.data {
                TsTypeData::TsParenthesizedType(parentheses) => {
                    let parentheses =
                        self.prepare_type_parentheses(parentheses, None, TypeContext::Root);
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
            TypeBoundaryLayout::Source(gap) => {
                self.format_verbatim(*gap, writer)?;
                return Ok(None);
            }
            TypeBoundaryLayout::Formatted { before, after } => (before, after),
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
        let boundary = self.prepare_type_boundary(
            Span::new(head_end, annotation_span.start),
            TypeBoundaryTarget::Alias(&mut type_node),
        )?;
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
        let (before, after) = match &boundary.layout {
            TypeBoundaryLayout::Source(gap) => {
                self.format_verbatim(*gap, writer)?;
                return self.format_prepared_type_with_layout(
                    node,
                    TypeContext::Root,
                    TypeLayout::OpeningProvided,
                    writer,
                );
            }
            TypeBoundaryLayout::Formatted { before, after } => (before, after),
        };
        if node.source_owner == Some(node.node) {
            self.format_type_boundary(boundary, writer)?;
            return self.format_prepared_type(node, TypeContext::Root, writer);
        }
        if let TsTypeData::TsParenthesizedType(parentheses) = node.data {
            let parentheses =
                self.prepare_type_parentheses(parentheses, node.source_owner, TypeContext::Root);
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
            let single = match single.into_headless_type() {
                Ok(child) => {
                    return self.format_type_alias_rhs(boundary, child, rhs_layout, writer);
                }
                Err(single) => single,
            };
            if single.movable() {
                self.format_comment_boundary(before, writer)?;
                before
                    .gap()
                    .write(writer, FlatSeparator::Space, Writer::space);
                writer.text("=");
                let (head, content) = writer.record_with_value(|writer| {
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
        let expand = head.preferred_breaks();
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
            TypeBoundaryTarget::InferConstraint(_) => (
                "extends",
                TypeBoundaryRole::Constraint(LineConstraint::Unconstrained),
            ),
            TypeBoundaryTarget::Default(_) => ("=", TypeBoundaryRole::Default),
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
        if self.has_suppression && self.content_mode(comments) == ContentMode::Verbatim {
            for (run, position) in self.suppressed_comment_runs(before, operator.start) {
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
                        if let TypeBoundaryTarget::Alias(node)
                        | TypeBoundaryTarget::Constraint { node, .. }
                        | TypeBoundaryTarget::InferConstraint(node)
                        | TypeBoundaryTarget::Default(node) = &mut target
                        {
                            self.select_type_head(node, position);
                        }
                    }
                }
            }
            for (_, position) in self.suppressed_comment_runs(after, gap.end) {
                if !matches!(role, TypeBoundaryRole::Alias)
                    && matches!(position, SuppressionPosition::EndOfLine)
                {
                    left_mode = ContentMode::Verbatim;
                } else {
                    right_mode = ContentMode::Verbatim;
                    if let TypeBoundaryTarget::Alias(node)
                    | TypeBoundaryTarget::Constraint { node, .. }
                    | TypeBoundaryTarget::InferConstraint(node)
                    | TypeBoundaryTarget::Default(node) = &mut target
                    {
                        self.select_type_head(node, position);
                    }
                }
            }
        }
        let layout = if left_mode == ContentMode::Verbatim || right_mode == ContentMode::Verbatim {
            TypeBoundaryLayout::Source(gap)
        } else {
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
            TypeBoundaryLayout::Formatted {
                before: self.prepare_comment_boundary(before, gap.start, operator.start),
                after: self.prepare_comment_boundary(after, operator.end, gap.end),
            }
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
        }
    }

    fn select_delimited_type_head(
        &self,
        node: &mut PreparedType,
        comments: &[Comment],
        end: u32,
        first: bool,
    ) {
        for (run, position) in self.suppressed_comment_runs(comments, end) {
            let preceding = !first
                && !matches!(position, SuppressionPosition::OwnLine)
                && (matches!(position, SuppressionPosition::EndOfLine)
                    || run.last().is_some_and(|last| {
                        self.find_comma(Span::new(last.span.end, end), comments)
                            .is_some()
                    }));
            if !preceding {
                self.select_type_head(node, position);
            }
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

    fn select_type_head(&self, node: &mut PreparedType, position: SuppressionPosition) {
        if node.source_owner == Some(node.node) {
            return;
        }
        let mut data = node.data;
        let owner = if matches!(position, SuppressionPosition::OwnLine) {
            loop {
                match data {
                    TsTypeData::TsParenthesizedType(parentheses) => {
                        let child = parentheses.type_annotation(self.ast);
                        data = self.ast.ts_type_data(child);
                    }
                    TsTypeData::TsUnionType(union) => {
                        break union
                            .types(self.ast)
                            .first()
                            .map_or(node.node, |id| self.ast.get_node_in_sub_range(id));
                    }
                    _ => break node.node,
                }
            }
        } else {
            node.node
        };
        node.source_owner = Some(owner);
    }

    fn select_type_head_comments(
        &self,
        node: &mut PreparedType,
        comments: &[swc_next_ecma_ast::Comment],
        end: u32,
    ) {
        if !self.has_suppression || self.content_mode(comments) != ContentMode::Verbatim {
            return;
        }
        for (_, position) in self.suppressed_comment_runs(comments, end) {
            self.select_type_head(node, position);
        }
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
        if node.source_owner != Some(node.node)
            && let TsTypeData::TsParenthesizedType(parentheses) = node.data
        {
            let parentheses =
                self.prepare_type_parentheses(parentheses, node.source_owner, context);
            return self.format_prepared_type_parentheses(parentheses, context, layout, writer);
        }
        if node.source_owner != Some(node.node)
            && let TsTypeData::TsIntersectionType(intersection) = node.data
            && intersection.types(self.ast).len() > 1
        {
            let intersection = self.prepare_intersection(node, intersection.types(self.ast))?;
            return self.format_prepared_intersection(intersection, context, layout, writer);
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
                return self.format_compound_type(node, types, kind, context, layout, writer);
            }
        }
        self.format_type_in_context_with_enclosure(
            node.data,
            context,
            layout,
            writer,
            |formatter, layout, enclosure, writer| {
                formatter.format_type_data(node, layout, enclosure, writer)
            },
        )
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
            writer,
            |formatter, layout, _, writer| print(formatter, layout, writer),
        )
    }

    fn format_type_in_context_with_enclosure(
        &mut self,
        data: TsTypeData,
        context: TypeContext,
        layout: TypeLayout,
        writer: &mut Writer<'ast>,
        print: impl FnOnce(
            &mut Self,
            TypeLayout,
            TypeEnclosure,
            &mut Writer<'ast>,
        ) -> Result<Option<ExpressionTail<'ast>>, FormatError>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        if context.needs_parentheses(data, self.ast) {
            if matches!(
                data,
                TsTypeData::TsIntersectionType(_)
                    | TsTypeData::TsFunctionType(_)
                    | TsTypeData::TsConstructorType(_)
            ) || matches!(context, TypeContext::TypeConstraint)
                && matches!(data, TsTypeData::TsConditionalType(_))
            {
                writer.text("(");
                let tail = print(self, TypeLayout::Root, TypeEnclosure::Parentheses, writer)?;
                self.format_expression_tail(tail, writer)?;
                writer.text(")");
                return Ok(None);
            }
            return writer.group(false, |writer| {
                writer.text("(");
                writer.indent(|writer| {
                    writer.soft_line();
                    let tail = print(
                        self,
                        TypeLayout::CallerIndented,
                        TypeEnclosure::Parentheses,
                        writer,
                    )?;
                    self.format_expression_tail(tail, writer)
                })?;
                writer.soft_line();
                writer.text(")");
                Ok(None)
            });
        }
        print(self, layout, context.enclosure(), writer)
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
        context: TypeContext,
    ) -> PreparedTypeParentheses<'ast> {
        let ast = self.ast;
        let span = node.span(ast);
        let child = node.type_annotation(ast);
        let child_span = child.span(ast);
        let mut child_type = self.prepare_type(child, ContentMode::Formatted);
        child_type.source_owner = source_owner;
        let before = self
            .comments
            .in_range(Span::new(span.start + 1, child_span.start));
        let after = self
            .comments
            .in_range(Span::new(child_span.end, span.end - 1));
        let sensitive = self.diagnostic_range_is_sensitive(span);
        let retained = !before.is_empty() || !after.is_empty() || sensitive;
        let (source, movable) = if retained {
            if self.has_suppression && self.content_mode(before) == ContentMode::Verbatim {
                if matches!(context, TypeContext::UnionMember) {
                    child_type.source_owner = Some(child);
                } else {
                    self.select_type_head_comments(&mut child_type, before, child_span.start);
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
                        | TypeContext::IntersectionLineContinuation
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
            let source = source_owner == Some(TsType::TsParenthesizedType(node))
                || sensitive
                || !leading_binding.allows_relocation(leading_target)
                || !self
                    .comment_binding(after, span, trailing_target)
                    .allows_relocation(trailing_target);
            let movable =
                after.is_empty() && !source && matches!(leading_binding, CommentBinding::Unbound);
            (source, movable)
        } else {
            (
                source_owner == Some(TsType::TsParenthesizedType(node)),
                false,
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
        let fenced_tail = !trailing.is_empty()
            && !matches!(
                context,
                TypeContext::CompoundMember
                    | TypeContext::UnionMember
                    | TypeContext::IntersectionLineContinuation
            );
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
        if matches!(context, TypeContext::IntersectionLineContinuation) && trailing.is_empty() {
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

    fn format_recorded_type_head(
        &mut self,
        recorded: RecordedTypeHead<'ast>,
        context: TypeContext,
        layout: TypeLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let RecordedTypeHead { head, content, .. } = recorded;
        if let compound::PreparedTypeContent::Union(union) = content {
            return self.format_prepared_union(union, context, layout, Some(head), writer);
        }
        writer.group(false, |writer| {
            writer.write_fragment(head);
            self.format_prepared_type_content(content, context, layout, writer)
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
        let recorded = self.record_type_head(
            compound::PreparedTypeContent::Type(node),
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
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
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
            return self.format_annotated_binding(binding, group, writer);
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
            let boundary = self.assignment_boundary(
                left_end,
                &content,
                Span::new(operator, operator + 1),
                right.span(self.ast).end,
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
            let rhs_layout = self.ordinary_rhs_layout(content.data, context, &boundary);
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
            self.format_initializer_right(left_end, right, layout, context, writer)
        })
    }

    pub(super) fn format_prepared_delimited_type_after_comments(
        &mut self,
        body: PreparedDelimitedType<'ast>,
        after: &CommentBoundary<'ast>,
        context: TypeContext,
        layout: TypeLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let (movable_head, leading) = self.prepared_content_head(&body.content, context);
        if ((!after.is_empty() && self.prepared_content_accepts_caller_head(&body.content))
            || movable_head)
            && matches!(
                self.comment_boundary_binding(after, body.span),
                CommentBinding::Unbound
            )
        {
            let recorded = self.record_type_head(
                body.content,
                PendingTypeHead::new(after.comments(), body.span.start, leading),
                context,
                writer,
            )?;
            return self.format_recorded_type_head(recorded, context, layout, writer);
        }
        self.format_comment_boundary(after, writer)?;
        after
            .gap()
            .write(writer, FlatSeparator::Empty, Writer::soft_line);
        self.format_prepared_delimited_type(body, context, layout, writer)
    }

    pub(super) fn prepare_binary_type_head(
        &self,
        node: PreparedType,
        after: &CommentBoundary<'ast>,
        operator_end: u32,
        tail_owner: TailOwner,
    ) -> BinaryTypeHead<'ast> {
        if node.source_owner.is_some() {
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
            self.type_movable_head(node, TypeContext::Root)
        } else {
            None
        };
        let (comments, next_start) = if after.is_empty() {
            let Some(head) = leading else {
                return BinaryTypeHead(BinaryTypeHeadLayout::Direct);
            };
            (head.comments, head.next_start)
        } else {
            (after.comments(), target.start)
        };
        if let TsTypeData::TsTypeReference(reference) = node.data {
            let blocks = comments
                .iter()
                .all(|comment| comment.kind == CommentKind::Block);
            if blocks {
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
        let starts_line = comments.first().is_some_and(|first| {
            self.source_line_starts_line(first.span.start)
                || self.line_breaks(first.span.start, first.span.end) != 0
        });
        let ends_line = starts_line
            && comments.iter().enumerate().any(|(index, comment)| {
                let next = comments
                    .get(index + 1)
                    .map_or(next_start, |next| next.span.start);
                self.line_breaks(comment.span.end, next) != 0
            });
        let own_line = comments.first().filter(|_| ends_line);
        if own_line.is_none()
            && !compound
            && comments
                .iter()
                .all(|comment| comment.kind == CommentKind::Block)
        {
            return BinaryTypeHead(BinaryTypeHeadLayout::Direct);
        }
        BinaryTypeHead(BinaryTypeHeadLayout::Caller {
            leading,
            own_line: own_line.map(|first| self.line_constraint(operator_end, first.span.start)),
        })
    }

    pub(super) fn format_binary_type_rhs_head(
        &mut self,
        node: PreparedType,
        after: &CommentBoundary<'ast>,
        head: BinaryTypeHead<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let context = TypeContext::Root;
        if matches!(head.0, BinaryTypeHeadLayout::InlineNamed) {
            writer.space();
            self.format_comment_boundary_with_layout(
                after,
                CommentRunLayout::FlatLeading(node.node.span(self.ast).start),
                writer,
            )?;
            return self.format_prepared_type_with_layout(node, context, TypeLayout::Root, writer);
        }
        if matches!(head.0, BinaryTypeHeadLayout::Named) {
            let comments = after.comments();
            if let Some(first) = comments.first() {
                self.format_comment_boundary_with_layout(
                    after,
                    CommentRunLayout::SourceAfter(first.span.start),
                    writer,
                )?;
            }
            if let Some(last) = comments.last() {
                self.comment_gap(last.span.end, node.node.span(self.ast).start, writer);
            }
            return self.format_prepared_type_with_layout(node, context, TypeLayout::Root, writer);
        }
        if let BinaryTypeHeadLayout::TrailingNamed { name, tail } = head.0 {
            // This name has no nested token boundary to consume the leased packet.
            writer.space();
            writer.text(self.ast.get_utf8(name.name(self.ast)));
            return Ok(Some(tail));
        }
        let BinaryTypeHead(BinaryTypeHeadLayout::Caller { leading, own_line }) = head else {
            self.format_type_comment_gap(after, FlatSeparator::Space, writer)?;
            return self.format_prepared_type_with_layout(node, context, TypeLayout::Root, writer);
        };
        let mut pending =
            PendingTypeHead::new(after.comments(), node.node.span(self.ast).start, leading);
        if own_line.is_none() {
            pending = pending.with_inline_prefix();
        }
        let recorded = self.record_type_head(
            compound::PreparedTypeContent::Type(node),
            pending,
            context,
            writer,
        )?;
        let opening = match own_line {
            _ if recorded.riding => {
                writer.space();
                None
            }
            Some(opening)
                if !matches!(recorded.content, compound::PreparedTypeContent::Union(_)) =>
            {
                Some(opening)
            }
            _ => {
                writer.space();
                return self.format_recorded_type_head(recorded, context, TypeLayout::Root, writer);
            }
        };
        writer.indent(|writer| {
            if let Some(opening) = opening {
                opening.write(writer, FlatSeparator::Space, Writer::hard_line);
            }
            self.format_recorded_type_head(recorded, context, TypeLayout::CallerIndented, writer)
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
pub(super) fn type_requires_terminator(mut node: TsType, ast: &Ast<'_>) -> bool {
    let mut context = TypeContext::Root;
    loop {
        let data = ast.ts_type_data(node);
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
            TsTypeData::TsTypeOperator(node) => {
                (node.type_annotation(ast), TypeContext::ModifierOperand)
            }
            _ => return false,
        };
        node = next;
        context = next_context;
    }
}
