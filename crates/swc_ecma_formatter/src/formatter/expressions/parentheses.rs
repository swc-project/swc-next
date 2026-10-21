//! Source delimiter ownership and semantic boundaries.

use swc_next_ecma_ast::{
    Comment, CommentKind, Expr, ExprData, GetSpan, ParenthesizedExpression, Span,
};

use super::{
    ExpressionContext, ExpressionPosition, ExpressionStart, Formatter, OperandIndent,
    ParenthesesOwner, SourceTailOwner, TailBoundary, TailOwner,
};
use crate::formatter::comments::{
    ArrayCommentOwner, CommentBinding, CommentRunLayout, ExpressionTail, NodeHead,
};
use crate::formatter::diagnostics::{FlatSeparator, LineConstraint};
use crate::formatter::source::is_source_whitespace;
use crate::formatter::suppression::{ContentMode, SuppressionPosition, TrailingSuppression};
use crate::{FormatError, Semicolons, document::Writer};

#[derive(Clone, Copy)]
pub(in crate::formatter) struct SourceParentheses {
    pub(in crate::formatter) outer: Span,
    pub(in crate::formatter) inner: Span,
    pub(in crate::formatter) inner_target: ParenthesisInner,
}

/// Source extents stay authoritative when the statement drops a transparent pair.
#[derive(Clone, Copy)]
struct StatementSourceOwner {
    source: Span,
    destination: Span,
}

#[derive(Clone, Copy)]
enum DiagnosticParenthesesOwner {
    Source,
    Statement(StatementSourceOwner),
}

#[derive(Clone, Copy)]
pub(in crate::formatter) enum ParenthesisInner {
    Expression(Expr),
    SourcePair,
    Unresolved,
}

pub(in crate::formatter) struct ExpressionContent {
    pub(in crate::formatter) expression: Expr,
    pub(in crate::formatter) data: ExprData,
    pub(super) parentheses: Option<RemovedExpressionParentheses>,
}

/// A removed source envelope keeps the binding resolved by its real owner.
#[derive(Clone, Copy)]
pub(super) struct RemovedExpressionParentheses {
    pub(super) source: SourceParentheses,
    pub(super) head: SourceParenthesesHead,
    pub(super) tail: Option<SourceParenthesesTail>,
}

/// Binding and immutable head presence are computed by the same source owner.
#[derive(Clone, Copy)]
pub(super) struct SourceParenthesesHead {
    pub(super) binding: CommentBinding,
    pub(super) has_comments: bool,
}

/// The removed source pair gives its emitted replacement this trailing boundary.
#[derive(Clone, Copy)]
pub(super) struct SourceParenthesesTail {
    span: Span,
}

struct RemovableSourceParentheses {
    head: SourceParenthesesHead,
    has_tail_comments: bool,
}

pub(in crate::formatter) struct ResolvedExpressionHead<'ast> {
    pub(in crate::formatter) head: NodeHead<'ast>,
    pub(in crate::formatter) binding: CommentBinding,
}

#[derive(Clone, Copy)]
pub(super) struct EmittedHeadAnnotation {
    comment: Comment,
}

#[derive(Clone, Copy)]
pub(super) struct GeneratedEnclosureOwner {
    expression: Expr,
    annotation: Comment,
}

pub(super) struct PreparedGeneratedEnclosure {
    pub(super) owner: GeneratedEnclosureOwner,
    pub(super) content: ExpressionContent,
}

/// Numeric selection retains the source decision and a separate pure reference view.
#[derive(Clone, Copy)]
pub(super) enum NumericContentView {
    Source,
    Reference,
    Both,
}

pub(super) struct NumericArgumentContent {
    pub(super) source: Option<ExprData>,
    pub(super) reference: Option<NumericReferenceRoot>,
}

pub(super) struct NumericReferenceRoot {
    pub(super) expression: Expr,
    pub(super) data: ExprData,
}

#[derive(Clone, Copy)]
enum ParenthesisCastPosition {
    Outer,
    Inner,
}

enum SourceCastState {
    Pending,
    Known(Option<Comment>),
}

/// Move this child boundary into the next wrapper iteration instead of
/// reconstructing its comment prefix or an already computed source cast.
struct NumericCastBoundary<'ast> {
    start: u32,
    before: &'ast [Comment],
    source: SourceCastState,
}

/// A source operand's opening packet is inspected without extending its operator gap.
#[derive(Clone, Copy)]
pub(super) struct SourceOpeningPacket<'ast> {
    outer: Span,
    pub(super) content: Expr,
    pub(super) content_start: u32,
    pub(super) comments: &'ast [Comment],
}

struct SourceParenthesesComments<'ast> {
    head: &'ast [Comment],
    tail: &'ast [Comment],
}

pub(in crate::formatter) struct IgnoredSourceParentheses {
    pub(in crate::formatter) expression: Expr,
    pub(in crate::formatter) parenthesized: ParenthesizedExpression,
    pub(in crate::formatter) leading: Span,
}

pub(in crate::formatter) struct SourceParenthesesContent {
    pub(super) data: ExprData,
    pub(in crate::formatter) mode: ContentMode,
    pub(in crate::formatter) ignored: Option<IgnoredSourceParentheses>,
}

impl SourceParenthesesContent {
    pub(super) fn leading(&self) -> Option<Span> {
        self.ignored.as_ref().map(|ignored| ignored.leading)
    }
}

enum RetainedParentheses {
    Cast,
    Annotation,
    Operand,
    FunctionReceiver,
}

struct RetainedSourceParentheses {
    boundary: SourceParentheses,
    kind: RetainedParentheses,
    hug: bool,
    leading_mode: Option<ContentMode>,
}

