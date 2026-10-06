//! Expression arrows with parameter, arrow-token and body boundary ownership.
use super::parentheses::{ExpressionContent, SourceParentheses};
use super::{
    ExpressionContext, ExpressionLayout, ExpressionPosition, Formatter, SourceTailOwner, TailOwner,
};
use crate::formatter::comments::{CommentBinding, CommentRunLayout, ExpressionTail, NodeHead};
use crate::formatter::diagnostics::{FlatSeparator, LineConstraint};
use crate::formatter::functions::{FunctionBodyAttachment, SignatureBoundaryRole};
use crate::formatter::parameters::{ArrowParameterParentheses, SourceParameters};
use crate::formatter::statements::{IncomingStatement, StatementBodyLeading};
use crate::formatter::suppression::ContentMode;
use crate::{
    ArrowParentheses, FormatError, Semicolons,
    document::{Fragment, RecordedGroup, Writer},
};
use swc_next_ecma_ast::{
    ArrowFunctionBody, ArrowFunctionBodyData, ArrowFunctionExpression, AssignmentTargetData,
    Comment, CommentKind, Expr, ExprData, FormalParameterItemData, FormalParameterPatternData,
    FormalParameters, FunctionBody, GetSpan, Span,
};
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ArrowLayout {
    Ordinary,
    Assignment(ArrowHeadGap),
    /// The assignment already supplies the chain's opening gap and indent.
    AssignmentContinuation,
    Callee,
    Argument,
}

/// A leading comment run owns the gap to the arrow's first signature.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ArrowHeadGap {
    Expression,
    LeadingComments,
}

impl ArrowLayout {
    pub(super) fn with_leading_comments(self) -> Self {
        match self {
            Self::Assignment(_) => Self::Assignment(ArrowHeadGap::LeadingComments),
            _ => self,
        }
    }
}

pub(super) enum RecordedArrow<'a> {
    Verbatim(Fragment<'a>),
    Formatted(FormattedRecordedArrow<'a>),
}

pub(super) struct FormattedRecordedArrow<'a> {
    pub(super) signature: Fragment<'a>,
    pub(super) independent_signature: bool,
    pub(super) body: RecordedGroup<'a>,
    pub(super) body_layout: ArrowBodyLayout,
    pub(super) normal: Fragment<'a>,
    pub(super) tail: Option<ExpressionTail<'a>>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ArrowBodyLayout {
    Hug,
    Block,
    Expression,
}

struct ArrowLink<'a> {
    signature: Fragment<'a>,
    following: Option<Fragment<'a>>,
    trailing: Option<ArrowLinkTail<'a>>,
}

struct ArrowLinkTail<'a> {
    span: Span,
    comments: &'a [Comment],
    owner: ArrowLinkTailOwner,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ArrowLinkTailOwner {
    Body,
    ParentOperation,
}

struct RecordedArrowChain<'a> {
    first: ArrowLink<'a>,
    rest: Vec<ArrowLink<'a>>,
    body: RecordedGroup<'a>,
    tail: Option<ExpressionTail<'a>>,
    should_break: bool,
    body_layout: ArrowBodyLayout,
}

struct ArrowChainChild<'a> {
    head: FormattedArrowHead<'a>,
    leading: &'a [Comment],
    start: u32,
    parentheses: Option<SourceParentheses>,
}

pub(super) struct VerbatimArrow {
    span: Span,
    is_async: bool,
}
pub(super) struct FormattedArrowHead<'a> {
    arrow: ArrowFunctionExpression,
    parameters: SourceParameters,
    signature_end: u32,
    body: ArrowFunctionBody,
    body_span: Span,
    operator: u32,
    comments: &'a [Comment],
    before_count: usize,
}
pub(super) enum SourceArrow<'a> {
    Verbatim(VerbatimArrow),
    Formatted(FormattedArrowHead<'a>),
}
pub(super) struct FormattedSourceArrow<'a> {
    arrow: ArrowFunctionExpression,
    next: Option<ArrowChainChild<'a>>,
    parameters: SourceParameters,
    signature_end: u32,
    before: &'a [Comment],
    body: SourceArrowBody<'a>,
}
struct ArrowBoundary<'ast> {
    parameters: FormalParameters,
    signature_end: u32,
    body: ArrowFunctionBody,
    body_span: Span,
    operator: u32,
    comments: &'ast [Comment],
    before_count: usize,
}

