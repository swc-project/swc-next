//! Conditional positions, aligned branches, and prepared operator boundaries.

use swc_next_ecma_ast::{Comment, ConditionalExpression, Expr, ExprData, GetSpan, Span};

use super::parentheses::{
    EmittedHeadAnnotation, ExpressionContent, PreparedGeneratedEnclosure, ResolvedExpressionHead,
};
use super::{
    ExpressionContext, ExpressionOutput, ExpressionPosition, ExpressionStart, Formatter,
    OperandIndent, TailOwner,
};
use crate::formatter::comments::{
    CommentBinding, CommentGap, CommentRunLayout, ExpressionTail, NodeHead,
};
use crate::formatter::diagnostics::{FlatSeparator, LineConstraint};
use crate::formatter::source::is_source_whitespace;
use crate::formatter::suppression::{ContentMode, SuppressionPosition};
use crate::{FormatError, IndentStyle, document::Writer};

/// Continuation responsibility follows only an operation's actual head edge.
/// A direct root has no extra enclosure; a head borrows that root's scope.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ConditionalEnvelope {
    Local,
    ContinuationRoot,
    OperationHead,
}

impl ConditionalEnvelope {
    pub(super) fn terminal_head(self) -> Self {
        match self {
            Self::ContinuationRoot => Self::OperationHead,
            Self::Local | Self::OperationHead => Self::Local,
        }
    }
    pub(super) fn chain_head(self) -> Self {
        match self {
            Self::Local => Self::Local,
            Self::ContinuationRoot | Self::OperationHead => Self::OperationHead,
        }
    }
}

/// The immediate suffix owns the closing pair's optional break.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ConditionalClosing {
    Inline,
    MemberLine,
}

/// Branches share a root's layout decision; tests begin a separate chain.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ConditionalChain {
    Root,
    Standard,
    Jsx { first: Expr },
}

impl ConditionalChain {
    fn is_jsx(self) -> bool {
        matches!(self, Self::Jsx { .. })
    }
}

#[derive(Clone, Copy)]
enum ConditionalLayout {
    Root,
    Test,
    Consequent,
    Alternate,
}

struct ConditionalOperand {
    original: Expr,
    source_end: u32,
    operand_end: u32,
    source_start: u32,
}

struct PreparedConditional<'ast> {
    test: ConditionalOperand,
    consequent: ConditionalOperand,
    alternate: ConditionalOperand,
    question: ConditionalBoundary<'ast>,
    colon: ConditionalBoundary<'ast>,
}

struct ConditionalBranch<'ast> {
    operand: ConditionalOperand,
    operand_indent: OperandIndent,
    position: ExpressionPosition,
    boundary: ResolvedConditionalBoundary<'ast>,
    following_mode: ContentMode,
}

/// Source effects are prepared before either operand is printed. Ordinary
/// attachment waits for the preceding operand's complete pending packet.
struct ConditionalBoundary<'ast> {
    operator: &'static str,
    source_range: Span,
    comments: &'ast [Comment],
    source: Option<ConditionalSourceBoundary<'ast>>,
}

struct ConditionalSourceBoundary<'ast> {
    operator_start: u32,
    binding: CommentBinding,
    previous_mode: ContentMode,
    next_mode: ContentMode,
    suppression: Option<ConditionalSuppression<'ast>>,
    before_operator: LineConstraint,
    after_operator: LineConstraint,
}

/// Source suppression targets do not change when an inner envelope releases comments.
struct ConditionalSuppression<'ast> {
    previous: &'ast [Comment],
    following: &'ast [Comment],
}

impl ConditionalBoundary<'_> {
    fn previous_mode(&self) -> ContentMode {
        self.source
            .as_ref()
            .map_or(ContentMode::Formatted, |source| source.previous_mode)
    }
}

/// One claimed packet supplies the previous tail and the following node head.
/// Its following content and head share that final packet scope. No already
/// printed operand can change mode.
struct ResolvedConditionalBoundary<'ast> {
    operator: &'static str,
    leading: &'ast [Comment],
    content: ExpressionContent,
    annotation: Option<EmittedHeadAnnotation>,
    next_binding: CommentBinding,
    next_mode: ContentMode,
    before_operator: LineConstraint,
    after_operator: LineConstraint,
    deferred_tail: DeferredConditionalTail<'ast>,
}

/// Only the exterior packet can outlive a generated JSX pair.
enum DeferredConditionalTail<'ast> {
    Empty,
    Whole {
        outer: &'ast [Comment],
        boundary_start: u32,
    },
    Split {
        outer: &'ast [Comment],
        previous_count: usize,
        boundary_start: u32,
    },
}