enum SourceParenthesesChoice {
    Verbatim,
    Retained {
        kind: RetainedParentheses,
        mode: ContentMode,
    },
    Content {
        mode: ContentMode,
        ignored_child: Option<Expr>,
    },
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn source_opening_packet(
        &self,
        expression: Expr,
    ) -> Option<SourceOpeningPacket<'ast>> {
        if !self.has_suppression {
            return None;
        }
        let outer = expression.span(self.ast);
        let mut content = expression;
        while let ExprData::ParenthesizedExpression(pair) = self.ast.expr_data(content) {
            content = pair.expression(self.ast);
        }
        if content == expression {
            return None;
        }
        let content_start = content.span(self.ast).start;
        let comments = self
            .comments
            .in_range(Span::new(outer.start, content_start));
        // Inline packets move with the child's existing prepared head. Only a
        // line-owning packet needs this source envelope to retain its binding.
        self.suppression_positions(comments, content_start)
            .any(|position| !matches!(position, SuppressionPosition::Inline))
            .then_some(SourceOpeningPacket {
                outer,
                content,
                content_start,
                comments,
            })
    }

    /// The parent has resolved this packet's owner; its original pairs retain the binding.
    pub(super) fn format_source_opening_packet(
        &mut self,
        source: SourceOpeningPacket<'ast>,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let content_span = source.content.span(self.ast);
        let closing = Span::new(content_span.end, source.outer.end);
        let mode = if context.content_mode == ContentMode::Verbatim
            || self.content_mode(self.comments.in_range(closing)) == ContentMode::Verbatim
        {
            ContentMode::Verbatim
        } else {
            ContentMode::Formatted
        };
        self.format_verbatim(Span::new(source.outer.start, content_span.start), writer)?;
        let mut pending = None;
        if mode == ContentMode::Verbatim {
            self.format_verbatim(content_span, writer)?;
        } else {
            let position =
                self.type_assertion_parentheses_position(source.content, None, context.position);
            let mut inner_context = context.inside_parentheses();
            inner_context.position = position;
            if let Some(tail) = self.format_expression(source.content, inner_context, writer)? {
                // Removed inner pairs can release comments before the child's source end.
                // Emit that leased prefix; the original closing slice owns only its suffix.
                let count = tail
                    .comments
                    .partition_point(|comment| comment.span.end <= content_span.end);
                let (released, remaining) = tail.comments.split_at(count);
                if !released.is_empty() {
                    self.format_expression_tail(
                        Some(ExpressionTail {
                            source_end: tail.source_end,
                            comments: released,
                        }),
                        writer,
                    )?;
                    if released
                        .last()
                        .is_some_and(|comment| comment.kind == CommentKind::Line)
                    {
                        writer.hard_line();
                    }
                }
                let count =
                    remaining.partition_point(|comment| comment.span.end <= source.outer.end);
                let after = &remaining[count..];
                if !after.is_empty() {
                    pending = Some(ExpressionTail {
                        source_end: source.outer.end,
                        comments: after,
                    });
                }
            }
        }
        self.format_verbatim(closing, writer)?;
        Ok(pending)
    }

    pub(in crate::formatter) fn expression_content(&self, expression: Expr) -> ExpressionContent {
        self.expression_content_for_position(expression, ExpressionPosition::Nested)
    }

    /// Inspect data after removing source parentheses transparent in the current
    /// comment scope, without preparing a source envelope for an ordinary node.
    pub(in crate::formatter) fn expression_data(&self, expression: Expr) -> ExprData {
        self.expression_data_for_position(expression, ExpressionPosition::Nested)
    }

    pub(super) fn expression_data_for_position(
        &self,
        expression: Expr,
        position: ExpressionPosition,
    ) -> ExprData {
        match self.ast.expr_data(expression) {
            ExprData::ParenthesizedExpression(_) => {
                self.expression_content_for_position(expression, position)
                    .data
            }
            data => data,
        }
    }

    pub(super) fn expression_content_for_position(
        &self,
        expression: Expr,
        position: ExpressionPosition,
    ) -> ExpressionContent {
        let data = self.ast.expr_data(expression);
        if !matches!(data, ExprData::ParenthesizedExpression(_)) {
            return ExpressionContent {
                expression,
                data,
                parentheses: None,
            };
        }
        let diagnostic_owner = if matches!(position, ExpressionPosition::Statement)
            && !self.diagnostic_lines.is_empty()
        {
            // Inspect the transparent source chain once. Every wrapper below still
            // resolves its own cast, comment, grammar and suppression boundaries.
            let mut destination = expression;
            while let ExprData::ParenthesizedExpression(pair) = self.ast.expr_data(destination) {
                destination = pair.expression(self.ast);
            }
            DiagnosticParenthesesOwner::Statement(StatementSourceOwner {
                source: expression.span(self.ast),
                destination: destination.span(self.ast),
            })
        } else {
            DiagnosticParenthesesOwner::Source
        };
        self.expression_content_with_diagnostic_owner(expression, diagnostic_owner)
    }

    fn expression_content_with_diagnostic_owner(
        &self,
        expression: Expr,
        diagnostic_owner: DiagnosticParenthesesOwner,
    ) -> ExpressionContent {
        let mut inner = expression;
        let mut head = SourceParenthesesHead {
            binding: CommentBinding::Unbound,
            has_comments: false,
        };
        let mut has_tail_comments = false;
        let mut optimizer_binding = None;
        let data = loop {
            let data = self.ast.expr_data(inner);
            let ExprData::ParenthesizedExpression(parenthesized) = data else {
                break data;
            };
            let child = parenthesized.expression(self.ast);
            if self
                .expression_needs_parentheses(
                    self.ast.expr_data(child),
                    ExpressionPosition::Nested,
                    ContentMode::Formatted,
                )
                .is_none()
            {
                break data;
            }
            let Some(next) = self.removable_parentheses_head_with_owner(
                SourceParentheses {
                    outer: parenthesized.span(self.ast),
                    inner: child.span(self.ast),
                    inner_target: ParenthesisInner::Expression(child),
                },
                &mut optimizer_binding,
                diagnostic_owner,
            ) else {
                break data;
            };
            let binding = head.binding.combine(next.head.binding);
            if matches!(binding, CommentBinding::SourceBound) {
                break data;
            }
            head = SourceParenthesesHead {
                binding,
                has_comments: head.has_comments || next.head.has_comments,
            };
            has_tail_comments |= next.has_tail_comments;
            inner = child;
        };
        let parentheses = (inner != expression).then(|| RemovedExpressionParentheses {
            source: SourceParentheses {
                outer: expression.span(self.ast),
                inner: inner.span(self.ast),
                inner_target: ParenthesisInner::Expression(inner),
            },
            head,
            tail: has_tail_comments.then_some(SourceParenthesesTail {
                span: Span::new(inner.span(self.ast).end, expression.span(self.ast).end),
            }),
        });
        ExpressionContent {
            expression: inner,
            data,
            parentheses,
        }
    }

    fn verbatim_parentheses_are_removable(
        &self,
        parenthesized: ParenthesizedExpression,
        position: ExpressionPosition,
    ) -> bool {
        if matches!(position, ExpressionPosition::Statement) {
            return false;
        }
        let child = parenthesized.expression(self.ast);
        let boundary = SourceParentheses {
            outer: parenthesized.span(self.ast),
            inner: child.span(self.ast),
            inner_target: ParenthesisInner::Expression(child),
        };
        let tail = self.source_parentheses_comments(boundary).tail;
        // An ignored envelope owns its closing line. Removing that line's delimiter
        // can change the attachment of the source packet that follows the envelope.
        if tail.iter().any(|comment| comment.kind == CommentKind::Line)
            || tail
                .last()
                .is_some_and(|last| self.line_breaks(boundary.inner.end, last.span.end) != 0)
        {
            return false;
        }
        self.parentheses_are_removable(boundary)
            && self.expression_needs_parentheses(
                self.ast.expr_data(child),
                position,
                ContentMode::Verbatim,
            ) == Some(false)
    }

    /// The selected Multiple-array owner drops a view once its decision is false.
    /// A requested source view keeps the existing expression_content stopping rule.
    pub(super) fn numeric_argument_content(
        &self,
        expression: Expr,
        view: NumericContentView,
    ) -> NumericArgumentContent {
        let mut inner = expression;
        let mut source = None;
        let mut source_pending =
            matches!(view, NumericContentView::Source | NumericContentView::Both);
        let mut reference = matches!(
            view,
            NumericContentView::Reference | NumericContentView::Both
        );
        let mut pending_cast = None;
        let mut optimizer_binding = None;
        loop {
            let data = self.ast.expr_data(inner);
            let ExprData::ParenthesizedExpression(parenthesized) = data else {
                if source_pending {
                    source = Some(data);
                }
                return NumericArgumentContent {
                    source,
                    reference: reference.then_some(NumericReferenceRoot {
                        expression: inner,
                        data,
                    }),
                };
            };
            let child = parenthesized.expression(self.ast);
            let boundary = SourceParentheses {
                outer: parenthesized.span(self.ast),
                inner: child.span(self.ast),
                inner_target: ParenthesisInner::Expression(child),
            };
            let mut outer_cast = pending_cast.take();
            let mut inner_cast = None;
            if source_pending
                && (self
                    .expression_needs_parentheses(
                        self.ast.expr_data(child),
                        ExpressionPosition::Nested,
                        ContentMode::Formatted,
                    )
                    .is_none()
                    || self
                        .removable_parentheses_head_with_casts(
                            boundary,
                            &mut optimizer_binding,
                            DiagnosticParenthesesOwner::Source,
                            |position| {
                                let (start, slot) = match position {
                                    ParenthesisCastPosition::Outer => {
                                        (boundary.outer.start, &mut outer_cast)
                                    }
                                    ParenthesisCastPosition::Inner => {
                                        (boundary.inner.start, &mut inner_cast)
                                    }
                                };
                                // Preserve source_type_cast's guard before its prefix query.
                                if !self.ast.source()[start as usize..].starts_with('(') {
                                    return None;
                                }
                                let cast =
                                    slot.get_or_insert_with(|| self.numeric_cast_boundary(start));
                                self.numeric_source_cast(cast)
                            },
                        )
                        .is_none())
            {
                source = Some(data);
                source_pending = false;
            }
            if reference && !self.ast.comments().is_empty() {
                let cast = outer_cast
                    .get_or_insert_with(|| self.numeric_cast_boundary(boundary.outer.start));
                if let Some(&comment) = cast.before.last()
                    && self.reference_parenthesis_cast(comment, cast.start)
                {
                    reference = false;
                }
            }
            if !source_pending && !reference {
                return NumericArgumentContent {
                    source,
                    reference: None,
                };
            }
            pending_cast = inner_cast;
            inner = child;
        }
    }

    fn numeric_cast_boundary(&self, start: u32) -> NumericCastBoundary<'ast> {
        let comments = self.ast.comments();
        let count = comments.partition_point(|comment| comment.span.end <= start);
        NumericCastBoundary {
            start,
            before: &comments[..count],
            source: SourceCastState::Pending,
        }
    }

    fn numeric_source_cast(&self, boundary: &mut NumericCastBoundary<'ast>) -> Option<Comment> {
        match boundary.source {
            SourceCastState::Known(cast) => cast,
            SourceCastState::Pending => {
                let cast = self.source_type_cast_before(boundary.start, boundary.before);
                boundary.source = SourceCastState::Known(cast);
                cast
            }
        }
    }

    /// Pinned reference postprocess keeps only an immediately preceding Closure cast.
    /// Keep its immediate-comment and trim-gap owner while sharing tag recognition.
    fn reference_parenthesis_cast(&self, comment: Comment, start: u32) -> bool {
        if comment.kind != CommentKind::Block {
            return false;
        }
        let raw = self.source(comment.span);
        if !raw.starts_with("/**")
            || !self
                .source(Span::new(comment.span.end, start))
                .chars()
                .all(|character| {
                    matches!(character,
                    '\u{0009}'..='\u{000d}' | ' ' | '\u{00a0}' | '\u{1680}'
                    | '\u{2000}'..='\u{200a}' | '\u{2028}' | '\u{2029}'
                    | '\u{202f}' | '\u{205f}' | '\u{3000}' | '\u{feff}')
                })
        {
            return false;
        }
        self.comment_is_type_cast(comment)
    }

    pub(in crate::formatter) fn parentheses_are_removable(
        &self,
        boundary: SourceParentheses,
    ) -> bool {
        self.removable_parentheses_head(boundary, &mut None)
            .is_some()
    }

    fn removable_parentheses_head(
        &self,
        boundary: SourceParentheses,
        optimizer_binding: &mut Option<CommentBinding>,
    ) -> Option<RemovableSourceParentheses> {
        self.removable_parentheses_head_with_owner(
            boundary,
            optimizer_binding,
            DiagnosticParenthesesOwner::Source,
        )
    }

    fn removable_parentheses_head_with_owner(
        &self,
        boundary: SourceParentheses,
        optimizer_binding: &mut Option<CommentBinding>,
        diagnostic_owner: DiagnosticParenthesesOwner,
    ) -> Option<RemovableSourceParentheses> {
        self.removable_parentheses_head_with_casts(
            boundary,
            optimizer_binding,
            diagnostic_owner,
            |position| {
                let start = match position {
                    ParenthesisCastPosition::Outer => boundary.outer.start,
                    ParenthesisCastPosition::Inner => boundary.inner.start,
                };
                self.source_type_cast(start)
            },
        )
    }

    fn removable_parentheses_head_with_casts(
        &self,
        boundary: SourceParentheses,
        optimizer_binding: &mut Option<CommentBinding>,
        diagnostic_owner: DiagnosticParenthesesOwner,
        mut cast: impl FnMut(ParenthesisCastPosition) -> Option<Comment>,
    ) -> Option<RemovableSourceParentheses> {
        let comments = self.ast.comments();
        if comments.is_empty() {
            return Some(RemovableSourceParentheses {
                head: SourceParenthesesHead {
                    binding: CommentBinding::Unbound,
                    has_comments: false,
                },
                has_tail_comments: false,
            });
        }
        let SourceParenthesesComments { head, tail } = self.source_parentheses_comments(boundary);
        if self.parentheses_separate_diagnostic_lines_with_owner(boundary, diagnostic_owner, tail)
            || self.parenthesis_has_jsdoc_enclosure(boundary.outer.start)
        {
            return None;
        }
        let leading_binding =
            self.source_parentheses_head_binding(head, boundary, optimizer_binding, || {
                cast(ParenthesisCastPosition::Inner)
            });
        let inner_target = match leading_binding {
            CommentBinding::BoundTo(target) => Some(target),
            _ => None,
        };
        if !leading_binding.allows_relocation(inner_target) {
            return None;
        }
        let trailing_binding = self.source_parentheses_tail_binding(tail, boundary);
        if !trailing_binding.allows_relocation(None)
            || cast(ParenthesisCastPosition::Outer).is_some()
        {
            return None;
        }
        Some(RemovableSourceParentheses {
            head: SourceParenthesesHead {
                binding: leading_binding,
                has_comments: !head.is_empty(),
            },
            has_tail_comments: !tail.is_empty(),
        })
    }

    fn source_parentheses_tail_binding(
        &self,
        comments: &[Comment],
        boundary: SourceParentheses,
    ) -> CommentBinding {
        let mut optimizer = false;
        let binding = self.comment_run_binding(comments.iter().copied(), |comment| {
            let semantics = self.comment_semantics(comment);
            if semantics.suppression
                || semantics.diagnostic
                || semantics.file_directive
                || semantics.type_cast
            {
                return CommentBinding::SourceBound;
            }
            optimizer |= semantics.optimizer;
            CommentBinding::Unbound
        });
        if !optimizer || matches!(binding, CommentBinding::SourceBound) {
            return binding;
        }
        let ParenthesisInner::Expression(_) = boundary.inner_target else {
            return CommentBinding::SourceBound;
        };
        let Some(closing) = boundary.outer.end.checked_sub(1) else {
            return CommentBinding::SourceBound;
        };
        if self.ast.source().as_bytes().get(closing as usize) != Some(&b')') {
            return CommentBinding::SourceBound;
        }
        // A trailing marker cannot target the preceding expression. Its source
        // frontier ends at the real closer, before any enclosing continuation.
        let mut start = boundary.inner.end;
        for comment in comments {
            if comment.span.start < start
                || comment.span.end > closing
                || !self
                    .source(Span::new(start, comment.span.start))
                    .chars()
                    .all(is_source_whitespace)
            {
                return CommentBinding::SourceBound;
            }
            start = comment.span.end;
        }
        if start > closing
            || !self
                .source(Span::new(start, closing))
                .chars()
                .all(is_source_whitespace)
        {
            return CommentBinding::SourceBound;
        }
        CommentBinding::Unbound
    }

    fn source_parentheses_comments(
        &self,
        boundary: SourceParentheses,
    ) -> SourceParenthesesComments<'ast> {
        let comments = self.ast.comments();
        let range = |span: Span| {
            let first = comments.partition_point(|comment| comment.span.start < span.start);
            let inside = &comments[first..];
            let count = inside.partition_point(|comment| comment.span.end <= span.end);
            &inside[..count]
        };
        SourceParenthesesComments {
            head: range(Span::new(boundary.outer.start, boundary.inner.start)),
            tail: range(Span::new(boundary.inner.end, boundary.outer.end)),
        }
    }

    fn parentheses_separate_diagnostic_lines(&self, boundary: SourceParentheses) -> bool {
        self.parentheses_separate_diagnostic_lines_with_owner(
            boundary,
            DiagnosticParenthesesOwner::Source,
            &[],
        )
    }

    fn parentheses_separate_diagnostic_lines_with_owner(
        &self,
        boundary: SourceParentheses,
        diagnostic_owner: DiagnosticParenthesesOwner,
        tail: &[Comment],
    ) -> bool {
        if !self.diagnostic_range_is_sensitive(boundary.outer) {
            return false;
        }
        // Delimiters can separate covered tokens from the next source line.
        let statement = match diagnostic_owner {
            DiagnosticParenthesesOwner::Statement(owner)
                if owner.source.start <= boundary.outer.start
                    && boundary.outer.end <= owner.source.end =>
            {
                Some(owner)
            }
            _ => None,
        };
        // An uncovered complete statement may replace its covered opener.
        // Only existing coverage needs protection; activating the directive
        // on the resulting statement does not require preserving parentheses.
        let statement_opening = statement.is_some_and(|owner| {
            !self.diagnostic_line_is_sensitive(owner.destination.start)
                && !self.diagnostic_line_is_sensitive(owner.destination.end)
        });
        let opening = !statement_opening
            && (self.diagnostic_line_is_sensitive(boundary.outer.start)
                || self.diagnostic_line_is_sensitive(boundary.inner.start))
            && self.line_breaks(boundary.outer.start, boundary.inner.start) != 0;
        if opening {
            return true;
        }
        let closing = self.diagnostic_line_is_sensitive(boundary.inner.end)
            && self.line_breaks(boundary.inner.end, boundary.outer.end) != 0;
        if !closing {
            return false;
        }
        // Only the complete statement owns this vacated closing line. An
        // operand's closer can still keep an outside diagnostic target covered.
        !(statement.is_some()
            && tail.is_empty()
            && self
                .source(Span::new(boundary.inner.end, boundary.outer.end - 1))
                .chars()
                .all(is_source_whitespace))
    }

    pub(in crate::formatter) fn capture_expression_tail(
        &self,
        span: Span,
        tail: Option<ExpressionTail<'ast>>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let source_end = tail.as_ref().map_or(span.start, |tail| tail.source_end);
        let comments = self.comments.between(Span::new(source_end, span.end))?;
        Ok((!comments.is_empty()).then_some(ExpressionTail {
            source_end,
            comments,
        }))
    }

    pub(in crate::formatter) fn parenthesis_has_type_cast(&self, start: u32) -> bool {
        self.source_type_cast(start).is_some()
    }

    /// Compiler documentation keeps every contiguous source owner, unlike a cast's
    /// immediately preceding JSDoc selection.
    fn parenthesis_has_jsdoc_enclosure(&self, start: u32) -> bool {
        if !self.ast.source()[start as usize..].starts_with('(') || !self.has_jsdoc_enclosures() {
            return false;
        }
        let comments = self.ast.comments();
        let count = comments.partition_point(|comment| comment.span.end <= start);
        let mut next_start = start;
        for &comment in comments[..count].iter().rev() {
            if !self
                .source(Span::new(comment.span.end, next_start))
                .chars()
                .rev()
                .all(is_source_whitespace)
            {
                break;
            }
            if self.comment_needs_jsdoc_enclosure(comment) {
                return true;
            }
            next_start = comment.span.start;
        }
        false
    }

    pub(in crate::formatter) fn expression_has_jsdoc_enclosure_head(
        &self,
        mut expression: Expr,
        mut context: ExpressionContext,
    ) -> Result<bool, FormatError> {
        if !self.has_jsdoc_enclosures() {
            return Ok(false);
        }
        loop {
            let data = self.ast.expr_data(expression);
            if let ExprData::ParenthesizedExpression(node) = data {
                if self.parenthesis_has_jsdoc_enclosure(node.span(self.ast).start) {
                    return Ok(true);
                }
                expression = node.expression(self.ast);
                continue;
            }
            let Some((child, child_context)) = self.expression_head_operand(data, context)? else {
                return Ok(false);
            };
            expression = child;
            context = child_context;
        }
    }

    pub(super) fn source_cast_head(&self, mut expression: Expr) -> Option<ParenthesizedExpression> {
        if !self.has_type_casts {
            return None;
        }
        loop {
            expression = match self.ast.expr_data(expression) {
                ExprData::ParenthesizedExpression(parenthesized) => {
                    if self.parenthesis_has_type_cast(parenthesized.span(self.ast).start) {
                        return Some(parenthesized);
                    }
                    parenthesized.expression(self.ast)
                }
                ExprData::MemberExpression(member) => member.object(self.ast),
                ExprData::CallExpression(call) => call.callee(self.ast),
                ExprData::TaggedTemplateExpression(template) => template.tag(self.ast),
                ExprData::ChainExpression(chain) => chain.expression(self.ast),
                _ => return None,
            };
        }
    }

    fn source_type_cast(&self, start: u32) -> Option<Comment> {
        if !self.has_type_casts || !self.ast.source()[start as usize..].starts_with('(') {
            return None;
        }
        self.source_cast_before(start)
    }

    pub(in crate::formatter) fn source_cast_before(&self, start: u32) -> Option<Comment> {
        if !self.has_type_casts {
            return None;
        }
        let comments = self.ast.comments();
        if comments.is_empty() {
            return None;
        }
        let count = comments.partition_point(|comment| comment.span.end <= start);
        self.source_type_cast_before(start, &comments[..count])
    }

    fn source_type_cast_before(&self, start: u32, comments: &[Comment]) -> Option<Comment> {
        let mut next_start = start;
        for &comment in comments.iter().rev() {
            if !self
                .source(Span::new(comment.span.end, next_start))
                .chars()
                .all(is_source_whitespace)
            {
                break;
            }
            // Ordinary trivia is transparent; a newer JSDoc replaces the older one.
            if self.source(comment.span).starts_with("/**") {
                return self.comment_is_type_cast(comment).then_some(comment);
            }
            next_start = comment.span.start;
        }
        None
    }

    fn source_parentheses_head_binding(
        &self,
        comments: &[Comment],
        boundary: SourceParentheses,
        optimizer_binding: &mut Option<CommentBinding>,
        mut cast: impl FnMut() -> Option<Comment>,
    ) -> CommentBinding {
        let function_target = match boundary.inner_target {
            ParenthesisInner::Expression(expression)
                if matches!(
                    self.ast.expr_data(expression),
                    ExprData::Function(_) | ExprData::ArrowFunctionExpression(_)
                ) =>
            {
                Some(expression.span(self.ast))
            }
            _ => None,
        };
        let function_binding = |comment: Comment| {
            function_target
                .filter(|_| {
                    let raw = self.source(comment.span);
                    raw.starts_with("/**") && !raw.starts_with("/***")
                })
                .map_or(CommentBinding::Unbound, CommentBinding::BoundTo)
        };
        let mut cast_binding = None;
        let resolve = |comment| {
            let semantics = self.comment_semantics(comment);
            if semantics.suppression || semantics.diagnostic || semantics.file_directive {
                return CommentBinding::SourceBound;
            }
            let optimizer = if semantics.optimizer {
                *optimizer_binding
                    .get_or_insert_with(|| self.source_optimizer_binding(comments, boundary))
            } else {
                CommentBinding::Unbound
            };
            if !semantics.type_cast {
                return optimizer.combine(function_binding(comment));
            }
            let binding = *cast_binding.get_or_insert_with(|| {
                if cast().is_none() {
                    return CommentBinding::Unbound;
                }
                let mut expression = match boundary.inner_target {
                    ParenthesisInner::Expression(expression) => expression,
                    ParenthesisInner::SourcePair => return CommentBinding::BoundTo(boundary.inner),
                    ParenthesisInner::Unresolved => return CommentBinding::SourceBound,
                };
                loop {
                    if expression.span(self.ast).start != boundary.inner.start {
                        return CommentBinding::SourceBound;
                    }
                    let data = self.ast.expr_data(expression);
                    if let ExprData::ParenthesizedExpression(node) = data {
                        return CommentBinding::BoundTo(node.span(self.ast));
                    }
                    if let ExprData::ChainExpression(node) = data {
                        expression = node.expression(self.ast);
                        continue;
                    }
                    if let ExprData::CallExpression(node) = data {
                        expression = node.callee(self.ast);
                        continue;
                    }
                    if let ExprData::TaggedTemplateExpression(node) = data {
                        expression = node.tag(self.ast);
                        continue;
                    }
                    if let ExprData::MemberExpression(node) = data {
                        expression = node.object(self.ast);
                        continue;
                    }
                    if let ExprData::ConditionalExpression(node) = data {
                        expression = node.test(self.ast);
                        continue;
                    }
                    if let ExprData::SequenceExpression(node) = data {
                        let Some(first) = node.expressions(self.ast).first() else {
                            return CommentBinding::SourceBound;
                        };
                        expression = self.ast.get_node_in_sub_range(first);
                        continue;
                    }
                    match super::binary::BinaryLike::from_data(data, self.ast) {
                        Some(binary) => expression = binary.left,
                        None => return CommentBinding::Unbound,
                    }
                }
            });
            optimizer.combine(match binding {
                CommentBinding::Unbound => function_binding(comment),
                _ => binding,
            })
        };
        if function_target.is_none() {
            return self.comment_run_binding(comments.iter().copied(), resolve);
        }
        CommentBinding::combine_all(comments.iter().copied().map(resolve))
    }

    /// Resolve only the packet's lexical head, without searching other operands.
    /// Applicable targets retain their source envelope and its continuation boundary.
    fn source_optimizer_binding(
        &self,
        packet: &[Comment],
        boundary: SourceParentheses,
    ) -> CommentBinding {
        let ParenthesisInner::Expression(mut expression) = boundary.inner_target else {
            return CommentBinding::SourceBound;
        };
        // Owners pass a contiguous source-comment slice (or one comment), so
        // its span identifies packet membership without another comment scan.
        let packet_span = packet
            .first()
            .zip(packet.last())
            .map(|(first, last)| Span::new(first.span.start, last.span.end));
        let mut head_start = boundary.inner.start;
        loop {
            if expression.span(self.ast).start != head_start {
                return CommentBinding::SourceBound;
            }
            expression = match self.ast.expr_data(expression) {
                ExprData::ParenthesizedExpression(node) => {
                    // The cast binding independently retains this source pair.
                    // Inspect its head only to prove an optimizer target absent;
                    // callable and foreign semantic frontiers stay source-bound.
                    let child = node.expression(self.ast);
                    let nested = SourceParentheses {
                        outer: node.span(self.ast),
                        inner: child.span(self.ast),
                        inner_target: ParenthesisInner::Expression(child),
                    };
                    let comments = self.source_parentheses_comments(nested);
                    // Another head packet owns a new resolution frontier. A
                    // trailing packet can instead prove its frontier is closed.
                    let outside_packet = |comment: &Comment| {
                        !packet_span.is_some_and(|packet| {
                            packet.start <= comment.span.start && comment.span.end <= packet.end
                        })
                    };
                    if comments.head.iter().any(|comment| {
                        outside_packet(comment) && !self.comment_semantics(*comment).is_ordinary()
                    }) || comments.tail.iter().any(outside_packet)
                        && !self
                            .source_parentheses_tail_binding(comments.tail, nested)
                            .allows_relocation(None)
                    {
                        return CommentBinding::SourceBound;
                    }
                    head_start = child.span(self.ast).start;
                    child
                }
                ExprData::CallExpression(_)
                | ExprData::NewExpression(_)
                | ExprData::Function(_)
                | ExprData::ArrowFunctionExpression(_) => {
                    return CommentBinding::SourceBound;
                }
                ExprData::ChainExpression(node) => node.expression(self.ast),
                ExprData::MemberExpression(node) => node.object(self.ast),
                ExprData::ConditionalExpression(node) => node.test(self.ast),
                ExprData::SequenceExpression(node) => {
                    let Some(first) = node.expressions(self.ast).first() else {
                        return CommentBinding::SourceBound;
                    };
                    self.ast.get_node_in_sub_range(first)
                }
                ExprData::IdentifierReference(_)
                | ExprData::StringLiteral(_)
                | ExprData::BooleanLiteral(_)
                | ExprData::NullLiteral(_)
                | ExprData::NumericLiteral(_)
                | ExprData::BigIntLiteral(_)
                | ExprData::RegExpLiteral(_)
                | ExprData::ThisExpression(_)
                | ExprData::MetaProperty(_)
                | ExprData::ArrayExpression(_)
                | ExprData::ObjectExpression(_)
                | ExprData::UnaryExpression(_)
                | ExprData::AwaitExpression(_)
                | ExprData::YieldExpression(_) => {
                    return CommentBinding::Unbound;
                }
                data => match super::binary::BinaryLike::from_data(data, self.ast) {
                    Some(binary) => binary.left,
                    None => return CommentBinding::SourceBound,
                },
            };
        }
    }

    pub(super) fn property_rhs_content(
        &self,
        content: &ExpressionContent,
        mode: ContentMode,
    ) -> ExprData {
        let context =
            ExpressionContext::at(ExpressionPosition::PropertyValue).with_content_mode(mode);
        self.source_parentheses_rhs_content(content, context)
    }

    pub(in crate::formatter) fn source_parentheses_head_content(
        &self,
        mut data: ExprData,
        mut context: ExpressionContext,
    ) -> SourceParenthesesContent {
        let original = data;
        let mut ignored = None;
        let first = match data {
            ExprData::ParenthesizedExpression(first) if self.has_suppression => first,
            _ => {
                return SourceParenthesesContent {
                    data,
                    mode: context.content_mode,
                    ignored,
                };
            }
        };
        let start = first.span(self.ast).start;
        while let ExprData::ParenthesizedExpression(parenthesized) = data {
            if context.content_mode == ContentMode::Verbatim
                && !self.verbatim_parentheses_are_removable(parenthesized, context.position)
            {
                break;
            }
            let (mode, child) = match self.source_parentheses_choice(parenthesized, context) {
                SourceParenthesesChoice::Content {
                    mode,
                    ignored_child: Some(child),
                } => (mode, child),
                SourceParenthesesChoice::Content {
                    mode: ContentMode::Formatted,
                    ignored_child: None,
                } if matches!(
                    context.position,
                    ExpressionPosition::ForInitializer {
                        grammar: super::ForInitializerGrammar::Expression,
                        parentheses: true,
                    }
                ) =>
                {
                    data = self.ast.expr_data(parenthesized.expression(self.ast));
                    continue;
                }
                _ => break,
            };
            context.content_mode = mode;
            ignored = Some(IgnoredSourceParentheses {
                expression: child,
                parenthesized,
                leading: Span::new(start, child.span(self.ast).start),
            });
            data = self.ast.expr_data(child);
        }
        if ignored.is_none() {
            data = original;
        }
        SourceParenthesesContent {
            data,
            mode: context.content_mode,
            ignored,
        }
    }

    pub(in crate::formatter) fn source_parentheses_rhs_content(
        &self,
        content: &ExpressionContent,
        context: ExpressionContext,
    ) -> ExprData {
        self.source_parentheses_head_content(content.data, context)
            .data
    }

    /// Only the printer that emits this pair can claim its removed source tail.
    pub(super) fn format_parentheses_tail(
        &mut self,
        owner: Option<SourceParenthesesTail>,
        tail: Option<ExpressionTail<'ast>>,
        array_comments: Option<&mut ArrayCommentOwner<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let Some(owner) = owner else {
            return Ok(tail);
        };
        let tail = self.capture_expression_tail(owner.span, tail)?;
        let line_comment = tail.as_ref().is_some_and(|tail| {
            tail.comments
                .last()
                .is_some_and(|comment| comment.kind == CommentKind::Line)
        });
        self.format_expression_tail_with_array_comments(tail, array_comments, writer)?;
        if line_comment {
            writer.hard_line();
        }
        Ok(None)
    }

    pub(super) fn ignored_parentheses_child(
        &self,
        parenthesized: ParenthesizedExpression,
        mode: ContentMode,
    ) -> Option<Expr> {
        if !self.has_suppression {
            return None;
        }
        let span = parenthesized.span(self.ast);
        let child = parenthesized.expression(self.ast);
        let child_span = child.span(self.ast);
        let mode = self.parentheses_content_mode(
            SourceParentheses {
                outer: span,
                inner: child_span,
                inner_target: ParenthesisInner::Expression(child),
            },
            mode,
        );
        if mode != ContentMode::Verbatim {
            return None;
        }
        let head = self
            .comments
            .in_range(Span::new(span.start, child_span.start));
        let trailing = Span::new(child_span.end, span.end - 1);
        let tail = self.comments.in_range(trailing);
        self.ignored_parentheses_child_with_comments(
            child,
            mode,
            SourceParenthesesComments { head, tail },
        )
    }

    fn ignored_parentheses_child_with_comments(
        &self,
        child: Expr,
        mode: ContentMode,
        comments: SourceParenthesesComments<'ast>,
    ) -> Option<Expr> {
        if !self.has_suppression || mode != ContentMode::Verbatim {
            return None;
        }
        comments
            .head
            .iter()
            .chain(comments.tail)
            .all(|&comment| {
                let semantics = self.comment_semantics(comment);
                semantics.is_ordinary() || semantics.suppression
            })
            .then_some(child)
    }

    fn parentheses_content_mode(
        &self,
        boundary: SourceParentheses,
        mode: ContentMode,
    ) -> ContentMode {
        if mode == ContentMode::Verbatim || !self.has_suppression {
            return mode;
        }
        let leading = self
            .comments
            .in_range(Span::new(boundary.outer.start, boundary.inner.start));
        if self.content_mode(leading) == ContentMode::Verbatim {
            return ContentMode::Verbatim;
        }
        let trailing = Span::new(boundary.inner.end, boundary.outer.end - 1);
        self.trailing_content_mode(trailing, true)
    }

    /// Resolve the real emitted head without discarding its source envelope.
    /// Retained, ignored and grammar-owned pairs remain the chain's head owner.
    pub(super) fn prepare_chain_parentheses_head(
        &self,
        parenthesized: ParenthesizedExpression,
        context: ExpressionContext,
    ) -> Option<ExpressionContent> {
        if context.content_mode != ContentMode::Formatted
            || self.diagnostic_range_is_sensitive(parenthesized.span(self.ast))
        {
            return None;
        }
        let SourceParenthesesChoice::Content {
            mode: ContentMode::Formatted,
            ignored_child: None,
        } = self.source_parentheses_choice(parenthesized, context)
        else {
            return None;
        };
        let content = self.expression_content(Expr::ParenthesizedExpression(parenthesized));
        let removed = content.parentheses?;
        // The source owner has selected removal. Its required generated pair
        // may still stop the chain walk; the leading run remains outside it.
        if matches!(removed.head.binding, CommentBinding::SourceBound) {
            return None;
        }
        Some(content)
    }

    /// Only a selected formatted source envelope can expose its operand head.
    pub(super) fn source_parentheses_head_is_removable(
        &self,
        parenthesized: ParenthesizedExpression,
        content: &ExpressionContent,
        context: ExpressionContext,
    ) -> bool {
        let Some(removed) = content.parentheses else {
            return false;
        };
        if context.content_mode != ContentMode::Formatted
            || matches!(removed.head.binding, CommentBinding::SourceBound)
            || self.diagnostic_range_is_sensitive(parenthesized.span(self.ast))
        {
            return false;
        }
        matches!(
            self.source_parentheses_choice(parenthesized, context),
            SourceParenthesesChoice::Content {
                mode: ContentMode::Formatted,
                ignored_child: None,
            }
        )
    }

    pub(super) fn jsx_source_pair_is_enclosed(
        &self,
        expression: Expr,
        content: &ExpressionContent,
        context: ExpressionContext,
    ) -> bool {
        matches!(
            content.data,
            ExprData::JsxElement(_) | ExprData::JsxFragment(_)
        ) && context.content_mode == ContentMode::Formatted
            && context.parentheses_owner == ParenthesesOwner::Expression
            && Self::jsx_grammar_pair_is_enclosed(context.position)
            && expression.as_parenthesized_expression(self.ast).is_some()
            && self.expression_is_parenthesized(content.data, context)
    }

    fn source_parentheses_choice(
        &self,
        parenthesized: ParenthesizedExpression,
        mut context: ExpressionContext,
    ) -> SourceParenthesesChoice {
        let span = parenthesized.span(self.ast);
        let child = parenthesized.expression(self.ast);
        let child_span = child.span(self.ast);
        let boundary = SourceParentheses {
            outer: span,
            inner: child_span,
            inner_target: ParenthesisInner::Expression(child),
        };
        let comments = if self.has_suppression {
            self.source_parentheses_comments(boundary)
        } else {
            SourceParenthesesComments {
                head: &[],
                tail: &[],
            }
        };
        let trailing = Span::new(child_span.end, span.end - 1);
        if matches!(context.position, ExpressionPosition::NonNullOperand) {
            let comments = self.comments.in_range(trailing);
            let breaks = comments
                .iter()
                .any(|comment| comment.kind == CommentKind::Line)
                || comments
                    .last()
                    .is_some_and(|comment| self.line_breaks(trailing.start, comment.span.end) != 0);
            if breaks {
                return SourceParenthesesChoice::Retained {
                    kind: RetainedParentheses::Operand,
                    mode: context.content_mode,
                };
            }
        }
        let tail_end = comments
            .tail
            .partition_point(|comment| comment.span.end <= trailing.end);
        let trailing_suppression =
            self.trailing_suppression_from_comments(trailing, &comments.tail[..tail_end], true);
        let trailing_binding = match trailing_suppression {
            None => CommentBinding::Unbound,
            Some(_) => CommentBinding::BoundTo(child_span),
        };
        // The source pair keeps an inline trailing marker attached to its child.
        // Removing or rebuilding its delimiters can select a containing node instead.
        if matches!(trailing_suppression, Some(TrailingSuppression::Inline)) {
            return SourceParenthesesChoice::Verbatim;
        }
        if context.content_mode == ContentMode::Formatted && self.has_suppression {
            context.content_mode = if trailing_suppression.is_some() {
                ContentMode::Verbatim
            } else {
                self.content_mode(comments.head)
            };
        }
        if self.parenthesis_has_type_cast(span.start) {
            // A cast pair is part of its ignored source extent, including its trivia.
            if context.content_mode == ContentMode::Verbatim {
                return SourceParenthesesChoice::Verbatim;
            }
            return SourceParenthesesChoice::Retained {
                kind: RetainedParentheses::Cast,
                mode: context.content_mode,
            };
        }
        if self.parenthesis_has_jsdoc_enclosure(span.start) {
            if context.content_mode == ContentMode::Verbatim {
                return SourceParenthesesChoice::Verbatim;
            }
            return SourceParenthesesChoice::Retained {
                kind: RetainedParentheses::Annotation,
                mode: context.content_mode,
            };
        }
        if matches!(trailing_suppression, Some(TrailingSuppression::Terminal)) {
            // The terminal marker selects the child through this actual closer.
            // Raw child content alone cannot preserve that target after removal.
            return SourceParenthesesChoice::Retained {
                kind: RetainedParentheses::Operand,
                mode: context.content_mode,
            };
        }
        if let ExprData::ParenthesizedExpression(inner) = self.ast.expr_data(child)
            && self.parenthesis_has_type_cast(child_span.start)
            && self.parentheses_content_mode(
                SourceParentheses {
                    outer: child_span,
                    inner: inner.expression(self.ast).span(self.ast),
                    inner_target: ParenthesisInner::Expression(inner.expression(self.ast)),
                },
                context.content_mode,
            ) == ContentMode::Verbatim
            && self
                .source_type_assertion_needs_grammar_barrier(self.ast.expr_data(child), context)
                .unwrap_or(true)
        {
            // Keep the actual grammar enclosure around an opaque cast child.
            // A removed parent cannot be replaced inside the ignored source pair;
            // an unrecognized grammar boundary also keeps its source enclosure.
            return SourceParenthesesChoice::Retained {
                kind: RetainedParentheses::Operand,
                mode: context.content_mode,
            };
        }
        if context.parentheses_owner == ParenthesesOwner::Expression
            && matches!(
                context.position,
                ExpressionPosition::CallCallee { .. } | ExpressionPosition::TemplateTag
            )
        {
            let receiver = match context.content_mode {
                ContentMode::Formatted => self.expression_content(child).expression,
                ContentMode::Verbatim => child,
            };
            if matches!(
                self.ast.expr_data(receiver),
                ExprData::Function(_) | ExprData::ArrowFunctionExpression(_)
            ) {
                let comments = self.source_parentheses_comments(SourceParentheses {
                    outer: span,
                    inner: receiver.span(self.ast),
                    inner_target: ParenthesisInner::Expression(receiver),
                });
                if !comments.head.is_empty() || !comments.tail.is_empty() {
                    return SourceParenthesesChoice::Retained {
                        kind: RetainedParentheses::FunctionReceiver,
                        mode: context.content_mode,
                    };
                }
            }
        }
        if self.ast.is_jsx()
            && context.content_mode == ContentMode::Formatted
            && context.parentheses_owner == ParenthesesOwner::Expression
            && Self::jsx_grammar_pair_is_enclosed(context.position)
        {
            let content = self.expression_content(child);
            if self.jsx_source_pair_is_enclosed(
                Expr::ParenthesizedExpression(parenthesized),
                &content,
                context,
            ) {
                // The real source pair keeps the JSX head and tail in its frame.
                return SourceParenthesesChoice::Retained {
                    kind: RetainedParentheses::Operand,
                    mode: context.content_mode,
                };
            }
        }
        if Self::conditional_test_pair_is_enclosed(context.position)
            && matches!(
                self.ast.expr_data(child),
                ExprData::ConditionalExpression(_)
            )
        {
            return SourceParenthesesChoice::Retained {
                kind: RetainedParentheses::Operand,
                mode: context.content_mode,
            };
        }
        let ignored_child =
            self.ignored_parentheses_child_with_comments(child, context.content_mode, comments);
        if let Some(ignored) = ignored_child
            && (!trailing_binding.allows_relocation(None)
                && context.tail_owner == TailOwner::ParentOperation
                || self.ignored_operand_owns_operation_head(ignored, context.position)
                || matches!(
                    context.position,
                    ExpressionPosition::Statement
                        | ExpressionPosition::UnaryArgument(_)
                        | ExpressionPosition::MemberObject(_)
                        | ExpressionPosition::MemberAssignmentObject
                        | ExpressionPosition::CallCallee { .. }
                        | ExpressionPosition::NewCallee
                        | ExpressionPosition::ArrowBody
                        | ExpressionPosition::RestrictedArgument
                        | ExpressionPosition::YieldArgument
                        | ExpressionPosition::AwaitArgument
                )
                || context.expression_start == ExpressionStart::ArrowBody
                || self.for_initializer_span.is_some_and(|scope| {
                    self.opaque_initializer_needs_enclosure(self.ast.expr_data(ignored), scope)
                })
                || self.parentheses_separate_diagnostic_lines(boundary)
                || self
                    .expression_needs_parentheses(
                        self.ast.expr_data(child),
                        context.position,
                        ContentMode::Verbatim,
                    )
                    .is_none())
        {
            return SourceParenthesesChoice::Retained {
                kind: RetainedParentheses::Operand,
                mode: ContentMode::Verbatim,
            };
        }
        SourceParenthesesChoice::Content {
            mode: context.content_mode,
            ignored_child,
        }
    }

    pub(in crate::formatter) fn format_expression_with_leading_comments(
        &mut self,
        expression: Expr,
        head: NodeHead<'ast>,
        context: ExpressionContext,
        array_comments: Option<&mut ArrayCommentOwner<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let span = expression.span(self.ast);
        let binding = self.source_parentheses_head_binding(
            head.comments,
            SourceParentheses {
                outer: span,
                inner: span,
                inner_target: ParenthesisInner::Expression(expression),
            },
            &mut None,
            || self.source_type_cast(span.start),
        );
        self.format_expression_leading_prepared(
            expression,
            None,
            ResolvedExpressionHead { head, binding },
            context,
            array_comments,
            writer,
        )
    }

    pub(super) fn format_expression_leading_prepared(
        &mut self,
        expression: Expr,
        prepared: Option<ExpressionContent>,
        resolved: ResolvedExpressionHead<'ast>,
        context: ExpressionContext,
        array_comments: Option<&mut ArrayCommentOwner<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_expression_head_and_content(
            expression,
            prepared,
            resolved,
            context,
            array_comments,
            writer,
        )
    }

    fn format_expression_head_and_content(
        &mut self,
        expression: Expr,
        prepared: Option<ExpressionContent>,
        resolved: ResolvedExpressionHead<'ast>,
        context: ExpressionContext,
        array_comments: Option<&mut ArrayCommentOwner<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let prepared = prepared.or_else(|| {
            (matches!(context.position, ExpressionPosition::Statement)
                && context.content_mode == ContentMode::Formatted
                && expression.is_parenthesized_expression(self.ast))
            .then(|| self.expression_content_for_position(expression, context.position))
        });
        // An empty head has no packet to place. Source-bound and inherited
        // protected heads still need the enclosing owner's placement proof.
        if !resolved.head.comments.is_empty()
            || matches!(resolved.binding, CommentBinding::SourceBound)
            || context
                .protected_head_start
                .is_some_and(|start| start == expression.span(self.ast).start)
        {
            return self.format_expression_head_enclosure(
                expression,
                prepared,
                resolved,
                context,
                array_comments,
                writer,
            );
        }
        self.expression_layout(expression).write(writer, |writer| {
            self.format_expression_content_prepared(
                expression,
                prepared,
                context,
                array_comments,
                writer,
            )
        })
    }

    fn format_expression_head_enclosure(
        &mut self,
        expression: Expr,
        mut prepared: Option<ExpressionContent>,
        resolved: ResolvedExpressionHead<'ast>,
        mut context: ExpressionContext,
        mut array_comments: Option<&mut ArrayCommentOwner<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let ResolvedExpressionHead { head, binding } = resolved;
        let binding = context.resolve_emitted_head_binding(&head, binding);
        if !head.comments.is_empty() {
            context.arrow_layout = context.arrow_layout.with_leading_comments();
        }
        if self.has_type_casts
            && !head.comments.is_empty()
            && matches!(binding, CommentBinding::Unbound)
            && context.content_mode == ContentMode::Formatted
        {
            let content = prepared.get_or_insert_with(|| self.expression_content(expression));
            if let ExprData::SequenceExpression(sequence) = content.data
                && self.sequence_head_has_type_cast(sequence, head.comments)
                && self.expression_is_parenthesized(content.data, context)
            {
                return self.format_parenthesized_expression(
                    expression,
                    prepared,
                    head,
                    context,
                    array_comments,
                    writer,
                );
            }
        }
        if !head.comments.is_empty()
            && matches!(binding, CommentBinding::Unbound)
            && context.content_mode == ContentMode::Formatted
            && Self::jsx_expression_has_optional_frame(context)
            && matches!(
                prepared
                    .get_or_insert_with(|| self.expression_content(expression))
                    .data,
                ExprData::JsxElement(_) | ExprData::JsxFragment(_)
            )
        {
            let span = Span::new(head.comments[0].span.start, expression.span(self.ast).end);
            let head = NodeHead {
                layout: CommentRunLayout::GroupedLeading(
                    head.gap_end.unwrap_or(head.content_start),
                ),
                gap_end: None,
                ..head
            };
            let source_tail = prepared
                .as_ref()
                .and_then(|content| content.parentheses)
                .and_then(|removed| removed.tail)
                .filter(|_| {
                    context.source_tail_owner == SourceTailOwner::Enclosure
                        && context.tail_owner != TailOwner::ParentOperation
                });
            return self.format_jsx_expression(span, context, writer, |formatter, writer| {
                let tail = formatter.format_expression_leading_prepared(
                    expression,
                    prepared,
                    ResolvedExpressionHead { head, binding },
                    context.inside_parentheses(),
                    array_comments.as_deref_mut(),
                    writer,
                )?;
                let tail = formatter.format_parentheses_tail(
                    context.source_parentheses_tail,
                    tail,
                    array_comments.as_deref_mut(),
                    writer,
                )?;
                let tail = if let Some(owner) = source_tail
                    && tail.is_some()
                {
                    if context.tail_boundary == TailBoundary::Separated {
                        writer.break_parent();
                    }
                    let tail = formatter.capture_expression_tail(owner.span, tail)?;
                    formatter.format_expression_tail_with_array_comments(
                        tail,
                        array_comments.as_deref_mut(),
                        writer,
                    )?;
                    None
                } else {
                    tail
                };
                if context.tail_owner == TailOwner::ParentOperation
                    || context.source_tail_owner == SourceTailOwner::Enclosure
                {
                    Ok(tail)
                } else {
                    formatter.format_expression_tail_with_array_comments(
                        tail,
                        array_comments,
                        writer,
                    )?;
                    Ok(None)
                }
            });
        }
        // Added readability pairs have their own placement proof; the source
        // binding still owns ASI and diagnostic line constraints.
        let pair_binding = self.optional_nullish_head_binding(
            head.comments,
            || {
                let span = expression.span(self.ast);
                SourceParentheses {
                    outer: span,
                    inner: span,
                    inner_target: ParenthesisInner::Expression(expression),
                }
            },
            binding,
        );
        let protected_head = context
            .protected_head_start
            .is_some_and(|start| start == expression.span(self.ast).start);
        let nullish_operand = context.parentheses_owner == ParenthesesOwner::Expression
            && (!head.comments.is_empty()
                || matches!(pair_binding, CommentBinding::SourceBound)
                || protected_head)
            && self.is_nullish_conditional_operand(
                prepared
                    .get_or_insert_with(|| self.expression_content(expression))
                    .data,
                context.position,
            );
        if nullish_operand
            && (matches!(pair_binding, CommentBinding::SourceBound) || protected_head)
        {
            // The enclosing owner withholds this optional pair to preserve its
            // complete source packet's adjacency to the actual lexical head.
            context.parentheses_owner = ParenthesesOwner::Parent;
        }
        let enclosure_binding = if nullish_operand {
            pair_binding
        } else {
            binding
        };
        let inner_cast =
            if context.content_mode == ContentMode::Formatted
                && matches!(enclosure_binding, CommentBinding::SourceBound)
            {
                self.source_type_cast(head.content_start).and_then(|annotation| {
                if !matches!(
                    self.expression_head_binding(std::slice::from_ref(&annotation), expression),
                    CommentBinding::BoundTo(target) if target != expression.span(self.ast)
                ) {
                    return None;
                }
                head.comments.iter().position(|comment| *comment == annotation)
            })
            } else {
                None
            };
        let generated = if self.has_type_casts
            && !head.comments.is_empty()
            && matches!(binding, CommentBinding::Unbound)
            && context.content_mode == ContentMode::Formatted
            && context.parentheses_owner == ParenthesesOwner::Expression
        {
            let content = prepared.get_or_insert_with(|| self.expression_content(expression));
            self.generated_enclosure_owner(expression, content, &head, binding, context)
        } else {
            None
        };
        let parenthesize_head = !head.comments.is_empty()
            && !context.is_verbatim_statement_string(self.ast.expr_data(expression))
            && (matches!(enclosure_binding, CommentBinding::BoundTo(_))
                || inner_cast.is_some()
                || generated.is_some())
            && context.parentheses_owner == ParenthesesOwner::Expression
            && self.expression_needs_parentheses(
                prepared
                    .get_or_insert_with(|| self.expression_content(expression))
                    .data,
                context.position,
                ContentMode::Formatted,
            ) == Some(true);
        if parenthesize_head {
            let head = if let Some(index) = inner_cast {
                // Source constraints and a cast's inner target are independent:
                // keep the prefix outside, and the cast inside the added pair.
                let prefix = &head.comments[..index];
                context.expression_start = self.format_expression_head(
                    expression,
                    NodeHead {
                        comments: prefix,
                        content_start: head.comments[index].span.start,
                        layout: head.layout,
                        gap_end: head.gap_end.map(|_| head.comments[index].span.start),
                    },
                    binding,
                    context,
                    array_comments.as_deref_mut(),
                    writer,
                )?;
                if context.expression_start.needs_asi_guard()
                    && self.options.semicolons == Semicolons::AsNeeded
                {
                    writer.text(";");
                    context.expression_start = ExpressionStart::StatementGuarded;
                }
                context.protected_head_start = Some(head.content_start);
                NodeHead {
                    comments: &head.comments[index..],
                    ..head
                }
            } else {
                head
            };
            return self.format_parenthesized_expression(
                expression,
                prepared,
                head,
                context,
                array_comments.as_deref_mut(),
                writer,
            );
        }
        if !head.comments.is_empty() && !matches!(pair_binding, CommentBinding::Unbound) {
            context.protected_head_start = Some(expression.span(self.ast).start);
        }
        if !head.comments.is_empty() {
            // Only an exterior packet can join a released head. Inspect a
            // replacement frame only after finding a removed, commented pair.
            let released_head = prepared
                .as_ref()
                .and_then(|content| {
                    let removed = content.parentheses?;
                    (removed.head.has_comments
                        && context.content_mode == ContentMode::Formatted
                        && context.parentheses_owner == ParenthesesOwner::Expression
                        && !self.expression_has_generated_head_frame(content, context))
                    .then(|| self.source_parentheses_comments(removed.source).head)
                })
                .unwrap_or(&[]);
            context.expression_start = self.format_expression_head_with_released_comments(
                expression,
                head,
                binding,
                released_head,
                context,
                array_comments.as_deref_mut(),
                writer,
            )?;
        }
        self.expression_layout(expression).write(writer, |writer| {
            self.format_expression_content_prepared(
                expression,
                prepared,
                context,
                array_comments,
                writer,
            )
        })
    }

    /// Required grammar and optional operand frames both enclose their heads.
    pub(in crate::formatter) fn expression_has_generated_head_frame(
        &self,
        content: &ExpressionContent,
        context: ExpressionContext,
    ) -> bool {
        self.expression_is_parenthesized(content.data, context)
            || self.is_nullish_conditional_operand(content.data, context.position)
    }

    fn format_expression_content_prepared(
        &mut self,
        expression: Expr,
        prepared: Option<ExpressionContent>,
        mut context: ExpressionContext,
        mut array_comments: Option<&mut ArrayCommentOwner<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        if context.content_mode == ContentMode::Verbatim
            && matches!(context.position, ExpressionPosition::ForInInitializer)
        {
            self.format_verbatim(expression.span(self.ast), writer)?;
            return Ok(None);
        }
        if let ExprData::ParenthesizedExpression(parenthesized) = self.ast.expr_data(expression) {
            if context.content_mode == ContentMode::Verbatim
                && !self.verbatim_parentheses_are_removable(parenthesized, context.position)
            {
                self.format_verbatim_expression(expression, context, writer)?;
                return Ok(None);
            }
            return self.format_source_parentheses(
                parenthesized,
                prepared,
                context,
                array_comments.as_deref_mut(),
                writer,
            );
        }
        if context.content_mode == ContentMode::Verbatim {
            self.format_verbatim_expression(expression, context, writer)?;
            return Ok(None);
        }
        let expression = prepared
            .as_ref()
            .map_or(expression, |content| content.expression);
        let data = self.ast.expr_data(expression);
        if let ExprData::SequenceExpression(sequence) = data {
            return self.format_sequence(sequence, context, writer);
        }
        let parenthesize = self.expression_is_parenthesized(data, context);
        if parenthesize
            && (Self::for_head_grammar_pair_is_enclosed(data, context.position)
                || matches!(data, ExprData::JsxElement(_) | ExprData::JsxFragment(_))
                    && Self::jsx_grammar_pair_is_enclosed(context.position))
        {
            return self.format_parenthesized_expression(
                expression,
                prepared,
                NodeHead {
                    comments: &[],
                    content_start: expression.span(self.ast).start,
                    layout: CommentRunLayout::Source,
                    gap_end: None,
                },
                context,
                array_comments,
                writer,
            );
        }

        let binary = super::binary::BinaryLike::from_data(data, self.ast);
        if context.expression_start == ExpressionStart::StatementPending
            && self.options.semicolons == Semicolons::AsNeeded
        {
            let mut asi_guard = parenthesize
                || match data {
                    ExprData::ArrayExpression(_)
                    | ExprData::TemplateLiteral(_)
                    | ExprData::RegExpLiteral(_)
                    | ExprData::TsTypeAssertion(_)
                    | ExprData::JsxElement(_)
                    | ExprData::JsxFragment(_) => true,
                    ExprData::UnaryExpression(unary) => matches!(
                        unary.operator(self.ast),
                        swc_next_ecma_ast::UnaryOperator::Positive
                            | swc_next_ecma_ast::UnaryOperator::Negate
                    ),
                    _ => false,
                };
            if let Some(binary) = binary {
                asi_guard |= self.expression_needs_parentheses(
                    self.expression_data(binary.left),
                    ExpressionPosition::BinaryOperand {
                        operator: binary.operator,
                        side: super::binary::OperandSide::Left,
                    },
                    ContentMode::Formatted,
                ) == Some(true);
            }
            if asi_guard {
                context.expression_start = ExpressionStart::StatementGuard;
            }
        }
        let conditional = matches!(data, ExprData::ConditionalExpression(_));
        if context.expression_start == ExpressionStart::StatementGuard
            && (binary.is_none() && !conditional || parenthesize)
        {
            writer.text(";");
            context.expression_start = ExpressionStart::None;
        }
        if let Some(binary) = binary {
            if parenthesize && Self::binary_grammar_pair_owns_indent(context.position) {
                return writer.group(false, |writer| {
                    writer.text("(");
                    writer.indent(|writer| {
                        writer.soft_line();
                        let tail = self.format_binary_enclosed(binary, writer)?;
                        self.format_expression_tail(tail, writer)
                    })?;
                    writer.soft_line();
                    writer.text(")");
                    Ok(None)
                });
            }
            if parenthesize {
                writer.text("(");
                context.expression_start = ExpressionStart::None;
            }
            let tail = self.format_binary(binary, context, writer)?;
            if parenthesize {
                writer.text(")");
            }
            return Ok(tail);
        }
        if let ExprData::ConditionalExpression(conditional) = data
            && parenthesize
            && Self::conditional_grammar_pair_is_enclosed(context)
        {
            return self.format_conditional_enclosed(conditional, context, writer);
        }
        if let ExprData::ConditionalExpression(conditional) = data
            && Self::conditional_test_pair_is_enclosed(context.position)
        {
            return writer.group(false, |writer| {
                writer.text("(");
                writer.indent(|writer| {
                    writer.soft_line();
                    let tail = self.format_conditional(
                        conditional,
                        context
                            .with_expression_start(ExpressionStart::None)
                            .with_operand_indent(OperandIndent::Parent),
                        writer,
                    )?;
                    self.format_expression_tail(tail, writer)
                })?;
                writer.soft_line();
                writer.text(")");
                Ok(None)
            });
        }
        if parenthesize {
            writer.text("(");
            context.expression_start = ExpressionStart::None;
        }
        let tail = self.format_expression_data(expression, context, parenthesize, writer)?;
        let tail = if parenthesize {
            self.format_parentheses_tail(
                context.source_parentheses_tail,
                tail,
                array_comments,
                writer,
            )?
        } else {
            tail
        };
        if parenthesize {
            writer.text(")");
        }
        Ok(tail)
    }

    pub(super) fn format_expression_head(
        &self,
        expression: Expr,
        head: NodeHead<'ast>,
        binding: CommentBinding,
        context: ExpressionContext,
        array_comments: Option<&mut ArrayCommentOwner<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<ExpressionStart, FormatError> {
        self.format_expression_head_with_released_comments(
            expression,
            head,
            binding,
            &[],
            context,
            array_comments,
            writer,
        )
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "Keep the released source head borrowed from the prepared expression owner."
    )]
    fn format_expression_head_with_released_comments(
        &self,
        expression: Expr,
        mut head: NodeHead<'ast>,
        binding: CommentBinding,
        released: &[Comment],
        mut context: ExpressionContext,
        mut array_comments: Option<&mut ArrayCommentOwner<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<ExpressionStart, FormatError> {
        let binding = context.resolve_emitted_head_binding(&head, binding);
        if context
            .expression_start
            .needs_statement_guard(self.options.semicolons)
            && context.content_mode == ContentMode::Verbatim
            && matches!(head.layout, CommentRunLayout::Source)
            && head.comments.last().is_some_and(|comment| {
                comment.kind == CommentKind::Block
                    && self.line_breaks(comment.span.end, head.content_start) == 0
            })
            && (matches!(
                context.expression_start,
                ExpressionStart::StatementGuard | ExpressionStart::StatementBarrier { .. }
            ) || self.expression_needs_asi_guard(expression, context)?)
        {
            head.gap_end = None;
        }
        if context
            .expression_start
            .needs_statement_guard(self.options.semicolons)
            && self.has_target_annotations
            && let Some(index) = head
                .comments
                .iter()
                .position(|&comment| self.comment_has_target_annotation(comment))
                .or_else(|| {
                    // Removing a wrapper joins this source-bound exterior run
                    // with its released annotation head. Choose the guard owner
                    // before either renderer claims or prints its own comments.
                    (!head.comments.is_empty()
                        && matches!(binding, CommentBinding::SourceBound)
                        && released
                            .iter()
                            .any(|&comment| self.comment_has_target_annotation(comment)))
                    .then_some(0)
                })
            && (matches!(
                context.expression_start,
                ExpressionStart::StatementGuard | ExpressionStart::StatementBarrier { .. }
            ) || self.expression_needs_asi_guard(expression, context)?)
        {
            let ignored_head =
                self.has_suppression && context.content_mode == ContentMode::Verbatim;
            let source_bound = matches!(binding, CommentBinding::SourceBound);
            let index = if source_bound { 0 } else { index };
            let (prefix, protected) = head.comments.split_at(index);
            self.format_node_head(
                NodeHead {
                    comments: prefix,
                    content_start: protected[0].span.start,
                    layout: head.layout,
                    gap_end: head.gap_end.map(|_| protected[0].span.start),
                },
                ExpressionStart::None,
                array_comments.as_deref_mut(),
                writer,
            )?;
            context.expression_start.write_statement_guard(writer);
            if source_bound {
                writer.hard_line();
            } else if ignored_head {
                let prefix = if self.source_position_starts_line(protected[0].span.start) {
                    LineConstraint::NewLine(1)
                } else {
                    LineConstraint::SameLine
                };
                prefix.write(writer, FlatSeparator::Empty, |_| {});
            } else if protected
                .iter()
                .any(|comment| comment.kind == CommentKind::Line)
                && !self.comment_is_type_cast(protected[0])
            {
                writer.space();
            }
            context.expression_start = ExpressionStart::StatementGuarded;
            head.comments = protected;
        }
        self.format_node_head(head, context.expression_start, array_comments, writer)
    }

    pub(in crate::formatter) fn format_node_head(
        &self,
        head: NodeHead<'ast>,
        mut start: ExpressionStart,
        array_comments: Option<&mut ArrayCommentOwner<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<ExpressionStart, FormatError> {
        if head.comments.is_empty() {
            return Ok(start);
        }
        if start.needs_statement_guard(self.options.semicolons)
            && (self.parenthesis_has_type_cast(head.content_start)
                || self.parenthesis_has_jsdoc_enclosure(head.content_start))
        {
            start.write_statement_guard(writer);
            start = ExpressionStart::None;
        }
        self.format_comment_run_with_array_comments(
            head.comments,
            head.layout,
            array_comments,
            writer,
        )?;
        if let Some(end) = head.gap_end
            && let Some(last) = head.comments.last()
        {
            self.comment_gap(last.span.end, end, writer);
        }
        Ok(start)
    }

    fn format_source_parentheses(
        &mut self,
        parenthesized: ParenthesizedExpression,
        prepared: Option<ExpressionContent>,
        mut context: ExpressionContext,
        mut array_comments: Option<&mut ArrayCommentOwner<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let span = parenthesized.span(self.ast);
        let ignored_child = match self.source_parentheses_choice(parenthesized, context) {
            SourceParenthesesChoice::Verbatim => {
                self.format_verbatim_expression(
                    Expr::ParenthesizedExpression(parenthesized),
                    context,
                    writer,
                )?;
                return Ok(None);
            }
            SourceParenthesesChoice::Retained { kind, mode } => {
                return self.format_retained_parentheses(
                    parenthesized,
                    context.with_content_mode(mode),
                    kind,
                    array_comments.as_deref_mut(),
                    writer,
                );
            }
            SourceParenthesesChoice::Content {
                mode,
                ignored_child,
            } => {
                context.content_mode = mode;
                ignored_child
            }
        };
        let content = prepared.unwrap_or_else(|| {
            self.expression_content(Expr::ParenthesizedExpression(parenthesized))
        });
        if ignored_child.is_none()
            && let Some(boundary) = self
                .type_binary_operand_parentheses(&content, context)
                .or_else(|| self.protected_grammar_parentheses(&content, context))
                .or_else(|| self.sequence_member_parentheses(&content, context))
        {
            return self.format_protected_grammar_parentheses(
                boundary,
                content,
                context,
                array_comments.as_deref_mut(),
                writer,
            );
        }
        if ignored_child.is_none() && content.parentheses.is_none() {
            return self.format_retained_parentheses(
                parenthesized,
                context,
                RetainedParentheses::Annotation,
                array_comments.as_deref_mut(),
                writer,
            );
        }
        if context.protected_head_start == Some(span.start) {
            context.protected_head_start = ignored_child
                .is_none()
                .then_some(content.expression.span(self.ast).start);
        }
        let inner = ignored_child.unwrap_or(content.expression);
        let inner_span = inner.span(self.ast);
        let leading = if ignored_child.is_some()
            || content
                .parentheses
                .is_some_and(|removed| removed.head.has_comments)
        {
            self.comments
                .between(Span::new(span.start, inner_span.start))?
        } else {
            &[]
        };
        self.comments.claim(leading)?;
        let layout = if matches!(context.position, ExpressionPosition::Statement) {
            CommentRunLayout::Source
        } else if matches!(content.data, ExprData::SequenceExpression(_))
            && context.content_mode == ContentMode::Formatted
            && ignored_child.is_none()
        {
            CommentRunLayout::GroupedLeading(inner_span.start)
        } else {
            CommentRunLayout::Leading(inner_span.start)
        };
        // Removing a source wrapper cannot release its prepared ignored owner.
        let mode = if context.content_mode == ContentMode::Verbatim || ignored_child.is_some() {
            ContentMode::Verbatim
        } else {
            self.content_mode(leading)
        };
        let binding = content
            .parentheses
            .map_or(CommentBinding::SourceBound, |removed| removed.head.binding);
        let owns_source_tail = context.source_tail_owner == SourceTailOwner::Enclosure
            && matches!(
                content.data,
                ExprData::SequenceExpression(_) | ExprData::AssignmentExpression(_)
            );
        if ignored_child.is_none()
            && owns_source_tail
            && self.expression_is_parenthesized(content.data, context)
        {
            context.source_parentheses_tail = content.parentheses.and_then(|removed| removed.tail);
        }
        let tail = self.format_expression_leading_prepared(
            inner,
            None,
            ResolvedExpressionHead {
                head: NodeHead {
                    comments: leading,
                    content_start: inner_span.start,
                    layout,
                    gap_end: matches!(context.position, ExpressionPosition::Statement)
                        .then_some(inner_span.start),
                },
                binding,
            },
            context.with_content_mode(mode),
            array_comments,
            writer,
        )?;
        let source_tail = content.parentheses.and_then(|removed| removed.tail);
        if ignored_child.is_some() {
            self.capture_expression_tail(Span::new(inner_span.end, span.end), tail)
        } else if let Some(source_tail) = source_tail {
            self.capture_expression_tail(source_tail.span, tail)
        } else {
            Ok(tail)
        }
    }

    fn format_retained_parentheses(
        &mut self,
        parenthesized: ParenthesizedExpression,
        context: ExpressionContext,
        kind: RetainedParentheses,
        array_comments: Option<&mut ArrayCommentOwner<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let span = parenthesized.span(self.ast);
        let child = parenthesized.expression(self.ast);
        let (inner, data) = if context.content_mode == ContentMode::Verbatim {
            (child, self.ast.expr_data(child))
        } else {
            let content = self.expression_content(child);
            (content.expression, content.data)
        };
        let inner_span = inner.span(self.ast);
        let leading = self
            .comments
            .between(Span::new(span.start, inner_span.start))?;
        let trailing = self.comments.in_range(Span::new(inner_span.end, span.end));
        let hug = match data {
            ExprData::ArrayExpression(array) => {
                leading.is_empty()
                    && trailing.is_empty()
                    && (!array.elements(self.ast).is_empty()
                        || self.comments.in_range(inner_span).is_empty())
            }
            ExprData::ObjectExpression(object) => {
                leading.is_empty()
                    && trailing.is_empty()
                    && (!object.properties(self.ast).is_empty()
                        || self.comments.in_range(inner_span).is_empty())
            }
            _ => false,
        };
        self.format_retained_source_parentheses(
            RetainedSourceParentheses {
                boundary: SourceParentheses {
                    outer: span,
                    inner: inner_span,
                    inner_target: ParenthesisInner::Expression(inner),
                },
                kind,
                hug,
                leading_mode: None,
            },
            context,
            array_comments,
            writer,
            |formatter, context, array_comments, writer| {
                formatter.format_expression_with_array_comments(
                    inner,
                    context,
                    array_comments,
                    writer,
                )
            },
        )
    }

    pub(super) fn format_retained_target_parentheses(
        &mut self,
        boundary: SourceParentheses,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
        content: impl FnOnce(
            &mut Self,
            ExpressionContext,
            &mut Writer<'ast>,
        ) -> Result<Option<ExpressionTail<'ast>>, FormatError>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let kind = if self.parenthesis_has_type_cast(boundary.outer.start) {
            RetainedParentheses::Cast
        } else {
            RetainedParentheses::Annotation
        };
        self.format_retained_source_parentheses(
            RetainedSourceParentheses {
                boundary,
                kind,
                hug: false,
                leading_mode: None,
            },
            context,
            None,
            writer,
            |formatter, context, _, writer| content(formatter, context, writer),
        )
    }

    fn format_retained_source_parentheses(
        &mut self,
        source: RetainedSourceParentheses,
        context: ExpressionContext,
        mut array_comments: Option<&mut ArrayCommentOwner<'ast>>,
        writer: &mut Writer<'ast>,
        content: impl FnOnce(
            &mut Self,
            ExpressionContext,
            Option<&mut ArrayCommentOwner<'ast>>,
            &mut Writer<'ast>,
        ) -> Result<Option<ExpressionTail<'ast>>, FormatError>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let RetainedSourceParentheses {
            boundary,
            kind,
            hug,
            leading_mode,
        } = source;
        let span = boundary.outer;
        let inner_span = boundary.inner;
        let source_comments = self.source_parentheses_comments(boundary);
        let mode = if context.content_mode == ContentMode::Verbatim {
            ContentMode::Verbatim
        } else {
            leading_mode.unwrap_or_else(|| self.content_mode(source_comments.head))
        };
        let sequence_head = if mode == ContentMode::Formatted
            && let ParenthesisInner::Expression(expression) = boundary.inner_target
            && let ExprData::SequenceExpression(sequence) = self.ast.expr_data(expression)
        {
            self.enclosed_sequence_head_start(sequence)
                .map(|start| (sequence, start))
        } else {
            None
        };
        let head_end = sequence_head.map_or(inner_span.start, |(_, start)| start);
        let source_leading = if sequence_head.is_some() {
            self.comments.in_range(Span::new(span.start, head_end))
        } else {
            source_comments.head
        };
        let leading = self.comments.between(Span::new(span.start, head_end))?;
        let trailing = source_comments.tail;
        let first = source_leading
            .first()
            .map_or(inner_span.start, |comment| comment.span.start);
        let binding =
            self.source_parentheses_head_binding(source_leading, boundary, &mut None, || {
                self.source_type_cast(inner_span.start)
            });
        let protected_head_start = (!source_leading.is_empty()
            && !matches!(
                self.optional_nullish_head_binding(source_leading, || boundary, binding),
                CommentBinding::Unbound
            ))
        .then_some(inner_span.start);
        let opening = if !matches!(kind, RetainedParentheses::FunctionReceiver)
            && (leading_mode.is_some() || matches!(binding, CommentBinding::SourceBound))
        {
            self.source_line_constraint(span.start + 1, first)
        } else {
            self.line_constraint(span.start + 1, first)
        };
        let last = trailing
            .last()
            .map_or(inner_span.end, |comment| comment.span.end);
        let closing = self.line_constraint(last, span.end - 1);
        let expand = matches!(kind, RetainedParentheses::Cast)
            && (source_comments
                .head
                .first()
                .is_some_and(|comment| self.line_breaks(span.start, comment.span.start) != 0)
                || trailing.first().is_some_and(|comment| {
                    self.line_breaks(inner_span.end, comment.span.start) != 0
                }));
        let leading_layout = if matches!(kind, RetainedParentheses::Annotation) {
            CommentRunLayout::FlatLeading(head_end)
        } else {
            CommentRunLayout::GroupedLeading(head_end)
        };
        if context.expression_start.needs_asi_guard()
            && self.options.semicolons == Semicolons::AsNeeded
        {
            writer.text(";");
        }
        let hug = hug
            && !matches!(opening, LineConstraint::NewLine(_))
            && !matches!(closing, LineConstraint::NewLine(_));
        let operand_indent = if (context.operand_indent == OperandIndent::Parent
            || matches!(kind, RetainedParentheses::Annotation))
            && !matches!(kind, RetainedParentheses::Cast)
            && matches!(boundary.inner_target, ParenthesisInner::Expression(expression) if matches!(self.ast.expr_data(expression), ExprData::BinaryExpression(_) | ExprData::LogicalExpression(_) | ExprData::ParenthesizedExpression(_)))
        {
            OperandIndent::Parent
        } else {
            OperandIndent::Expression
        };
        let write_contents = |writer: &mut Writer<'ast>, operand_indent| {
            self.comments.claim(leading)?;
            self.format_comment_run_with_array_comments(
                leading,
                leading_layout,
                array_comments.as_deref_mut(),
                writer,
            )?;
            let position = match boundary.inner_target {
                ParenthesisInner::Expression(expression) => {
                    self.type_assertion_parentheses_position(expression, None, context.position)
                }
                ParenthesisInner::SourcePair | ParenthesisInner::Unresolved => {
                    ExpressionPosition::Nested
                }
            };
            let mut inner_context = if matches!(kind, RetainedParentheses::FunctionReceiver) {
                context.inside_parentheses()
            } else {
                match (mode, context.position) {
                    (ContentMode::Verbatim, ExpressionPosition::UnaryArgument(operator)) => {
                        ExpressionContext::at(ExpressionPosition::ParenthesizedUnaryArgument(
                            operator,
                        ))
                    }
                    (
                        _,
                        position @ (ExpressionPosition::MemberAssignmentObject
                        | ExpressionPosition::AssignmentTarget
                        | ExpressionPosition::UpdateArgument),
                    ) => ExpressionContext::at(position).inside_parentheses(),
                    _ => ExpressionContext::at(position).inside_parentheses(),
                }
            }
            .with_content_mode(mode)
            .with_operand_indent(operand_indent)
            .with_tail_owner(context.tail_owner);
            inner_context.member_chain_layout = context.member_chain_layout;
            if context.member_chain_layout == super::MemberChainLayout::Local {
                inner_context.member_layout = context.member_layout;
            }
            inner_context.protected_head_start = protected_head_start;
            if let Some((sequence, _)) = sequence_head {
                inner_context.head_comments = super::HeadComments::ClaimedSequence(sequence);
            }
            let tail = content(self, inner_context, array_comments.as_deref_mut(), writer)?;
            let tail = self.capture_expression_tail(Span::new(inner_span.end, span.end), tail)?;
            self.format_expression_tail_with_array_comments(tail, array_comments, writer)
        };
        let write = |writer: &mut Writer<'ast>| {
            writer.text("(");
            if hug {
                write_contents(writer, OperandIndent::Expression)?;
            } else {
                writer.indent(|writer| {
                    opening.write(writer, FlatSeparator::Empty, Writer::soft_line);
                    write_contents(writer, operand_indent)
                })?;
                if trailing
                    .last()
                    .is_some_and(|comment| comment.kind == CommentKind::Line)
                {
                    writer.hard_line();
                } else {
                    closing.write(writer, FlatSeparator::Empty, Writer::soft_line);
                }
            }
            writer.text(")");
            Ok(None)
        };
        if matches!(kind, RetainedParentheses::FunctionReceiver) {
            write(writer)
        } else {
            writer.group(expand, write)
        }
    }
    pub(super) fn generated_enclosure_owner(
        &self,
        expression: Expr,
        content: &ExpressionContent,
        head: &NodeHead<'ast>,
        binding: CommentBinding,
        context: ExpressionContext,
    ) -> Option<GeneratedEnclosureOwner> {
        if !self.has_type_casts
            || head.comments.is_empty()
            || !matches!(binding, CommentBinding::Unbound)
            || context.content_mode != ContentMode::Formatted
            || context.parentheses_owner != ParenthesesOwner::Expression
            || self.expression_needs_parentheses(
                content.data,
                context.position,
                context.content_mode,
            ) != Some(true)
        {
            return None;
        }
        // This complete exterior run is the emitted adjacency owner. A binary
        // hoist may cross removed source openers before reaching its lexical head.
        let annotation = self.source_cast_before(head.content_start)?;
        if !head.comments.contains(&annotation)
            || self.diagnostic_range_is_sensitive(Span::new(
                annotation.span.start,
                expression.span(self.ast).end,
            ))
        {
            return None;
        }
        Some(GeneratedEnclosureOwner {
            expression,
            annotation,
        })
    }

    pub(super) fn optional_nullish_head_binding(
        &self,
        comments: &[Comment],
        boundary: impl FnOnce() -> SourceParentheses,
        binding: CommentBinding,
    ) -> CommentBinding {
        if !matches!(binding, CommentBinding::SourceBound) || !self.has_target_annotations {
            return binding;
        }
        let mut optimizer = false;
        for &comment in comments {
            let semantics = self.comment_semantics(comment);
            if semantics.type_cast
                || semantics.suppression
                || semantics.diagnostic
                || semantics.file_directive
            {
                return binding;
            }
            optimizer |= semantics.optimizer;
        }
        if optimizer {
            self.optimizer_head_binding(boundary())
        } else {
            binding
        }
    }

    /// Resolve only an exterior packet's actual lexical target. Retained source
    /// pairs and prefix syntax keep the source owner's existing decision.
    fn optimizer_head_binding(&self, boundary: SourceParentheses) -> CommentBinding {
        let ParenthesisInner::Expression(mut expression) = boundary.inner_target else {
            return CommentBinding::SourceBound;
        };
        let mut head_start = boundary.inner.start;
        let mut removed_binding = CommentBinding::Unbound;
        loop {
            if expression.span(self.ast).start != head_start {
                return CommentBinding::SourceBound;
            }
            let data = self.ast.expr_data(expression);
            expression = match data {
                ExprData::ParenthesizedExpression(_) => {
                    let content = self.expression_content(expression);
                    let Some(removed) = content.parentheses else {
                        return CommentBinding::SourceBound;
                    };
                    removed_binding = removed_binding.combine(removed.head.binding);
                    if matches!(removed_binding, CommentBinding::SourceBound) {
                        return CommentBinding::SourceBound;
                    }
                    // Reuse the selected removal and require its known target to
                    // agree with the complete lexical target exposed below.
                    head_start = content.expression.span(self.ast).start;
                    content.expression
                }
                ExprData::CallExpression(_)
                | ExprData::NewExpression(_)
                | ExprData::Function(_)
                | ExprData::ArrowFunctionExpression(_) => {
                    return removed_binding
                        .combine(CommentBinding::BoundTo(expression.span(self.ast)));
                }
                ExprData::ChainExpression(node) => node.expression(self.ast),
                ExprData::MemberExpression(node) => node.object(self.ast),
                ExprData::ConditionalExpression(node) => node.test(self.ast),
                ExprData::SequenceExpression(node) => {
                    let Some(first) = node.expressions(self.ast).first() else {
                        return CommentBinding::SourceBound;
                    };
                    self.ast.get_node_in_sub_range(first)
                }
                ExprData::IdentifierReference(_)
                | ExprData::StringLiteral(_)
                | ExprData::BooleanLiteral(_)
                | ExprData::NullLiteral(_)
                | ExprData::NumericLiteral(_)
                | ExprData::BigIntLiteral(_)
                | ExprData::RegExpLiteral(_)
                | ExprData::ThisExpression(_) => {
                    return if removed_binding.allows_relocation(Some(expression.span(self.ast))) {
                        CommentBinding::Unbound
                    } else {
                        CommentBinding::SourceBound
                    };
                }
                _ => match super::binary::BinaryLike::from_data(data, self.ast) {
                    Some(binary) => binary.left,
                    None => return CommentBinding::SourceBound,
                },
            };
        }
    }

    pub(in crate::formatter) fn expression_head_binding(
        &self,
        comments: &[Comment],
        expression: Expr,
    ) -> CommentBinding {
        let span = expression.span(self.ast);
        self.source_parentheses_head_binding(
            comments,
            SourceParentheses {
                outer: span,
                inner: span,
                inner_target: ParenthesisInner::Expression(expression),
            },
            &mut None,
            || self.source_type_cast(span.start),
        )
    }

    /// A released packet can expose an optimizer to an enclosing continuation.
    /// Keep its active inner owner while preparing the complete emitted binding.
    pub(super) fn prospective_expression_head_binding(
        &self,
        comments: &[Comment],
        expression: Expr,
        binding: CommentBinding,
    ) -> CommentBinding {
        if !comments.is_empty() && matches!(binding, CommentBinding::BoundTo(_)) {
            binding.combine(self.expression_head_binding(comments, expression))
        } else {
            binding
        }
    }

    /// Discover a removed source packet that precedes the complete emitted head.
    /// The original expression renderer retains ownership of the borrowed packet.
    pub(super) fn exposed_expression_head(
        &self,
        mut expression: Expr,
        mut context: ExpressionContext,
    ) -> Result<Option<ResolvedExpressionHead<'ast>>, FormatError> {
        let owner = expression;
        if self.comments.in_range(expression.span(self.ast)).is_empty() {
            return Ok(None);
        }
        loop {
            if context.content_mode != ContentMode::Formatted {
                return Ok(None);
            }
            let data = self.ast.expr_data(expression);
            if let ExprData::ParenthesizedExpression(node) = data {
                let Some(content) = self.prepare_chain_parentheses_head(node, context) else {
                    return Ok(None);
                };
                let Some(removed) = content.parentheses else {
                    return Ok(None);
                };
                if matches!(removed.head.binding, CommentBinding::SourceBound)
                    || self.expression_needs_parentheses(
                        content.data,
                        context.position,
                        context.content_mode,
                    ) != Some(false)
                {
                    return Ok(None);
                }
                let comments = self.comments.in_range(Span::new(
                    removed.source.outer.start,
                    removed.source.inner.start,
                ));
                if !comments.is_empty() {
                    return Ok(Some(ResolvedExpressionHead {
                        head: NodeHead {
                            comments,
                            content_start: removed.source.inner.start,
                            layout: CommentRunLayout::GroupedLeading(removed.source.inner.start),
                            gap_end: None,
                        },
                        binding: self.prospective_expression_head_binding(
                            comments,
                            owner,
                            removed.head.binding,
                        ),
                    }));
                }
                expression = content.expression;
                continue;
            }
            if self.expression_is_parenthesized(data, context) {
                return Ok(None);
            }
            let Some((child, child_context)) = self.expression_head_operand(data, context)? else {
                return Ok(None);
            };
            expression = child;
            context = child_context;
        }
    }

    /// An emitted enclosure can own a complete head independently of its caller's gap.
    pub(super) fn enclosed_expression_head_binding(
        &self,
        content: &ExpressionContent,
        comments: &[Comment],
    ) -> Option<CommentBinding> {
        if comments.is_empty()
            || !matches!(
                content.data,
                ExprData::JsxElement(_) | ExprData::JsxFragment(_)
            ) && !matches!(content.data, ExprData::SequenceExpression(sequence)
                    if self.sequence_head_has_type_cast(sequence, comments))
        {
            return None;
        }
        let binding = self.expression_head_binding(comments, content.expression);
        Some(
            content
                .parentheses
                .map_or(binding, |removed| binding.combine(removed.head.binding)),
        )
    }

    pub(super) fn emitted_head_annotation(
        &self,
        comments: &[Comment],
    ) -> Option<EmittedHeadAnnotation> {
        if !self.has_type_casts {
            return None;
        }
        let comment = comments
            .iter()
            .rev()
            .find(|comment| self.source(comment.span).starts_with("/**"))?;
        // Ordinary trivia is transparent; a newer JSDoc replaces the older one.
        self.comment_is_type_cast(*comment)
            .then_some(EmittedHeadAnnotation { comment: *comment })
    }

    pub(super) fn emitted_head_enclosure_owner(
        &self,
        expression: Expr,
        content: &ExpressionContent,
        annotation: EmittedHeadAnnotation,
        binding: CommentBinding,
        context: ExpressionContext,
    ) -> Option<GeneratedEnclosureOwner> {
        if !matches!(binding, CommentBinding::Unbound)
            || context.content_mode != ContentMode::Formatted
            || context.parentheses_owner != ParenthesesOwner::Expression
            || self.expression_needs_parentheses(
                content.data,
                context.position,
                context.content_mode,
            ) != Some(true)
            || self.diagnostic_range_is_sensitive(Span::new(
                annotation.comment.span.start,
                expression.span(self.ast).end,
            ))
        {
            return None;
        }
        Some(GeneratedEnclosureOwner {
            expression,
            annotation: annotation.comment,
        })
    }

    pub(in crate::formatter) fn format_prepared_expression_with_resolved_head(
        &mut self,
        expression: Expr,
        content: ExpressionContent,
        resolved: ResolvedExpressionHead<'ast>,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_expression_leading_prepared(
            expression,
            Some(content),
            resolved,
            context,
            None,
            writer,
        )
    }

    pub(in crate::formatter) fn format_prepared_expression_with_leading_comments(
        &mut self,
        expression: Expr,
        content: ExpressionContent,
        head: NodeHead<'ast>,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let binding = self.expression_head_binding(head.comments, expression);
        let binding = content
            .parentheses
            .map_or(binding, |removed| binding.combine(removed.head.binding));
        self.format_prepared_expression_with_resolved_head(
            expression,
            content,
            ResolvedExpressionHead { head, binding },
            context,
            writer,
        )
    }

    pub(super) fn format_prepared_generated_enclosure_with_head(
        &mut self,
        enclosure: PreparedGeneratedEnclosure,
        head: NodeHead<'ast>,
        mut context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        context.expression_start = self.format_expression_head(
            enclosure.owner.expression,
            head,
            CommentBinding::Unbound,
            context,
            None,
            writer,
        )?;
        self.format_prepared_generated_enclosure(enclosure, context, writer)
    }

    pub(super) fn format_prepared_generated_enclosure(
        &mut self,
        enclosure: PreparedGeneratedEnclosure,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_generated_enclosure(
            enclosure.owner,
            Some(enclosure.content),
            context,
            None,
            writer,
        )
    }

    fn format_generated_enclosure(
        &mut self,
        owner: GeneratedEnclosureOwner,
        prepared: Option<ExpressionContent>,
        context: ExpressionContext,
        array_comments: Option<&mut ArrayCommentOwner<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let GeneratedEnclosureOwner {
            expression,
            annotation: _annotation,
        } = owner;
        let start = prepared
            .as_ref()
            .map_or(expression.span(self.ast).start, |content| {
                content.expression.span(self.ast).start
            });
        self.format_parenthesized_expression(
            expression,
            prepared,
            NodeHead {
                comments: &[],
                content_start: start,
                layout: CommentRunLayout::FlatLeading(start),
                gap_end: None,
            },
            context,
            array_comments,
            writer,
        )
    }

    pub(super) fn format_parenthesized_expression(
        &mut self,
        expression: Expr,
        mut prepared: Option<ExpressionContent>,
        head: NodeHead<'ast>,
        context: ExpressionContext,
        mut array_comments: Option<&mut ArrayCommentOwner<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        if !head.comments.is_empty() && context.content_mode == ContentMode::Formatted {
            let content = prepared.get_or_insert_with(|| self.expression_content(expression));
            if let ExprData::SequenceExpression(sequence) = content.data {
                let source_tail = content.parentheses.and_then(|removed| removed.tail);
                let mut context = context;
                context.parentheses_owner = ParenthesesOwner::Expression;
                if context.source_tail_owner == SourceTailOwner::Enclosure {
                    context.source_parentheses_tail =
                        source_tail.or(context.source_parentheses_tail);
                }
                let tail = self.format_sequence_with_leading_comments(
                    sequence,
                    head.comments,
                    context,
                    writer,
                )?;
                return match source_tail {
                    Some(source) => self.capture_expression_tail(source.span, tail),
                    None => Ok(tail),
                };
            }
        }
        if context.expression_start.needs_asi_guard()
            && self.options.semicolons == Semicolons::AsNeeded
        {
            writer.text(";");
        }
        let position = self.type_assertion_parentheses_position(
            expression,
            prepared.as_ref(),
            context.position,
        );
        let operand_indent = if self.is_nullish_conditional_operand(
            prepared
                .get_or_insert_with(|| self.expression_content(expression))
                .data,
            context.position,
        ) {
            OperandIndent::Parent
        } else {
            OperandIndent::Expression
        };
        let protected = context.protected_head_start == Some(head.content_start);
        let opening = if protected {
            self.line_constraint(head.content_start, head.content_start)
        } else {
            LineConstraint::Unconstrained
        };
        let closing = if protected {
            let end = expression.span(self.ast).end;
            self.line_constraint(end, end)
        } else {
            LineConstraint::Unconstrained
        };
        writer.group(false, |writer| {
            writer.text("(");
            let tail = writer.indent(|writer| {
                opening.write(writer, FlatSeparator::Empty, Writer::soft_line);
                self.format_comment_run_with_array_comments(
                    head.comments,
                    if matches!(
                        self.ast.expr_data(expression),
                        ExprData::Function(_) | ExprData::ArrowFunctionExpression(_)
                    ) {
                        CommentRunLayout::GroupedLeading(head.content_start)
                    } else {
                        CommentRunLayout::FlatLeading(head.content_start)
                    },
                    array_comments.as_deref_mut(),
                    writer,
                )?;
                let tail = self.format_expression_prepared_with_array_comments(
                    expression,
                    prepared,
                    ExpressionContext::at(position)
                        .with_operand_indent(operand_indent)
                        .with_content_mode(context.content_mode)
                        .with_tail_owner(context.tail_owner)
                        .with_member_chain_layout(context.member_chain_layout)
                        .with_target_member_intent(context)
                        .inside_parentheses(),
                    array_comments.as_deref_mut(),
                    writer,
                )?;
                let tail = self.format_parentheses_tail(
                    context.source_parentheses_tail,
                    tail,
                    array_comments.as_deref_mut(),
                    writer,
                )?;
                if context.tail_owner == TailOwner::ParentOperation {
                    Ok(tail)
                } else {
                    self.format_expression_tail_with_array_comments(tail, array_comments, writer)?;
                    Ok(None)
                }
            })?;
            closing.write(writer, FlatSeparator::Empty, Writer::soft_line);
            writer.text(")");
            Ok(tail)
        })
    }

    pub(super) fn protected_grammar_parentheses(
        &self,
        content: &ExpressionContent,
        context: ExpressionContext,
    ) -> Option<SourceParentheses> {
        if context.content_mode != ContentMode::Formatted {
            return None;
        }
        let boundary = content.parentheses?.source;
        let breakable = match content.data {
            ExprData::BinaryExpression(_) | ExprData::LogicalExpression(_) => {
                Self::binary_grammar_pair_owns_indent(context.position)
            }
            ExprData::ConditionalExpression(_) => {
                matches!(context.position, ExpressionPosition::ConditionalTest)
                    || matches!(
                        context.position,
                        ExpressionPosition::MemberObject(_)
                            | ExpressionPosition::MemberAssignmentObject
                    ) && (context.conditional_closing == super::ConditionalClosing::MemberLine
                        || context.conditional_envelope
                            == super::ConditionalEnvelope::OperationHead)
            }
            ExprData::JsxElement(_) | ExprData::JsxFragment(_) => {
                Self::jsx_grammar_pair_is_enclosed(context.position)
            }
            _ => false,
        };
        if !breakable
            || self.line_constraint(boundary.outer.start + 1, boundary.inner.start)
                != LineConstraint::SameLine
                && self.line_constraint(boundary.inner.end, boundary.outer.end - 1)
                    != LineConstraint::SameLine
            || !self.expression_is_parenthesized(content.data, context)
        {
            return None;
        }
        Some(boundary)
    }

    pub(super) fn format_protected_grammar_parentheses(
        &mut self,
        boundary: SourceParentheses,
        content: ExpressionContent,
        context: ExpressionContext,
        array_comments: Option<&mut ArrayCommentOwner<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let context = if matches!(
            content.data,
            ExprData::BinaryExpression(_) | ExprData::LogicalExpression(_)
        ) && Self::binary_grammar_pair_owns_indent(context.position)
        {
            context.with_operand_indent(OperandIndent::Parent)
        } else {
            context
        };
        self.format_retained_source_parentheses(
            RetainedSourceParentheses {
                boundary,
                kind: RetainedParentheses::Annotation,
                hug: false,
                leading_mode: None,
            },
            context,
            array_comments,
            writer,
            |formatter, inner, array_comments, writer| {
                formatter.format_expression_prepared_with_array_comments(
                    content.expression,
                    Some(content),
                    inner.with_tail_owner(context.tail_owner),
                    array_comments,
                    writer,
                )
            },
        )
    }

    fn type_binary_operand_parentheses(
        &self,
        content: &ExpressionContent,
        context: ExpressionContext,
    ) -> Option<SourceParentheses> {
        if context.content_mode != ContentMode::Formatted
            || !matches!(context.position, ExpressionPosition::TypeBinaryOperand)
        {
            return None;
        }
        let removed = content.parentheses?;
        let tail = removed.tail?;
        let comments = self.source_parentheses_comments(removed.source).tail;
        let last = comments.last()?;
        // Source-separated or multiline tail packets must close before `as`/`satisfies`.
        // An inline line suffix can remain pending through the type operator.
        (self.line_breaks(tail.span.start, last.span.end) != 0).then_some(removed.source)
    }

    fn ignored_operand_owns_operation_head(
        &self,
        expression: Expr,
        position: ExpressionPosition,
    ) -> bool {
        match position {
            ExpressionPosition::BinaryOperand {
                operator,
                side: super::binary::OperandSide::Left,
            } => super::binary::BinaryLike::from_data(self.ast.expr_data(expression), self.ast)
                .is_none_or(|child| !operator.flattens(child.operator)),
            ExpressionPosition::ConditionalTest
            | ExpressionPosition::TypeBinaryOperand
            | ExpressionPosition::NonNullOperand
            | ExpressionPosition::InstantiationOperand
            | ExpressionPosition::TemplateTag => true,
            _ => false,
        }
    }
}