pub(super) enum PreparedArrowBody {
    FunctionBody(FunctionBody),
    Expression {
        expression: Expr,
        content: ExpressionContent,
    },
}

pub(super) struct SourceArrowBody<'a> {
    body: PreparedArrowBody,
    body_span: Span,
    operator: u32,
    leading: &'a [Comment],
    body_leading: StatementBodyLeading<'a>,
    mode: ContentMode,
    first_statement: Option<IncomingStatement>,
    trailing_constraint: LineConstraint,
    leading_layout: CommentRunLayout,
    body_starts_line: bool,
    hug_body: bool,
    flat_conditional_pair: bool,
}

impl SourceArrowBody<'_> {
    fn layout(&self) -> ArrowBodyLayout {
        if self.hug_body {
            ArrowBodyLayout::Hug
        } else if matches!(self.body, PreparedArrowBody::FunctionBody(_)) {
            ArrowBodyLayout::Block
        } else {
            ArrowBodyLayout::Expression
        }
    }
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn arrow_needs_asi_guard(
        &self,
        arrow: ArrowFunctionExpression,
    ) -> Result<bool, FormatError> {
        if arrow.r#async(self.ast) {
            return Ok(false);
        }
        if self.options.arrow_parentheses == ArrowParentheses::Always {
            return Ok(true);
        }
        let boundary = self.source_arrow_boundary(arrow)?;
        let parentheses = if boundary.before_count == 0 {
            ArrowParameterParentheses::Optional
        } else {
            ArrowParameterParentheses::Required
        };
        Ok(self
            .source_arrow_parameters(boundary.parameters, parentheses)?
            .starts_with_parenthesis())
    }
    fn source_arrow_boundary(
        &self,
        arrow: ArrowFunctionExpression,
    ) -> Result<ArrowBoundary<'ast>, FormatError> {
        if arrow.return_type(self.ast).is_some() || arrow.type_parameters(self.ast).is_some() {
            return Err(self.unsupported("typed arrow", arrow));
        }
        let params = arrow.params(self.ast);
        let parameter_span = params.span(self.ast);
        let body = arrow.body(self.ast);
        let body_span = body.span(self.ast);
        let signature_end = parameter_span.end;
        let gap = Span::new(signature_end, body_span.start);
        let comments = self.comments.in_range(gap);
        let operator = self
            .find_token(gap, comments, "=>")
            .ok_or_else(|| self.unsupported("arrow token boundary", arrow))?;
        let before_count = comments.partition_point(|c| c.span.end <= operator);
        Ok(ArrowBoundary {
            parameters: params,
            signature_end,
            body,
            body_span,
            operator,
            comments,
            before_count,
        })
    }
    pub(super) fn source_arrow(
        &self,
        arrow: ArrowFunctionExpression,
    ) -> Result<SourceArrow<'ast>, FormatError> {
        let boundary = self.source_arrow_boundary(arrow)?;
        let parentheses = if boundary.before_count == 0 {
            ArrowParameterParentheses::Optional
        } else {
            ArrowParameterParentheses::Required
        };
        let parameters = self.source_arrow_parameters(boundary.parameters, parentheses)?;
        if parameters.enclosing_mode() == ContentMode::Verbatim {
            return Ok(SourceArrow::Verbatim(VerbatimArrow {
                span: arrow.span(self.ast),
                is_async: arrow.r#async(self.ast),
            }));
        }
        Ok(SourceArrow::Formatted(FormattedArrowHead {
            arrow,
            parameters,
            signature_end: boundary.signature_end,
            body: boundary.body,
            body_span: boundary.body_span,
            operator: boundary.operator,
            comments: boundary.comments,
            before_count: boundary.before_count,
        }))
    }
    pub(super) fn prepare_arrow_body(&self, body: ArrowFunctionBody) -> PreparedArrowBody {
        match self.ast.arrow_function_body_data(body) {
            ArrowFunctionBodyData::FunctionBody(body) => PreparedArrowBody::FunctionBody(body),
            ArrowFunctionBodyData::Expr(expression) => PreparedArrowBody::Expression {
                expression,
                content: self.expression_content(expression),
            },
        }
    }

    fn source_formatted_arrow(
        &self,
        head: FormattedArrowHead<'ast>,
        prepared_body: Option<PreparedArrowBody>,
    ) -> Result<FormattedSourceArrow<'ast>, FormatError> {
        let FormattedArrowHead {
            arrow,
            parameters,
            signature_end,
            body,
            body_span,
            operator,
            comments,
            before_count,
        } = head;
        let (before, leading) = comments.split_at(before_count);
        let body = prepared_body.unwrap_or_else(|| self.prepare_arrow_body(body));
        let (mode, first_statement, trailing_constraint) = match &body {
            PreparedArrowBody::FunctionBody(body) if self.has_suppression => {
                let modes = self.signature_body_modes(
                    *body,
                    leading,
                    body_span.start,
                    operator + 2,
                    SignatureBoundaryRole::Arrow {
                        has_parameters: parameters.has_parameters(),
                    },
                );
                (modes.body, modes.first_statement, modes.constraint)
            }
            PreparedArrowBody::FunctionBody(_) => {
                (ContentMode::Formatted, None, LineConstraint::Unconstrained)
            }
            PreparedArrowBody::Expression { .. } => (
                self.content_mode(leading),
                None,
                LineConstraint::Unconstrained,
            ),
        };
        let mode = match body {
            PreparedArrowBody::FunctionBody(body) => self.function_body_content_mode(body, mode),
            PreparedArrowBody::Expression { .. } => mode,
        };
        let (leading, body_leading) = if let PreparedArrowBody::FunctionBody(function_body) = body
            && parameters.has_parameters()
            && mode == ContentMode::Formatted
            && first_statement.is_none()
            && !leading.is_empty()
            && matches!(
                self.comment_binding(leading, Span::new(operator + 2, body_span.start), None),
                CommentBinding::Unbound
            ) {
            self.prepare_function_body_leading(
                leading,
                function_body,
                FunctionBodyAttachment::Arrow,
            )
        } else {
            (leading, StatementBodyLeading::empty())
        };
        let (head_end, hugs_shape, leading_layout) = match &body {
            PreparedArrowBody::Expression { content, .. } => (
                content.expression.span(self.ast).start,
                self.arrow_body_hugs_shape(content.data),
                CommentRunLayout::GroupedLeading(body_span.start),
            ),
            PreparedArrowBody::FunctionBody(_) => (
                body_span.start,
                true,
                CommentRunLayout::FlatLeading(body_span.start),
            ),
        };
        let head_comments = match body {
            PreparedArrowBody::FunctionBody(_) => leading,
            PreparedArrowBody::Expression { .. } => {
                self.comments.in_range(Span::new(operator + 2, head_end))
            }
        };
        let body_starts_line = head_comments.iter().enumerate().any(|(index, comment)| {
            let next_start = head_comments
                .get(index + 1)
                .map_or(head_end, |next| next.span.start);
            comment.kind == CommentKind::Line
                || (matches!(leading_layout, CommentRunLayout::GroupedLeading(_))
                    && self.source_position_starts_line(comment.span.start)
                    && self.line_breaks(comment.span.end, next_start) != 0)
        });
        let hug_body = hugs_shape && !body_starts_line;
        let flat_conditional_pair = !body_starts_line
            && mode == ContentMode::Formatted
            && matches!(&body, PreparedArrowBody::Expression { content, .. }
                if self.arrow_conditional_needs_flat_pair(content.data));
        let mut body = SourceArrowBody {
            body,
            body_span,
            operator,
            leading,
            body_leading,
            mode,
            first_statement,
            trailing_constraint,
            leading_layout,
            body_starts_line,
            hug_body,
            flat_conditional_pair,
        };
        let next = self.arrow_chain_child(&mut body)?;
        Ok(FormattedSourceArrow {
            arrow,
            next,
            parameters,
            signature_end,
            before,
            body,
        })
    }

    fn arrow_body_hugs_shape(&self, mut data: ExprData) -> bool {
        while let ExprData::ParenthesizedExpression(node) = data {
            data = self.ast.expr_data(node.expression(self.ast));
        }
        matches!(
            data,
            ExprData::ArrayExpression(_)
                | ExprData::ObjectExpression(_)
                | ExprData::SequenceExpression(_)
                | ExprData::ArrowFunctionExpression(_)
        )
    }
    pub(super) fn format_arrow(
        &mut self,
        arrow: ArrowFunctionExpression,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let mut source = match self.source_arrow(arrow)? {
            SourceArrow::Verbatim(source) => {
                self.format_verbatim_arrow(source, context, writer)?;
                return Ok(None);
            }
            SourceArrow::Formatted(head) => self.source_formatted_arrow(head, None)?,
        };
        if let Some(child) = source.next.take() {
            let chain = self.record_arrow_chain(source, child, context, writer)?;
            chain.write(context.arrow_layout, writer);
            return Ok(chain.tail);
        }
        writer.group(false, |writer| {
            let body = self.format_arrow_head(source, context, writer)?;
            self.format_arrow_body(body, writer)
        })
    }

    fn format_verbatim_arrow(
        &mut self,
        source: VerbatimArrow,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        if context.expression_start.needs_asi_guard()
            && self.options.semicolons == Semicolons::AsNeeded
            && !source.is_async
        {
            writer.text(";");
        }
        self.format_verbatim(source.span, writer)
    }

    fn arrow_chain_child(
        &self,
        body: &mut SourceArrowBody<'ast>,
    ) -> Result<Option<ArrowChainChild<'ast>>, FormatError> {
        let PreparedArrowBody::Expression {
            expression,
            content,
        } = &body.body
        else {
            return Ok(None);
        };
        let ExprData::ArrowFunctionExpression(arrow) = content.data else {
            return Ok(None);
        };
        let start = arrow.span(self.ast).start;
        let leading = self.comments.in_range(Span::new(body.operator + 2, start));
        if body.mode != ContentMode::Formatted
            || !self
                .comment_binding(leading, Span::new(body.operator + 2, start), None)
                .allows_relocation(None)
        {
            return Ok(None);
        }
        if self.expression_layout(*expression) == ExpressionLayout::Flat
            || self.expression_layout(content.expression) == ExpressionLayout::Flat
        {
            return Ok(None);
        }
        let head = match self.source_arrow(arrow)? {
            SourceArrow::Verbatim(_) => {
                body.mode = ContentMode::Verbatim;
                return Ok(None);
            }
            SourceArrow::Formatted(head) => head,
        };
        Ok(Some(ArrowChainChild {
            head,
            leading,
            start,
            parentheses: content.parentheses.map(|removed| removed.source),
        }))
    }

    fn arrow_chain_breaks(&self, source: &FormattedSourceArrow<'ast>) -> bool {
        let ast = self.ast;
        let params = source.arrow.params(ast);
        params.rest(ast).is_some()
            || params.items(ast).iter().any(|id| {
                let parameter = ast.get_node_in_sub_range(id);
                match ast.formal_parameter_item_data(parameter) {
                    FormalParameterItemData::FormalParameter(parameter) => {
                        parameter.initializer(ast).is_some()
                            || !matches!(
                                ast.formal_parameter_pattern_data(parameter.pattern(ast)),
                                FormalParameterPatternData::BindingIdentifier(_)
                            )
                    }
                    FormalParameterItemData::TsParameterProperty(_) => true,
                }
            })
    }

    pub(super) fn record_arrow(
        &mut self,
        source: SourceArrow<'ast>,
        prepared_body: Option<PreparedArrowBody>,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<RecordedArrow<'ast>, FormatError> {
        let mut source = match source {
            SourceArrow::Verbatim(source) => {
                let (value, ()) = writer.record_with_value(|writer| {
                    self.format_verbatim_arrow(source, context, writer)
                })?;
                return Ok(RecordedArrow::Verbatim(value));
            }
            SourceArrow::Formatted(head) => self.source_formatted_arrow(head, prepared_body)?,
        };
        let Some(first_child) = source.next.take() else {
            let independent_signature = !source.parameters.has_parameters();
            let (signature, source_body) = writer
                .record_with_value(|writer| self.format_arrow_head(source, context, writer))?;
            let body_layout = source_body.layout();
            let (body, tail) = writer.record_group_with_value(|writer| {
                self.format_arrow_body_content(source_body, writer)
            })?;
            let (normal, ()) = writer.record_with_value(|writer| {
                writer.group(false, |writer| {
                    writer.write_fragment(signature);
                    writer.write_fragment(if body_layout == ArrowBodyLayout::Hug {
                        body.contents()
                    } else {
                        body.grouped()
                    });
                });
                Ok::<_, FormatError>(())
            })?;
            return Ok(RecordedArrow::Formatted(FormattedRecordedArrow {
                signature,
                independent_signature,
                body,
                body_layout,
                normal,
                tail,
            }));
        };
        let chain = self.record_arrow_chain(source, first_child, context, writer)?;
        let (signature, ()) = writer.record_with_value(|writer| {
            for link in std::iter::once(&chain.first).chain(&chain.rest) {
                writer.write_fragment(link.signature);
                if let Some(following) = link.following {
                    writer.write_fragment(following);
                }
            }
            writer.text("=>");
            Ok::<_, FormatError>(())
        })?;
        let (normal, ()) = writer.record_with_value(|writer| {
            chain.write(context.arrow_layout, writer);
            Ok::<_, FormatError>(())
        })?;
        Ok(RecordedArrow::Formatted(FormattedRecordedArrow {
            signature,
            independent_signature: false,
            body: chain.body,
            body_layout: chain.body_layout,
            normal,
            tail: chain.tail,
        }))
    }

    fn record_arrow_chain(
        &mut self,
        mut source: FormattedSourceArrow<'ast>,
        first_child: ArrowChainChild<'ast>,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<RecordedArrowChain<'ast>, FormatError> {
        let mut should_break = self.arrow_chain_breaks(&source);
        let (signature, source_body) = writer
            .record_with_value(|writer| self.format_arrow_signature(source, context, writer))?;
        let (following, ()) = writer.record_with_value(|writer| {
            self.format_arrow_link(source_body, &first_child, writer)
        })?;
        let first = ArrowLink {
            signature,
            following: Some(following),
            trailing: first_child
                .parentheses
                .map(|boundary| self.prepare_arrow_link_tail(boundary)),
        };
        source = self.source_formatted_arrow(first_child.head, None)?;
        let mut links = Vec::new();
        let (body, tail, body_layout) = loop {
            should_break |= self.arrow_chain_breaks(&source);
            let next = source.next.take();
            let (signature, mut source_body) = writer.record_with_value(|writer| {
                self.format_arrow_signature(
                    source,
                    ExpressionContext::at(ExpressionPosition::ArrowBody),
                    writer,
                )
            })?;
            if let Some(child) = next {
                let (following, ()) = writer.record_with_value(|writer| {
                    self.format_arrow_link(source_body, &child, writer)
                })?;
                links.push(ArrowLink {
                    signature,
                    following: Some(following),
                    trailing: child
                        .parentheses
                        .map(|boundary| self.prepare_arrow_link_tail(boundary)),
                });
                source = self.source_formatted_arrow(child.head, None)?;
            } else {
                source_body.flat_conditional_pair &= !should_break;
                let body_layout = source_body.layout();
                links.push(ArrowLink {
                    signature,
                    following: None,
                    trailing: None,
                });
                let body_binding = match &source_body.body {
                    PreparedArrowBody::Expression {
                        expression,
                        content,
                    } => {
                        let binding =
                            self.expression_head_binding(source_body.leading, *expression);
                        content
                            .parentheses
                            .as_ref()
                            .map_or(binding, |removed| binding.combine(removed.head.binding))
                    }
                    PreparedArrowBody::FunctionBody(_) => self.comment_binding(
                        source_body.leading,
                        Span::new(source_body.operator + 2, source_body.body_span.start),
                        None,
                    ),
                };
                let inline_tails = context.arrow_layout != ArrowLayout::Callee
                    && source_body.mode == ContentMode::Formatted
                    && matches!(body_binding, CommentBinding::Unbound)
                    && !matches!(
                        &source_body.body,
                        PreparedArrowBody::Expression { content, .. }
                            if matches!(content.data, ExprData::ParenthesizedExpression(_))
                    )
                    && links.iter().chain(std::iter::once(&first)).all(|link| {
                        link.trailing
                            .as_ref()
                            .is_none_or(|tail| tail.owner == ArrowLinkTailOwner::Body)
                    });
                let (body, tail) = writer.record_group_with_value(|writer| {
                    let mut tail = self.format_arrow_body_content(source_body, writer)?;
                    for link in links.iter().rev().chain(std::iter::once(&first)) {
                        if let Some(trailing) = &link.trailing {
                            if inline_tails {
                                // Removed delimiters do not join the source gaps
                                // of independently owned inline tail packets.
                                self.format_expression_tail(tail.take(), writer)?;
                                self.comments.claim(trailing.comments)?;
                                self.format_comment_run(
                                    trailing.comments,
                                    CommentRunLayout::SourceAfter(trailing.span.start),
                                    writer,
                                )?;
                            } else {
                                tail = self.capture_expression_tail(trailing.span, tail)?;
                            }
                        }
                    }
                    Ok(tail)
                })?;
                break (body, tail, body_layout);
            }
        };
        Ok(RecordedArrowChain {
            first,
            rest: links,
            body,
            tail,
            should_break,
            body_layout,
        })
    }

    fn prepare_arrow_link_tail(&self, boundary: SourceParentheses) -> ArrowLinkTail<'ast> {
        let span = Span::new(boundary.inner.end, boundary.outer.end);
        let comments = self.comments.in_range(span);
        let inline = !self.diagnostic_range_is_sensitive(span)
            && comments.iter().all(|&comment| {
                comment.kind == CommentKind::Block
                    && !self.source_position_starts_line(comment.span.start)
                    && self.comment_semantics(comment).is_ordinary()
            });
        ArrowLinkTail {
            span,
            comments,
            owner: if inline {
                ArrowLinkTailOwner::Body
            } else {
                ArrowLinkTailOwner::ParentOperation
            },
        }
    }

    fn format_arrow_link(
        &mut self,
        source: SourceArrowBody<'ast>,
        child: &ArrowChainChild<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        writer.text("=>");
        let first = child
            .leading
            .first()
            .map_or(child.start, |comment| comment.span.start);
        self.line_constraint(source.operator + 2, first).write(
            writer,
            FlatSeparator::Space,
            |writer| {
                if source.body_starts_line {
                    writer.hard_line();
                } else {
                    writer.line();
                }
            },
        );
        self.comments.claim(child.leading)?;
        self.format_comment_run(
            child.leading,
            CommentRunLayout::GroupedLeading(child.start),
            writer,
        )
    }

    pub(super) fn format_arrow_head(
        &mut self,
        source: FormattedSourceArrow<'ast>,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<SourceArrowBody<'ast>, FormatError> {
        let body = self.format_arrow_signature(source, context, writer)?;
        writer.text("=>");
        Ok(body)
    }

    fn format_arrow_signature(
        &mut self,
        source: FormattedSourceArrow<'ast>,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<SourceArrowBody<'ast>, FormatError> {
        let FormattedSourceArrow {
            arrow,
            parameters,
            signature_end,
            before,
            body,
            ..
        } = source;
        if context.expression_start.needs_asi_guard()
            && self.options.semicolons == Semicolons::AsNeeded
            && !arrow.r#async(self.ast)
            && parameters.starts_with_parenthesis()
        {
            writer.text(";");
        }
        if arrow.r#async(self.ast) {
            writer.text("async");
            self.format_source_token_gap(
                arrow.span(self.ast).start + 5,
                parameters.span(self.ast).start,
                true,
                writer,
            )?;
        }
        writer.group(false, |writer| {
            self.format_source_parameters(parameters, writer)
        })?;
        self.comments.claim(before)?;
        self.format_comment_run(before, CommentRunLayout::SourceAfter(signature_end), writer)?;
        let before_end = before
            .last()
            .map_or(signature_end, |comment| comment.span.end);
        self.line_constraint(before_end, body.operator).write(
            writer,
            FlatSeparator::Space,
            Writer::space,
        );
        Ok(body)
    }

    fn arrow_conditional_needs_flat_pair(&self, data: ExprData) -> bool {
        let ExprData::ConditionalExpression(conditional) = data else {
            return false;
        };
        let ast = self.ast;
        let mut head = conditional.test(ast);
        loop {
            head = match ast.expr_data(head) {
                ExprData::ObjectExpression(_) => return false,
                ExprData::ParenthesizedExpression(node) => node.expression(ast),
                ExprData::BinaryExpression(node) => node.left(ast),
                ExprData::LogicalExpression(node) => node.left(ast),
                ExprData::AssignmentExpression(node) => {
                    let AssignmentTargetData::SimpleAssignmentTarget(target) =
                        ast.assignment_target_data(node.left(ast))
                    else {
                        return true;
                    };
                    super::targets::SimpleTarget::new(ast, target).expression()
                }
                ExprData::MemberExpression(node) => node.object(ast),
                ExprData::CallExpression(node) => node.callee(ast),
                ExprData::ConditionalExpression(node) => node.test(ast),
                ExprData::UpdateExpression(node) if !node.prefix(ast) => {
                    super::targets::SimpleTarget::new(ast, node.argument(ast)).expression()
                }
                ExprData::SequenceExpression(node) => {
                    let Some(first) = node.expressions(ast).iter().next() else {
                        return true;
                    };
                    ast.get_node_in_sub_range(first)
                }
                ExprData::ChainExpression(node) => node.expression(ast),
                ExprData::TsNonNullExpression(node) => node.expression(ast),
                ExprData::TsAsExpression(node) => node.expression(ast),
                ExprData::TsSatisfiesExpression(node) => node.expression(ast),
                _ => return true,
            };
        }
    }

    pub(super) fn format_arrow_body(
        &mut self,
        source: SourceArrowBody<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        if source.hug_body {
            self.format_arrow_body_content(source, writer)
        } else {
            writer.group(false, |writer| {
                self.format_arrow_body_content(source, writer)
            })
        }
    }

    fn format_arrow_body_content(
        &mut self,
        source: SourceArrowBody<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let SourceArrowBody {
            body,
            body_span,
            operator,
            leading,
            body_leading,
            mode,
            first_statement,
            trailing_constraint,
            leading_layout,
            body_starts_line,
            hug_body,
            flat_conditional_pair,
        } = source;
        let first = leading
            .first()
            .map_or(body_span.start, |comment| comment.span.start);
        let needs_gap = !matches!(body, PreparedArrowBody::FunctionBody(_))
            || mode != ContentMode::Verbatim && first_statement.is_none();
        let leading_constraint = needs_gap.then(|| self.line_constraint(operator + 2, first));
        let write_body = |writer: &mut Writer<'ast>| match body {
            PreparedArrowBody::Expression {
                expression,
                content,
            } => {
                let context = ExpressionContext::at(ExpressionPosition::ArrowBody)
                    .with_source_tail_owner(SourceTailOwner::Enclosure)
                    .with_content_mode(mode)
                    .with_tail_owner(TailOwner::ParentOperation);
                self.comments.claim(leading)?;
                self.format_prepared_expression_with_leading_comments(
                    expression,
                    content,
                    NodeHead {
                        comments: leading,
                        content_start: body_span.start,
                        layout: leading_layout,
                        gap_end: None,
                    },
                    context,
                    writer,
                )
            }
            PreparedArrowBody::FunctionBody(body) => {
                self.comments.claim(leading)?;
                if mode == ContentMode::Verbatim || first_statement.is_some() {
                    self.format_comment_run(
                        leading,
                        CommentRunLayout::SourceAfter(operator + 2),
                        writer,
                    )?;
                    trailing_constraint.write(writer, FlatSeparator::Space, Writer::space);
                } else {
                    self.format_comment_run(leading, leading_layout, writer)?;
                }
                self.format_function_body(body, mode, first_statement, body_leading, writer)?;
                Ok(None)
            }
        };
        let tail = if flat_conditional_pair {
            writer.space();
            // This pair follows the complete body's fit, independently of the
            // signature group and the conditional's own branch group.
            writer.group(false, |writer| {
                writer.if_flat("(");
                let tail = writer.indent(|writer| {
                    if let Some(constraint) = leading_constraint {
                        constraint.write(writer, FlatSeparator::Empty, Writer::soft_line);
                    }
                    write_body(writer)
                })?;
                writer.if_flat(")");
                Ok::<_, FormatError>(tail)
            })
        } else {
            let write = |writer: &mut Writer<'ast>| {
                if let Some(constraint) = leading_constraint {
                    constraint.write(writer, FlatSeparator::Space, |writer| {
                        if body_starts_line {
                            writer.hard_line();
                        } else if hug_body {
                            writer.space();
                        } else {
                            writer.line();
                        }
                    });
                }
                write_body(writer)
            };
            if hug_body {
                write(writer)
            } else {
                writer.indent(write)
            }
        }?;
        Ok(tail)
    }
}