/// The source packet is partitioned before any of its comments are emitted.
enum ConditionalTail<'ast> {
    Empty,
    Whole {
        comments: &'ast [Comment],
        source_end: u32,
    },
    Released {
        inner: &'ast [Comment],
        previous: &'ast [Comment],
        following: &'ast [Comment],
        source_end: u32,
        boundary_start: u32,
        gap: Option<CommentGap>,
        split_following: bool,
    },
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_conditional_enclosed(
        &mut self,
        conditional: ConditionalExpression,
        context: ExpressionContext<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let prepared = self.prepare_conditional(conditional)?;
        let chain = self.resolve_conditional_chain(context.conditional_chain, &prepared);
        let context = context.with_conditional_chain(chain);
        let continuation = context.conditional_envelope == ConditionalEnvelope::OperationHead;
        let closing_line = continuation
            || !chain.is_jsx() && Self::conditional_grammar_pair_has_breakable_close(context);
        writer.group(false, |writer| {
            writer.text("(");
            let mut context = context.with_expression_start(ExpressionStart::None);
            context.protected_head_start = None;
            let write = |writer: &mut Writer<'ast>| {
                let context = if continuation {
                    context.with_operand_indent(OperandIndent::Parent)
                } else {
                    context
                };
                let output = self.format_prepared_conditional(prepared, context, writer)?;
                self.format_expression_tail(output.pending, writer)
            };
            if continuation {
                writer.indent(|writer| {
                    writer.soft_line();
                    write(writer)
                })?;
            } else {
                write(writer)?;
            }
            if closing_line {
                writer.soft_line();
            }
            writer.text(")");
            Ok(None)
        })
    }

    pub(super) fn format_conditional(
        &mut self,
        conditional: ConditionalExpression,
        context: ExpressionContext<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_conditional_with_raw_end(conditional, context, writer)
            .map(|output| output.pending)
    }

    pub(super) fn format_conditional_with_raw_end(
        &mut self,
        conditional: ConditionalExpression,
        context: ExpressionContext<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<ExpressionOutput<'ast>, FormatError> {
        let prepared = self.prepare_conditional(conditional)?;
        self.format_prepared_conditional(prepared, context, writer)
    }

    fn format_prepared_conditional(
        &mut self,
        prepared: PreparedConditional<'ast>,
        context: ExpressionContext<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<ExpressionOutput<'ast>, FormatError> {
        let layout = match context.position {
            ExpressionPosition::ConditionalTest => ConditionalLayout::Test,
            ExpressionPosition::ConditionalConsequent => ConditionalLayout::Consequent,
            ExpressionPosition::ConditionalAlternate => ConditionalLayout::Alternate,
            _ => ConditionalLayout::Root,
        };
        let chain = self.resolve_conditional_chain(context.conditional_chain, &prepared);
        let write = |writer: &mut Writer<'ast>| {
            self.format_conditional_contents(prepared, context, layout, chain, writer)
        };
        match layout {
            ConditionalLayout::Root | ConditionalLayout::Test => writer.group(false, write),
            ConditionalLayout::Consequent | ConditionalLayout::Alternate => write(writer),
        }
    }

    pub(super) fn conditional_grammar_pair_has_breakable_close(
        context: ExpressionContext<'ast>,
    ) -> bool {
        context.conditional_envelope == ConditionalEnvelope::OperationHead
            || context.conditional_closing == ConditionalClosing::MemberLine
    }

    fn resolve_conditional_chain(
        &self,
        chain: ConditionalChain,
        prepared: &PreparedConditional<'_>,
    ) -> ConditionalChain {
        match chain {
            ConditionalChain::Root if self.ast.is_jsx() => self.conditional_chain(prepared),
            ConditionalChain::Root => ConditionalChain::Standard,
            chain => chain,
        }
    }

    fn conditional_chain(&self, prepared: &PreparedConditional<'_>) -> ConditionalChain {
        fn first_jsx(formatter: &Formatter<'_, '_>, mut expression: Expr) -> Option<Expr> {
            loop {
                match formatter.ast.expr_data(expression) {
                    ExprData::JsxElement(_) | ExprData::JsxFragment(_) => return Some(expression),
                    ExprData::ParenthesizedExpression(node) => {
                        expression = node.expression(formatter.ast)
                    }
                    ExprData::ConditionalExpression(node) => {
                        return [
                            node.test(formatter.ast),
                            node.consequent(formatter.ast),
                            node.alternate(formatter.ast),
                        ]
                        .into_iter()
                        .find_map(|expression| first_jsx(formatter, expression));
                    }
                    _ => return None,
                }
            }
        }
        let first = [
            prepared.test.original,
            prepared.consequent.original,
            prepared.alternate.original,
        ]
        .into_iter()
        .find_map(|expression| first_jsx(self, expression));
        first.map_or(ConditionalChain::Standard, |first| ConditionalChain::Jsx {
            first,
        })
    }

    fn conditional_test_chain(&self, chain: ConditionalChain, test: Expr) -> ConditionalChain {
        let ConditionalChain::Jsx { first } = chain else {
            return chain;
        };
        let test = test.span(self.ast);
        let first = first.span(self.ast);
        // The witness is the first JSX in source-order traversal. Tests before
        // it were already checked; only tests after it remain unvisited.
        if test.end <= first.start {
            ConditionalChain::Standard
        } else if test.start <= first.start && first.end <= test.end {
            chain
        } else {
            ConditionalChain::Root
        }
    }

    fn prepare_conditional(
        &self,
        conditional: ConditionalExpression,
    ) -> Result<PreparedConditional<'ast>, FormatError> {
        let test = self.conditional_operand(conditional.test(self.ast));
        let consequent = self.conditional_operand(conditional.consequent(self.ast));
        let alternate = self.conditional_operand(conditional.alternate(self.ast));
        let question = self.conditional_boundary(&test, &consequent, "?")?;
        let colon = self.conditional_boundary(&consequent, &alternate, ":")?;
        Ok(PreparedConditional {
            test,
            consequent,
            alternate,
            question,
            colon,
        })
    }

    fn conditional_operand(&self, original: Expr) -> ConditionalOperand {
        let mut operand = original;
        if self.has_suppression {
            while let ExprData::ParenthesizedExpression(pair) = self.ast.expr_data(operand) {
                operand = pair.expression(self.ast);
            }
        }
        ConditionalOperand {
            original,
            source_end: original.span(self.ast).end,
            operand_end: operand.span(self.ast).end,
            // The operand owns its inner head even when a source wrapper can
            // disappear. A following ignore can still retain that raw wrapper.
            source_start: original.span(self.ast).start,
        }
    }

    fn format_conditional_contents(
        &mut self,
        prepared: PreparedConditional<'ast>,
        context: ExpressionContext<'ast>,
        layout: ConditionalLayout,
        chain: ConditionalChain,
        writer: &mut Writer<'ast>,
    ) -> Result<ExpressionOutput<'ast>, FormatError> {
        let PreparedConditional {
            test,
            consequent,
            alternate,
            question,
            colon,
        } = prepared;
        let operand_indent = match context.position {
            ExpressionPosition::Nested
            | ExpressionPosition::RestrictedArgument
            | ExpressionPosition::CallCallee { .. }
            | ExpressionPosition::NewCallee => context.operand_indent,
            _ => OperandIndent::Parent,
        };
        let default_before_operator = if chain.is_jsx() {
            LineConstraint::SameLine
        } else {
            LineConstraint::Unconstrained
        };
        let test_chain = self.conditional_test_chain(chain, test.original);
        let mut write_test = |writer: &mut Writer<'ast>| {
            let mut test_context = ExpressionContext::at(ExpressionPosition::ConditionalTest)
                .with_parent_head(context)
                .with_operand_indent(operand_indent)
                .with_conditional_chain(test_chain)
                .with_content_mode(question.previous_mode())
                .with_tail_owner(TailOwner::ParentOperation);
            if question.previous_mode() == ContentMode::Verbatim {
                test_context.ignored_parentheses_closing =
                    Some(Span::new(test.operand_end, test.source_end));
            }
            self.format_prepared_expression_with_leading_comments(
                test.original,
                self.expression_content(test.original),
                NodeHead {
                    comments: &[],
                    content_start: test.source_start,
                    layout: CommentRunLayout::Source,
                    gap_end: None,
                },
                test_context,
                writer,
            )
        };
        let tail = if matches!(layout, ConditionalLayout::Alternate) {
            writer.align(2, write_test)?
        } else {
            write_test(writer)?
        };
        let question = self.resolve_conditional_boundary(
            question,
            &consequent,
            tail,
            default_before_operator,
            false,
            writer,
        )?;
        let options = self.options;
        let write_branches = |writer: &mut Writer<'ast>| {
            let colon = self.format_conditional_branch(
                ConditionalBranch {
                    operand: consequent,
                    operand_indent,
                    position: ExpressionPosition::ConditionalConsequent,
                    boundary: question,
                    following_mode: colon.previous_mode(),
                },
                chain,
                writer,
                |formatter, tail, enclosed, writer| {
                    formatter.resolve_conditional_boundary(
                        colon,
                        &alternate,
                        tail.pending,
                        default_before_operator,
                        enclosed,
                        writer,
                    )
                },
            )?;
            self.format_deferred_conditional_tail(&colon.deferred_tail, writer)?;
            self.format_conditional_branch(
                ConditionalBranch {
                    operand: alternate,
                    operand_indent,
                    position: ExpressionPosition::ConditionalAlternate,
                    boundary: colon,
                    following_mode: ContentMode::Formatted,
                },
                chain,
                writer,
                |formatter, tail, enclosed, writer| {
                    if enclosed {
                        formatter.format_expression_tail(tail.pending, writer)?;
                        Ok(ExpressionOutput {
                            pending: None,
                            right_edge: tail.right_edge,
                        })
                    } else {
                        Ok(tail)
                    }
                },
            )
        };
        if chain.is_jsx() {
            return write_branches(writer);
        }
        match layout {
            ConditionalLayout::Root | ConditionalLayout::Test => writer.indent(write_branches),
            ConditionalLayout::Consequent if options.indent_style == IndentStyle::Space => {
                let width = usize::from(options.indent_width.value()).saturating_sub(2);
                writer.align(width, write_branches)
            }
            ConditionalLayout::Consequent | ConditionalLayout::Alternate => write_branches(writer),
        }
    }

    fn conditional_boundary(
        &self,
        previous: &ConditionalOperand,
        next: &ConditionalOperand,
        operator: &'static str,
    ) -> Result<ConditionalBoundary<'ast>, FormatError> {
        let span = Span::new(previous.source_end, next.source_start);
        let comments = self.comments.in_range(span);
        let sensitive = self.diagnostic_range_is_sensitive(span);
        if comments.is_empty() && !sensitive {
            return Ok(ConditionalBoundary {
                operator,
                source_range: span,
                comments,
                source: None,
            });
        }
        let Some(operator_start) = self.find_token(span, comments, operator) else {
            return Err(FormatError::unsupported(
                "conditional operator boundary",
                None,
                span,
            ));
        };
        let before_count = comments.partition_point(|comment| comment.span.end <= operator_start);
        let (before, after) = comments.split_at(before_count);
        // Placement around punctuation does not determine the ignored operand:
        // own-line packets lead the next operand, while end-of-line packets
        // after the operator still trail the preceding operand.
        let suppression = (self.content_mode(comments) == ContentMode::Verbatim).then(|| {
            self.conditional_suppression(
                comments,
                before_count,
                previous.operand_end,
                next.source_start,
            )
        });
        let (trailing, leading) = match &suppression {
            Some(suppression) => (suppression.previous, suppression.following),
            None => self.split_separator_comments(
                comments,
                before_count,
                previous.source_end,
                Some(next.source_start),
            ),
        };
        let previous_binding = self.comment_run_binding(before.iter().copied(), |comment| {
            let semantics = self.comment_semantics(comment);
            if semantics.is_ordinary() || semantics.suppression {
                CommentBinding::Unbound
            } else if semantics.type_cast
                && !semantics.diagnostic
                && !semantics.file_directive
                && !semantics.optimizer
            {
                // The following source token is the actual ? or : operator,
                // so this annotation has no source cast-pair target.
                CommentBinding::Unbound
            } else {
                CommentBinding::SourceBound
            }
        });
        let (following_binding, opening_only) = if suppression.is_some() {
            // The source suppression packet already owns its raw operand.
            // Preserve its independent placement rule around punctuation.
            (
                self.comment_run_binding(after.iter().copied(), |comment| {
                    if self.comment_semantics(comment).suppression {
                        CommentBinding::Unbound
                    } else {
                        self.expression_head_binding(std::slice::from_ref(&comment), next.original)
                    }
                }),
                false,
            )
        } else {
            let prepared = self.prepare_expression_source_head(after, next.original);
            let opening_only = sensitive
                && !after.is_empty()
                && self
                    .source_expression_opening_packet(next.original)
                    .is_some_and(|packet| {
                        after.last() == Some(&packet)
                            && !self.diagnostic_range_is_sensitive(Span::new(
                                operator_start,
                                packet.span.start,
                            ))
                    });
            (
                if opening_only {
                    prepared.lexical_binding
                } else {
                    prepared.binding
                },
                opening_only,
            )
        };
        // Only a complete opening-only packet can use lexical placement. Real
        // operator coverage and all other bound heads keep their source boundary.
        let binding = previous_binding.combine(following_binding);
        let binding = if sensitive && !opening_only {
            CommentBinding::SourceBound
        } else {
            binding
        };
        let previous_end = before.last().map_or(span.start, |comment| comment.span.end);
        let next_start = after.first().map_or(span.end, |comment| comment.span.start);
        let previous_mode = self.content_mode(trailing);
        let mut before_operator = self.line_constraint(previous_end, operator_start);
        if previous_mode == ContentMode::Verbatim
            && self
                .ignored_expression_retained_parentheses(
                    previous.original,
                    if operator == "?" {
                        ExpressionPosition::ConditionalTest
                    } else {
                        ExpressionPosition::ConditionalConsequent
                    },
                )
                .is_some_and(|frame| self.line_breaks(previous.operand_end, frame.end) != 0)
            && trailing.last().is_some_and(|comment| {
                comment.span.end <= operator_start
                    && self.line_breaks(comment.span.end, operator_start) == 0
            })
        {
            // A retained wrapper can put its real operand on an earlier line.
            // Keep an inline trailing packet inline even when that frame expands,
            // so it cannot reattach to the following conditional operand.
            before_operator = LineConstraint::SameLine;
        }
        if previous_mode == ContentMode::Verbatim
            && !binding.allows_relocation(None)
            && self.line_breaks(previous.source_end, operator_start) == 0
            && trailing.iter().any(|&comment| {
                comment.span.start > operator_start && self.comment_semantics(comment).suppression
            })
        {
            // A source-bound packet stays after the operator. Keep its trailing
            // ignore attached to the preceding operand on the same physical line.
            before_operator = LineConstraint::SameLine;
        }
        Ok(ConditionalBoundary {
            operator,
            source_range: span,
            comments,
            source: Some(ConditionalSourceBoundary {
                operator_start,
                binding,
                previous_mode,
                next_mode: self.content_mode(leading),
                suppression,
                before_operator,
                after_operator: self.line_constraint(operator_start + 1, next_start),
            }),
        })
    }

    fn conditional_suppression(
        &self,
        comments: &'ast [Comment],
        before_operator: usize,
        operand_end: u32,
        following_start: u32,
    ) -> ConditionalSuppression<'ast> {
        let mut trailing = 0;
        let mut offset = 0;
        for run in comments.chunk_by(|left, right| {
            self.line_breaks(left.span.end, right.span.start) == 0
                && self
                    .source(Span::new(left.span.end, right.span.start))
                    .chars()
                    .all(is_source_whitespace)
        }) {
            let first = run[0];
            let last = run[run.len() - 1];
            let end = offset + run.len();
            let next = comments
                .get(end)
                .map_or(following_start, |comment| comment.span.start);
            let ends_line = self.comment_run_ends_line(last, next);
            if self.source_position_starts_line(first.span.start)
                || ends_line && self.line_breaks(operand_end, first.span.start) != 0
            {
                break;
            }
            if offset < before_operator || ends_line {
                trailing = end;
            }
            if ends_line {
                break;
            }
            offset = end;
        }
        let (previous, following) = comments.split_at(trailing);
        ConditionalSuppression {
            previous,
            following,
        }
    }

    fn split_conditional_suppression_comments(
        &self,
        suppression: &ConditionalSuppression<'ast>,
        comments: &'ast [Comment],
        source_range: Span,
        source_end: u32,
        before_operator: usize,
    ) -> ((&'ast [Comment], &'ast [Comment]), usize) {
        let inner_count =
            comments.partition_point(|comment| comment.span.end <= source_range.start);
        let inner = &comments[..inner_count];
        if !suppression.previous.is_empty()
            || self.diagnostic_range_is_sensitive(Span::new(source_end, source_range.start))
            || inner
                .iter()
                .any(|&comment| !self.comment_semantics(comment).is_ordinary())
        {
            let end = suppression.previous.last().map_or(inner_count, |last| {
                comments.partition_point(|comment| comment.span.end <= last.span.end)
            });
            return (comments.split_at(end), inner_count);
        }
        // Classify the complete emitted run, while the source packet keeps its
        // prepared following-operand target regardless of ordinary placement.
        let (trailing, _) = self.split_separator_comments(
            comments,
            before_operator,
            source_end,
            Some(source_range.end),
        );
        // Ordinary comments outside a retained closing delimiter can still
        // trail that delimiter. Only the following semantic packet fixes the
        // placement frontier; its source suppression target is unchanged.
        let protected = comments[inner_count..]
            .iter()
            .position(|&comment| !self.comment_semantics(comment).is_ordinary())
            .map_or(comments.len(), |offset| inner_count + offset);
        (
            comments.split_at(trailing.len().min(protected)),
            inner_count,
        )
    }

    fn resolve_conditional_boundary(
        &mut self,
        boundary: ConditionalBoundary<'ast>,
        next: &ConditionalOperand,
        tail: Option<ExpressionTail<'ast>>,
        default_before_operator: LineConstraint,
        enclosed: bool,
        writer: &mut Writer<'ast>,
    ) -> Result<ResolvedConditionalBoundary<'ast>, FormatError> {
        let source_end = tail
            .as_ref()
            .map_or(boundary.source_range.start, |tail| tail.source_end);
        let comments = match tail {
            Some(tail) => {
                let comments = self
                    .comments
                    .between(Span::new(source_end, boundary.source_range.end))?;
                debug_assert!(
                    comments.starts_with(tail.comments),
                    "conditional packet must begin with the preceding operand's pending lease",
                );
                comments
            }
            None => boundary.comments,
        };
        let suppression = boundary
            .source
            .as_ref()
            .and_then(|source| source.suppression.as_ref());
        let leading_before_operator = boundary.source.as_ref().is_some_and(|source| {
            !source.binding.allows_relocation(None)
                && source.next_mode == ContentMode::Verbatim
                && suppression.is_some_and(|suppression| {
                    suppression
                        .following
                        .first()
                        .is_some_and(|comment| comment.span.start < source.operator_start)
                })
        });
        let (trailing, leading, next_mode, before_operator, after_operator, inner_count) =
            match boundary.source.as_ref() {
                Some(source) => {
                    let before_count = comments
                        .partition_point(|comment| comment.span.end <= source.operator_start);
                    let ((trailing, leading), inner_count) =
                        if source.binding.allows_relocation(None) {
                            if let Some(suppression) = &source.suppression {
                                let (split, inner_count) = self
                                    .split_conditional_suppression_comments(
                                        suppression,
                                        comments,
                                        boundary.source_range,
                                        source_end,
                                        before_count,
                                    );
                                (split, Some(inner_count))
                            } else {
                                (
                                    self.split_separator_comments(
                                        comments,
                                        before_count,
                                        source_end,
                                        Some(boundary.source_range.end),
                                    ),
                                    None,
                                )
                            }
                        } else {
                            (comments.split_at(before_count), None)
                        };
                    (
                        trailing,
                        leading,
                        source.next_mode,
                        source.before_operator,
                        source.after_operator,
                        inner_count,
                    )
                }
                None if comments.is_empty() => (
                    &[][..],
                    &[][..],
                    ContentMode::Formatted,
                    LineConstraint::Unconstrained,
                    LineConstraint::Unconstrained,
                    None,
                ),
                None => {
                    // An ordinary source-envelope tail can arrive on the direct
                    // path. Locate only this original inter-operand operator.
                    let Some(operator_start) = self.find_token(
                        boundary.source_range,
                        boundary.comments,
                        boundary.operator,
                    ) else {
                        return Err(FormatError::unsupported(
                            "conditional operator boundary",
                            None,
                            boundary.source_range,
                        ));
                    };
                    let before_count =
                        comments.partition_point(|comment| comment.span.end <= operator_start);
                    let (trailing, leading) = self.split_separator_comments(
                        comments,
                        before_count,
                        source_end,
                        Some(boundary.source_range.end),
                    );
                    (
                        trailing,
                        leading,
                        ContentMode::Formatted,
                        LineConstraint::Unconstrained,
                        LineConstraint::Unconstrained,
                        None,
                    )
                }
            };
        // Prepare the child only after its complete emitted head is known.
        // Source coverage stays immutable while the remaining packet determines
        // which foreign comments still belong to this operand's opening scope.
        let content = self.expression_content_with_leading(next.original, leading);
        let next_binding = self.expression_head_binding(leading, next.original);
        let next_binding = content.parentheses.map_or(next_binding, |removed| {
            next_binding.combine(removed.head.binding)
        });
        let annotation = if matches!(next_binding, CommentBinding::Unbound) {
            self.emitted_head_annotation(leading)
        } else {
            None
        };
        let before_operator = if before_operator == LineConstraint::Unconstrained
            && (trailing
                .last()
                .is_some_and(|comment| comment.kind == swc_next_ecma_ast::CommentKind::Line)
                || self.comment_run_has_own_line_tail(trailing, source_end))
        {
            // Finish the preceding operand's packet before emitting punctuation;
            // an inline line suffix must not drift into the following enclosure.
            LineConstraint::NewLine(1)
        } else {
            before_operator
        };
        let before_operator = match before_operator {
            LineConstraint::Unconstrained => default_before_operator,
            constraint => constraint,
        };
        // A head can use the operator's opening only when its gap can break,
        // independently of the following operand's raw target binding.
        let operator_provides_opening = before_operator != LineConstraint::SameLine
            && boundary
                .source
                .as_ref()
                .is_some_and(|source| source.binding.allows_relocation(None));
        let after_operator = if after_operator == LineConstraint::Unconstrained
            && !operator_provides_opening
            && self
                .suppressed_comment_runs(leading, next.source_start)
                .next()
                .is_some_and(|(run, position)| {
                    matches!(position, SuppressionPosition::OwnLine)
                        && run[0].span.start == leading[0].span.start
                }) {
            LineConstraint::NewLine(1)
        } else {
            after_operator
        };
        let (tail, deferred_tail) = if trailing.is_empty() {
            (ConditionalTail::Empty, DeferredConditionalTail::Empty)
        } else if let Some(suppression) = suppression {
            let prefix = inner_count.map_or_else(
                || {
                    trailing
                        .partition_point(|comment| comment.span.end <= boundary.source_range.start)
                },
                |inner_count| inner_count.min(trailing.len()),
            );
            let (inner, outer) = trailing.split_at(prefix);
            let split_following = leading_before_operator && !outer.is_empty();
            let previous_count = if split_following {
                suppression.previous.last().map_or(0, |last| {
                    outer.partition_point(|comment| comment.span.end <= last.span.end)
                })
            } else {
                outer.len()
            };
            let defer_outer = enclosed
                && outer.first().is_some_and(|first| {
                    self.line_breaks(boundary.source_range.start, first.span.start) != 0
                });
            if defer_outer {
                let inside = if inner.is_empty() {
                    ConditionalTail::Empty
                } else {
                    ConditionalTail::Whole {
                        comments: inner,
                        source_end,
                    }
                };
                let outside = if split_following {
                    DeferredConditionalTail::Split {
                        outer,
                        previous_count,
                        boundary_start: boundary.source_range.start,
                    }
                } else {
                    DeferredConditionalTail::Whole {
                        outer,
                        boundary_start: boundary.source_range.start,
                    }
                };
                (inside, outside)
            } else {
                let (previous, following) = outer.split_at(previous_count);
                let gap = (!outer.is_empty()
                    && (!split_following || !previous.is_empty())
                    && inner.last().is_some_and(|comment| {
                        comment.kind == swc_next_ecma_ast::CommentKind::Line
                    }))
                .then(|| {
                    self.prepare_comment_boundary(inner, source_end, boundary.source_range.start)
                        .gap()
                });
                let inside = ConditionalTail::Released {
                    inner,
                    previous,
                    following,
                    source_end,
                    boundary_start: boundary.source_range.start,
                    gap,
                    split_following,
                };
                (inside, DeferredConditionalTail::Empty)
            }
        } else if enclosed {
            let prefix =
                trailing.partition_point(|comment| comment.span.end <= boundary.source_range.start);
            let (inner, outer) = trailing.split_at(prefix);
            let defer_outer = outer.first().is_some_and(|first| {
                self.line_breaks(boundary.source_range.start, first.span.start) != 0
            });
            if defer_outer {
                let inside = if inner.is_empty() {
                    ConditionalTail::Empty
                } else {
                    ConditionalTail::Whole {
                        comments: inner,
                        source_end,
                    }
                };
                let outside = DeferredConditionalTail::Whole {
                    outer,
                    boundary_start: boundary.source_range.start,
                };
                (inside, outside)
            } else {
                (
                    ConditionalTail::Whole {
                        comments: trailing,
                        source_end,
                    },
                    DeferredConditionalTail::Empty,
                )
            }
        } else {
            (
                ConditionalTail::Whole {
                    comments: trailing,
                    source_end,
                },
                DeferredConditionalTail::Empty,
            )
        };
        self.comments.claim(comments)?;
        self.format_conditional_tail(&tail, writer)?;
        Ok(ResolvedConditionalBoundary {
            operator: boundary.operator,
            leading,
            content,
            annotation,
            next_binding,
            next_mode,
            before_operator,
            after_operator,
            deferred_tail,
        })
    }

    fn format_conditional_tail(
        &self,
        tail: &ConditionalTail<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        match tail {
            ConditionalTail::Empty => {}
            ConditionalTail::Whole {
                comments,
                source_end,
            } => {
                self.format_comment_run(
                    comments,
                    CommentRunLayout::SourceAfter(*source_end),
                    writer,
                )?;
            }
            ConditionalTail::Released {
                inner,
                previous,
                following,
                source_end,
                boundary_start,
                gap,
                split_following,
            } => {
                self.format_comment_run(inner, CommentRunLayout::SourceAfter(*source_end), writer)?;
                let layout = if let Some(gap) = gap {
                    gap.write(writer, FlatSeparator::Empty, |_| {});
                    CommentRunLayout::Source
                } else {
                    CommentRunLayout::SourceAfter(*boundary_start)
                };
                self.format_comment_run(previous, layout, writer)?;
                if *split_following {
                    writer.hard_line();
                    self.format_comment_run(following, CommentRunLayout::Source, writer)?;
                }
            }
        }
        Ok(())
    }

    fn format_deferred_conditional_tail(
        &self,
        tail: &DeferredConditionalTail<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        match tail {
            DeferredConditionalTail::Empty => {}
            DeferredConditionalTail::Whole {
                outer,
                boundary_start,
            } => {
                self.format_comment_run(
                    outer,
                    CommentRunLayout::SourceAfter(*boundary_start),
                    writer,
                )?;
            }
            DeferredConditionalTail::Split {
                outer,
                previous_count,
                boundary_start,
            } => {
                let (previous, following) = outer.split_at(*previous_count);
                self.format_comment_run(
                    previous,
                    CommentRunLayout::SourceAfter(*boundary_start),
                    writer,
                )?;
                writer.hard_line();
                self.format_comment_run(following, CommentRunLayout::Source, writer)?;
            }
        }
        Ok(())
    }

    fn format_conditional_branch<T>(
        &mut self,
        branch: ConditionalBranch<'ast>,
        chain: ConditionalChain,
        writer: &mut Writer<'ast>,
        finish: impl FnOnce(
            &mut Self,
            ExpressionOutput<'ast>,
            bool,
            &mut Writer<'ast>,
        ) -> Result<T, FormatError>,
    ) -> Result<T, FormatError> {
        let ConditionalBranch {
            operand,
            operand_indent,
            position,
            boundary,
            following_mode,
        } = branch;
        boundary
            .before_operator
            .write(writer, FlatSeparator::Space, Writer::line);
        writer.text(boundary.operator);
        writer.space();
        let content = boundary.content;
        let ConditionalOperand {
            original,
            source_start,
            operand_end,
            source_end,
        } = operand;
        let mode = if following_mode == ContentMode::Verbatim {
            ContentMode::Verbatim
        } else {
            boundary.next_mode
        };
        let jsx_pair = chain.is_jsx()
            && mode == ContentMode::Formatted
            && matches!(boundary.next_binding, CommentBinding::Unbound)
            && match content.data {
                ExprData::NullLiteral(_) | ExprData::ParenthesizedExpression(_) => false,
                ExprData::IdentifierReference(identifier) => {
                    self.ast.get_utf8(identifier.name(self.ast)) != "undefined"
                }
                ExprData::ConditionalExpression(_) => {
                    !matches!(position, ExpressionPosition::ConditionalAlternate)
                }
                _ => true,
            };
        let pair_lines = jsx_pair.then(|| {
            let first = boundary
                .leading
                .first()
                .map_or(source_start, |comment| comment.span.start);
            let end = content.expression.span(self.ast).end;
            (
                self.line_constraint(first, first),
                self.line_constraint(end, end),
            )
        });
        let nested_consequent = !chain.is_jsx()
            && matches!(position, ExpressionPosition::ConditionalConsequent)
            && matches!(content.data, ExprData::ConditionalExpression(_))
            && mode == ContentMode::Formatted;
        if nested_consequent {
            writer.if_flat("(");
        }
        let indent_style = self.options.indent_style;
        let write = |writer: &mut Writer<'ast>| {
            boundary
                .after_operator
                .write(writer, FlatSeparator::Empty, |_| {});
            let source_head = mode == ContentMode::Verbatim
                || !matches!(boundary.next_binding, CommentBinding::Unbound)
                || boundary.leading.iter().any(|comment| {
                    comment.kind != swc_next_ecma_ast::CommentKind::Block
                        || self.line_breaks(comment.span.start, comment.span.end) != 0
                });
            let head = NodeHead {
                comments: boundary.leading,
                content_start: source_start,
                layout: if source_head {
                    CommentRunLayout::Source
                } else {
                    // This head follows the emitted operator, so source-owned
                    // opening lines become breakable inline comment gaps.
                    CommentRunLayout::InlineLeading(source_start)
                },
                gap_end: source_head.then_some(source_start),
            };
            let operand_indent = if self.is_nullish_conditional_operand(content.data, position) {
                operand_indent
            } else {
                OperandIndent::Parent
            };
            let mut context = ExpressionContext::at(position)
                .with_operand_indent(operand_indent)
                .with_conditional_chain(chain)
                .with_content_mode(mode)
                .with_tail_owner(TailOwner::ParentOperation);
            if following_mode == ContentMode::Verbatim {
                context.ignored_parentheses_closing = Some(Span::new(operand_end, source_end));
            }
            let generated = boundary.annotation.and_then(|annotation| {
                self.emitted_head_enclosure_owner(
                    original,
                    &content,
                    annotation,
                    boundary.next_binding,
                    context,
                )
            });
            let tail = match generated {
                Some(owner) => self
                    .format_prepared_generated_enclosure_with_head(
                        PreparedGeneratedEnclosure { owner, content },
                        head,
                        context,
                        writer,
                    )
                    .map(ExpressionOutput::closed),
                None => self.format_prepared_expression_with_resolved_head_and_raw_end(
                    original,
                    content,
                    ResolvedExpressionHead {
                        head,
                        binding: boundary.next_binding,
                    },
                    context,
                    writer,
                ),
            }?;
            if nested_consequent {
                writer.if_flat(")");
            }
            finish(self, tail, jsx_pair, writer)
        };
        if let Some((opening, closing)) = pair_lines {
            writer.if_break("(");
            let result = writer.indent(|writer| {
                opening.write(writer, FlatSeparator::Empty, Writer::soft_line);
                write(writer)
            })?;
            closing.write(writer, FlatSeparator::Empty, Writer::soft_line);
            writer.if_break(")");
            return Ok(result);
        }
        if chain.is_jsx() {
            return write(writer);
        }
        match indent_style {
            IndentStyle::Space => writer.align(2, write),
            IndentStyle::Tab => writer.indent(write),
        }
    }
}