impl<'a> RecordedArrowChain<'a> {
    fn write(&self, layout: ArrowLayout, writer: &mut Writer<'a>) {
        let Self {
            first,
            rest,
            body,
            should_break,
            body_layout,
            ..
        } = self;
        let hug_body = *body_layout == ArrowBodyLayout::Hug;
        writer.group(false, |writer| {
            let callee = layout == ArrowLayout::Callee;
            // A broken callee can keep its signatures flat. The final arrow and
            // body observe the enclosing group independently of the signatures.
            let group = writer.named_group(callee && !hug_body, |writer| {
                let write_remaining = |writer: &mut Writer<'a>| {
                    if let Some(following) = first.following {
                        writer.write_fragment(following);
                    }
                    for link in rest {
                        writer.write_fragment(link.signature);
                        if let Some(following) = link.following {
                            writer.write_fragment(following);
                        }
                    }
                };
                let write_signatures = |writer: &mut Writer<'a>| {
                    writer.group(*should_break, |writer| {
                        writer.write_fragment(first.signature);
                        if layout == ArrowLayout::Argument {
                            writer.indent(write_remaining);
                        } else {
                            write_remaining(writer);
                        }
                    });
                };
                match layout {
                    ArrowLayout::Argument | ArrowLayout::AssignmentContinuation => {
                        write_signatures(writer)
                    }
                    ArrowLayout::Callee | ArrowLayout::Assignment(ArrowHeadGap::Expression) => {
                        writer.indent(|writer| {
                            writer.soft_line();
                            write_signatures(writer);
                        })
                    }
                    ArrowLayout::Ordinary
                    | ArrowLayout::Assignment(ArrowHeadGap::LeadingComments) => {
                        writer.indent(write_signatures)
                    }
                }
            });
            writer.text("=>");
            if layout == ArrowLayout::AssignmentContinuation {
                writer.write_fragment(body.contents());
            } else {
                writer.indent_if_break(group, |writer| writer.write_fragment(body.contents()));
            }
            if callee {
                writer.soft_line_if_break(group);
            }
        });
    }
}
