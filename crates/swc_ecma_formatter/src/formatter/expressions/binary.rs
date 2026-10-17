//! Typed operator chains with one continuation indent per precedence group.

use swc_next_ecma_ast::{
    Ast, BinaryOperator, Comment, CommentKind, Expr, ExprData, GetSpan, LogicalOperator,
    Precedence, Span,
};

use super::parentheses::{
    ExpressionContent, GeneratedEnclosureOwner, PreparedGeneratedEnclosure, SourceParentheses,
};
use super::{
    ExpressionContext, ExpressionHead, ExpressionPosition, ExpressionStart, Formatter,
    HeadComments, TailOwner,
};
use crate::formatter::comments::{CommentBinding, CommentRunLayout, ExpressionTail, NodeHead};
use crate::formatter::diagnostics::{FlatSeparator, LineConstraint};
use crate::formatter::suppression::{ContentMode, SuppressionPosition};
use crate::{FormatError, document::Writer};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(in crate::formatter) enum BinaryOperatorKind {
    Binary(BinaryOperator),
    Logical(LogicalOperator),
}

#[derive(Clone, Copy)]
pub(in crate::formatter) enum OperandSide {
    Left,
    Right,
}

#[derive(Clone, Copy)]
pub(super) struct BinaryLike {
    pub(super) left: Expr,
    pub(super) right: Expr,
    pub(super) operator: BinaryOperatorKind,
}

impl BinaryLike {
    pub(super) fn from_data(data: ExprData, ast: &Ast<'_>) -> Option<Self> {
        match data {
            ExprData::BinaryExpression(node) => Some(Self {
                left: node.left(ast),
                right: node.right(ast),
                operator: BinaryOperatorKind::Binary(node.operator(ast)),
            }),
            ExprData::LogicalExpression(node) => Some(Self {
                left: node.left(ast),
                right: node.right(ast),
                operator: BinaryOperatorKind::Logical(node.operator(ast)),
            }),
            _ => None,
        }
    }
}

impl BinaryOperatorKind {
    pub(super) fn from_data(data: ExprData, ast: &Ast<'_>) -> Option<Self> {
        match data {
            ExprData::BinaryExpression(node) => Some(Self::Binary(node.operator(ast))),
            ExprData::LogicalExpression(node) => Some(Self::Logical(node.operator(ast))),
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Binary(operator) => operator.as_str(),
            Self::Logical(operator) => operator.as_str(),
        }
    }

    fn precedence(self) -> Precedence {
        match self {
            Self::Binary(operator) => operator.precedence(),
            Self::Logical(operator) => operator.precedence(),
        }
    }

    fn same_kind(self, other: Self) -> bool {
        matches!(
            (self, other),
            (Self::Binary(_), Self::Binary(_)) | (Self::Logical(_), Self::Logical(_))
        )
    }

    pub(super) fn flattens(self, child: Self) -> bool {
        if self.precedence() != child.precedence() {
            return false;
        }
        match self.precedence() {
            Precedence::Exponent | Precedence::Equality | Precedence::Shift => false,
            Precedence::Multiply => self == child && self != Self::Binary(BinaryOperator::Modulo),
            _ => true,
        }
    }

    pub(super) fn needs_parentheses(self, child: Self, side: OperandSide) -> bool {
        if matches!((self, child), (Self::Logical(_), Self::Logical(_))) && self != child {
            return true;
        }
        let parent_precedence = self.precedence();
        let child_precedence = child.precedence();
        if parent_precedence > child_precedence {
            return true;
        }
        if parent_precedence == child_precedence
            && (matches!(side, OperandSide::Right) || !self.flattens(child))
        {
            return true;
        }
        matches!(
            parent_precedence,
            Precedence::Shift
                | Precedence::BitwiseOr
                | Precedence::BitwiseXor
                | Precedence::BitwiseAnd
        ) || (parent_precedence == Precedence::Add && child == Self::Binary(BinaryOperator::Modulo))
    }
}

/// The actual operand owns preparation across comment and operator handoffs.
/// Its emitted scope remains available after the prepared content is consumed.
struct OperandOwner {
    expression: Expr,
    prepared: Option<PreparedOperandContent>,
    scope: OperandScope,
}

struct PreparedOperandContent {
    content: ExpressionContent,
    enclosure: Option<GeneratedEnclosureOwner>,
}

impl OperandOwner {
    fn new(expression: Expr) -> Self {
        Self {
            expression,
            prepared: None,
            scope: OperandScope::Expression,
        }
    }

    fn prepare(&mut self, formatter: &Formatter<'_, '_>) -> &mut PreparedOperandContent {
        let expression = self.expression;
        self.prepared.get_or_insert_with(|| PreparedOperandContent {
            content: formatter.expression_content(expression),
            enclosure: None,
        })
    }
}

struct BinaryChain {
    head: Expr,
    first: ChainParent,
    ancestors: Vec<ChainParent>,
}

/// Only the lexical head state passes from comment preparation to its operand.
#[derive(Clone, Copy)]
struct BinaryHead {
    expression_start: ExpressionStart,
    expression_head: ExpressionHead,
    protected_head_start: Option<u32>,
}

impl BinaryHead {
    fn from_context(context: &ExpressionContext) -> Self {
        Self {
            expression_start: context.expression_start,
            expression_head: context.expression_head,
            protected_head_start: context.protected_head_start,
        }
    }
}

struct PreparedRightStep {
    parentheses: Option<SourceParentheses>,
    next: ChainParent,
}

enum BinaryRightSteps {
    Source,
    Prepared(std::vec::IntoIter<PreparedRightStep>),
}

struct PreparedLogicalTail {
    steps: Vec<PreparedRightStep>,
    operand: PreparedLogicalOperand,
}

struct PreparedLogicalOperand {
    owner: OperandOwner,
    layout: LogicalOperandLayout,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum LogicalOperandLayout {
    Break,
    Inline,
    Jsx,
}

impl LogicalOperandLayout {
    fn inline(self) -> bool {
        self != Self::Break
    }
}

impl PreparedLogicalTail {
    fn inline_root(&self) -> bool {
        self.operand.layout == LogicalOperandLayout::Jsx
            || self.steps.is_empty() && self.operand.layout.inline()
    }
}

struct BinaryRightOperand<'ast> {
    entry: ChainParent,
    parent: Option<BinaryOperatorKind>,
    tail: Option<ExpressionTail<'ast>>,
    closing_end: Option<u32>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum OperandScope {
    Expression,
    Enclosure,
}

#[derive(Clone, Copy)]
struct ChainParent {
    binary: BinaryLike,
    left_content_end: u32,
    left_scope: OperandScope,
    suppression: Option<BinarySuppression>,
}

#[derive(Clone, Copy)]
struct BinarySuppression {
    operator: Span,
    left_mode: ContentMode,
    right_mode: ContentMode,
}

impl ChainParent {
    fn left_mode(self) -> ContentMode {
        self.suppression
            .map_or(ContentMode::Formatted, |source| source.left_mode)
    }

    fn right_mode(self) -> ContentMode {
        self.suppression
            .map_or(ContentMode::Formatted, |source| source.right_mode)
    }
}

#[derive(Clone, Copy)]
enum OperandLayout {
    Break,
    Inline,
    IndentedInline,
}

#[derive(Clone, Copy)]
enum BinaryContinuationLayout {
    Expression,
    ControlCondition,
}

struct BinaryRightContext {
    parent: Option<BinaryOperatorKind>,
    layout: BinaryContinuationLayout,
    tail_owner: TailOwner,
}

enum OperandBoundary<'ast> {
    Break,
    Inline,
    Constrained {
        constraint: LineConstraint,
        inline: bool,
        mode: ContentMode,
    },
    Comments {
        before: &'ast [Comment],
        after: &'ast [Comment],
        operator_start: u32,
        right_start: u32,
        layout: OperandLayout,
        mode: ContentMode,
    },
}

impl OperandBoundary<'_> {
    fn mode(&self) -> ContentMode {
        match self {
            Self::Constrained { mode, .. } | Self::Comments { mode, .. } => *mode,
            Self::Break | Self::Inline => ContentMode::Formatted,
        }
    }

    fn indents(&self) -> bool {
        matches!(
            self,
            Self::Comments {
                layout: OperandLayout::IndentedInline,
                ..
            }
        )
    }
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    fn binary_content(&self, expression: Expr) -> Option<(BinaryLike, Option<SourceParentheses>)> {
        let content = self.expression_content(expression);
        self.binary_content_from_prepared(&content)
    }

    fn binary_content_from_prepared(
        &self,
        content: &ExpressionContent,
    ) -> Option<(BinaryLike, Option<SourceParentheses>)> {
        if content
            .parentheses
            .is_some_and(|removed| !matches!(removed.head.binding, CommentBinding::Unbound))
        {
            return None;
        }
        let binary = BinaryLike::from_data(content.data, self.ast)?;
        Some((binary, content.parentheses.map(|removed| removed.source)))
    }

    fn prepare_binary_entry(
        &self,
        binary: BinaryLike,
        left_content_end: u32,
    ) -> Result<ChainParent, FormatError> {
        let mut entry = ChainParent {
            binary,
            left_content_end,
            left_scope: OperandScope::Expression,
            suppression: None,
        };
        if !self.has_suppression {
            return Ok(entry);
        }
        let right_start = binary.right.span(self.ast).start;
        let span = Span::new(binary.left.span(self.ast).end, right_start);
        let comments = self.comments.in_range(span);
        if self.content_mode(comments) != ContentMode::Verbatim {
            return Ok(entry);
        }
        let operator = binary.operator.as_str();
        let Some(start) = self.find_token(span, comments, operator) else {
            return Err(FormatError::unsupported(
                "binary operator boundary",
                None,
                span,
            ));
        };
        let split = comments.partition_point(|comment| comment.span.end <= start);
        let (before, after) = comments.split_at(split);
        let mut left_mode = ContentMode::Formatted;
        let mut right_mode = ContentMode::Formatted;
        for position in self.suppression_positions(before, start) {
            match position {
                SuppressionPosition::OwnLine => right_mode = ContentMode::Verbatim,
                SuppressionPosition::Inline | SuppressionPosition::EndOfLine => {
                    left_mode = ContentMode::Verbatim;
                }
            }
        }
        for position in self.suppression_positions(after, right_start) {
            match position {
                SuppressionPosition::EndOfLine => left_mode = ContentMode::Verbatim,
                SuppressionPosition::OwnLine | SuppressionPosition::Inline => {
                    right_mode = ContentMode::Verbatim;
                }
            }
        }
        entry.suppression = Some(BinarySuppression {
            operator: Span::new(start, start + operator.len() as u32),
            left_mode,
            right_mode,
        });
        Ok(entry)
    }

    fn binary_chain(&self, first: BinaryLike) -> Result<BinaryChain, FormatError> {
        let head = first.left;
        let mut first = self.prepare_binary_entry(first, first.left.span(self.ast).end)?;
        let mut ancestors = Vec::new();
        while first.left_mode() != ContentMode::Verbatim {
            let Some((left, left_parentheses)) = self.binary_content(first.binary.left) else {
                break;
            };
            if !first.binary.operator.flattens(left.operator) {
                break;
            }
            first.left_content_end = left_parentheses.map_or_else(
                || first.binary.left.span(self.ast).end,
                |boundary| boundary.inner.end,
            );
            ancestors.push(first);
            first = self.prepare_binary_entry(left, left.left.span(self.ast).end)?;
        }
        Ok(BinaryChain {
            head,
            first,
            ancestors,
        })
    }

    fn prepare_binary_right_step(
        &self,
        entry: ChainParent,
    ) -> Result<Option<PreparedRightStep>, FormatError> {
        if !matches!(entry.binary.operator, BinaryOperatorKind::Logical(_))
            || entry.right_mode() == ContentMode::Verbatim
        {
            return Ok(None);
        }
        let Some((right, parentheses)) = self.binary_content(entry.binary.right) else {
            return Ok(None);
        };
        if entry.binary.operator != right.operator {
            return Ok(None);
        }
        let next = self.prepare_binary_entry(right, right.left.span(self.ast).end)?;
        Ok(Some(PreparedRightStep { parentheses, next }))
    }

    fn prepare_binary_logical_tail(
        &self,
        mut entry: ChainParent,
    ) -> Result<Option<PreparedLogicalTail>, FormatError> {
        if !self.ast.is_jsx() || !matches!(entry.binary.operator, BinaryOperatorKind::Logical(_)) {
            return Ok(None);
        }
        let mut steps = Vec::new();
        loop {
            let mut owner = OperandOwner::new(entry.binary.right);
            let layout = if entry.right_mode() == ContentMode::Verbatim {
                LogicalOperandLayout::Break
            } else {
                let content = &owner.prepare(self).content;
                match content.data {
                    ExprData::JsxElement(_) | ExprData::JsxFragment(_) => LogicalOperandLayout::Jsx,
                    ExprData::ArrayExpression(array) if !array.elements(self.ast).is_empty() => {
                        LogicalOperandLayout::Inline
                    }
                    ExprData::ObjectExpression(object)
                        if !object.properties(self.ast).is_empty() =>
                    {
                        LogicalOperandLayout::Inline
                    }
                    _ => {
                        if let Some((right, parentheses)) =
                            self.binary_content_from_prepared(content)
                            && right.operator == entry.binary.operator
                        {
                            let next =
                                self.prepare_binary_entry(right, right.left.span(self.ast).end)?;
                            steps.push(PreparedRightStep { parentheses, next });
                            entry = next;
                            continue;
                        }
                        LogicalOperandLayout::Break
                    }
                }
            };
            return Ok(Some(PreparedLogicalTail {
                steps,
                operand: PreparedLogicalOperand { owner, layout },
            }));
        }
    }

    fn binary_head_start(&self, expression: Expr) -> u32 {
        let content = self.expression_content(expression);
        self.binary_head_start_from_content(expression, &content)
    }

    fn binary_head_start_from_content(
        &self,
        expression: Expr,
        content: &super::parentheses::ExpressionContent,
    ) -> u32 {
        if matches!(content.data, ExprData::ParenthesizedExpression(_))
            || content
                .parentheses
                .is_some_and(|removed| !matches!(removed.head.binding, CommentBinding::Unbound))
        {
            return expression.span(self.ast).start;
        }
        let Some(binary) = BinaryLike::from_data(content.data, self.ast) else {
            return content.expression.span(self.ast).start;
        };
        let mut expression = binary.left;
        loop {
            let content = self.expression_content(expression);
            if matches!(content.data, ExprData::ParenthesizedExpression(_))
                || content
                    .parentheses
                    .is_some_and(|removed| !matches!(removed.head.binding, CommentBinding::Unbound))
            {
                return expression.span(self.ast).start;
            }
            match BinaryLike::from_data(content.data, self.ast) {
                Some(binary) => expression = binary.left,
                None => return content.expression.span(self.ast).start,
            }
        }
    }

    fn format_binary_head_comments(
        &mut self,
        expression: Expr,
        entry: ChainParent,
        owner: &mut OperandOwner,
        mut context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<BinaryHead, FormatError> {
        let start = expression.span(self.ast).start;
        let operand = expression;
        let end = if entry.left_mode() == ContentMode::Verbatim {
            entry.binary.left.span(self.ast).start
        } else if owner.expression == expression {
            let prepared = owner.prepare(self);
            self.binary_head_start_from_content(expression, &prepared.content)
        } else {
            self.binary_head_start(expression)
        };
        let comments = self.comments.between(Span::new(start, end))?;
        let binding = if comments.is_empty() {
            CommentBinding::Unbound
        } else {
            let expression = owner.expression;
            let prepared = owner.prepare(self);
            let binding = self.expression_head_binding(comments, expression);
            prepared
                .content
                .parentheses
                .map_or(binding, |removed| binding.combine(removed.head.binding))
        };
        if self.has_type_casts
            && !comments.is_empty()
            && entry.left_mode() == ContentMode::Formatted
            && context.parentheses_owner == super::ParenthesesOwner::Expression
        {
            let expression = owner.expression;
            let prepared = owner.prepare(self);
            prepared.enclosure = self.generated_enclosure_owner(
                expression,
                &prepared.content,
                &NodeHead {
                    comments,
                    content_start: end,
                    layout: CommentRunLayout::Leading(end),
                    gap_end: None,
                },
                binding,
                ExpressionContext {
                    position: ExpressionPosition::BinaryOperand {
                        operator: entry.binary.operator,
                        side: OperandSide::Left,
                    },
                    content_mode: entry.left_mode(),
                    ..context
                },
            );
            owner.scope = if prepared.enclosure.is_some() {
                OperandScope::Enclosure
            } else {
                OperandScope::Expression
            };
        }
        let comments = self.comments.take(comments.len());
        let start = if self.has_target_annotations
            && context.expression_start.needs_asi_guard()
            && self.options.semicolons == crate::Semicolons::AsNeeded
            && !comments.is_empty()
        {
            self.format_expression_head(
                operand,
                NodeHead {
                    comments,
                    content_start: end,
                    layout: CommentRunLayout::Leading(end),
                    gap_end: None,
                },
                binding,
                context,
                None,
                writer,
            )?
        } else {
            self.format_comment_run(comments, CommentRunLayout::Leading(end), writer)?;
            context.expression_start
        };
        context.expression_start = start;
        Ok(BinaryHead::from_context(&context))
    }

    pub(super) fn format_binary(
        &mut self,
        binary: BinaryLike,
        mut context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        if matches!(context.position, ExpressionPosition::ControlCondition) {
            return self.format_binary_enclosed_with_context(binary, context, writer);
        }
        let mut chain = self.binary_chain(binary)?;
        if context.operand_indent == super::OperandIndent::Parent
            || matches!(
                context.position,
                ExpressionPosition::ParenthesizedUnaryArgument(_)
            )
        {
            return writer.group(false, |writer| {
                self.format_prepared_binary_enclosed(chain, context, None, writer)
            });
        }
        let root = chain.ancestors.first().copied().unwrap_or(chain.first);
        let arrow_body = matches!(context.position, ExpressionPosition::ArrowBody);
        let logical_tail = if !arrow_body {
            self.prepare_binary_logical_tail(root)?
        } else {
            None
        };
        let inline_right = !arrow_body
            && logical_tail.as_ref().map_or_else(
                || self.inline_logical_operand(binary.operator, binary.right),
                PreparedLogicalTail::inline_root,
            );
        let jsx_tail = logical_tail
            .as_ref()
            .is_some_and(|tail| tail.operand.layout == LogicalOperandLayout::Jsx);
        let parent_indents = match context.position {
            ExpressionPosition::AssignmentRight(_)
            | ExpressionPosition::Initializer(super::assignment::AssignmentParent::Statement)
            | ExpressionPosition::PropertyValue => {
                let has_right_prefix = logical_tail.as_ref().is_some_and(|tail| {
                    tail.operand.layout == LogicalOperandLayout::Jsx && !tail.steps.is_empty()
                });
                !inline_right
                    || !has_right_prefix
                        && !BinaryLike::from_data(
                            self.expression_content(binary.left).data,
                            self.ast,
                        )
                        .is_some_and(|left| binary.operator.flattens(left.operator))
            }
            _ => false,
        };
        if parent_indents && !jsx_tail {
            return writer.group(false, |writer| {
                self.format_prepared_binary_enclosed(chain, context, logical_tail, writer)
            });
        }
        let mut owner = OperandOwner::new(chain.first.binary.left);
        let head = if context.head_comments == HeadComments::Pending {
            let head = self.format_binary_head_comments(
                binary.left,
                chain.first,
                &mut owner,
                ExpressionContext {
                    position: ExpressionPosition::BinaryOperand {
                        operator: binary.operator,
                        side: OperandSide::Left,
                    },
                    ..context
                },
                writer,
            )?;
            context.expression_start = head.expression_start;
            context.protected_head_start = head.protected_head_start;
            context.head_comments = HeadComments::Claimed;
            head
        } else {
            BinaryHead::from_context(&context)
        };
        chain.first.left_scope = owner.scope;
        let inline = chain.ancestors.is_empty() && inline_right;
        let logical_tail = match logical_tail {
            Some(tail) if tail.operand.layout == LogicalOperandLayout::Jsx => {
                return self.format_binary_jsx_chain(chain, context, &mut owner, tail, writer);
            }
            tail => tail,
        };
        writer.group(false, |writer| {
            let tail = self.format_binary_head(chain.first, head, &mut owner, writer)?;
            if inline || arrow_body {
                self.format_binary_continuation(chain, context, tail, logical_tail, writer)
            } else {
                writer.indent(|writer| {
                    self.format_binary_continuation(chain, context, tail, logical_tail, writer)
                })
            }
        })
    }

    fn format_binary_jsx_chain(
        &mut self,
        chain: BinaryChain,
        context: ExpressionContext,
        owner: &mut OperandOwner,
        jsx_tail: PreparedLogicalTail,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let head = BinaryHead::from_context(&context);
        writer.group(false, |writer| {
            let PreparedLogicalTail {
                steps,
                operand:
                    PreparedLogicalOperand {
                        owner: mut right_owner,
                        ..
                    },
            } = jsx_tail;
            let mut entry = chain.first;
            if chain.ancestors.is_empty() && steps.is_empty() {
                let tail = self.format_binary_head(entry, head, owner, writer)?;
                let boundary = self.format_binary_operator(
                    entry,
                    &mut right_owner,
                    OperandSide::Right,
                    true,
                    tail,
                    writer,
                )?;
                return self.format_binary_right_operand(
                    &mut right_owner,
                    entry.binary.operator,
                    boundary,
                    context.tail_owner,
                    writer,
                );
            }
            let mut prepared = None;
            let prefix = writer.named_group(false, |writer| {
                prepared = Some((|| {
                    let mut tail = self.format_binary_head(entry, head, owner, writer)?;
                    writer.indent(|writer| {
                        for next in chain.ancestors.into_iter().rev() {
                            tail = self.format_binary_right(
                                entry,
                                BinaryRightContext {
                                    parent: Some(next.binary.operator),
                                    layout: BinaryContinuationLayout::Expression,
                                    tail_owner: TailOwner::ParentOperation,
                                },
                                tail,
                                None,
                                writer,
                            )?;
                            entry = next;
                        }
                        let operand = self.format_binary_right_prefix(
                            entry,
                            None,
                            BinaryContinuationLayout::Expression,
                            tail,
                            BinaryRightSteps::Prepared(steps.into_iter()),
                            writer,
                        )?;
                        let boundary = self.format_binary_operator(
                            operand.entry,
                            &mut right_owner,
                            OperandSide::Right,
                            true,
                            operand.tail,
                            writer,
                        )?;
                        Ok((boundary, operand.closing_end))
                    })
                })());
            });
            let (boundary, closing_end) =
                prepared.expect("named binary prefix executes its operand preparation")?;
            let tail = writer.indent_if_break(prefix, |writer| {
                self.format_binary_right_operand(
                    &mut right_owner,
                    entry.binary.operator,
                    boundary,
                    context.tail_owner,
                    writer,
                )
            })?;
            if let Some(end) = closing_end {
                self.capture_expression_tail(
                    Span::new(right_owner.expression.span(self.ast).end, end),
                    tail,
                )
            } else {
                Ok(tail)
            }
        })
    }

    pub(super) fn format_binary_enclosed(
        &mut self,
        binary: BinaryLike,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_binary_enclosed_with_context(
            binary,
            ExpressionContext::at(ExpressionPosition::Nested),
            writer,
        )
    }

    fn format_binary_enclosed_with_context(
        &mut self,
        binary: BinaryLike,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let chain = self.binary_chain(binary)?;
        self.format_prepared_binary_enclosed(chain, context, None, writer)
    }

    fn format_prepared_binary_enclosed(
        &mut self,
        mut chain: BinaryChain,
        context: ExpressionContext,
        logical_tail: Option<PreparedLogicalTail>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let mut owner = OperandOwner::new(chain.first.binary.left);
        let head = self.format_binary_head_comments(
            chain.head,
            chain.first,
            &mut owner,
            ExpressionContext {
                position: ExpressionPosition::BinaryOperand {
                    operator: chain.first.binary.operator,
                    side: OperandSide::Left,
                },
                ..context
            },
            writer,
        )?;
        chain.first.left_scope = owner.scope;
        let context = ExpressionContext {
            expression_start: head.expression_start,
            protected_head_start: head.protected_head_start,
            ..context
        };
        let tail = self.format_binary_head(chain.first, head, &mut owner, writer)?;
        self.format_binary_continuation(chain, context, tail, logical_tail, writer)
    }

    fn format_binary_head(
        &mut self,
        entry: ChainParent,
        head: BinaryHead,
        owner: &mut OperandOwner,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let first = entry.binary;
        writer.group(false, |writer| {
            let mut context = ExpressionContext::at(ExpressionPosition::BinaryOperand {
                operator: first.operator,
                side: OperandSide::Left,
            })
            .with_expression_start(head.expression_start)
            .with_content_mode(entry.left_mode());
            context.expression_head = head.expression_head;
            context.head_comments = HeadComments::Claimed;
            context.protected_head_start = head.protected_head_start;
            self.format_binary_owned_operand(owner, context, writer)
        })
    }

    fn format_binary_owned_operand(
        &mut self,
        owner: &mut OperandOwner,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        if context.content_mode == ContentMode::Verbatim {
            // Formatted source preparation does not replace the actual ignored owner.
            return self.format_expression(owner.expression, context, writer);
        }
        match owner.prepared.take() {
            Some(PreparedOperandContent {
                content,
                enclosure: Some(enclosure),
            }) => self.format_prepared_generated_enclosure(
                PreparedGeneratedEnclosure {
                    owner: enclosure,
                    content,
                },
                context,
                writer,
            ),
            Some(PreparedOperandContent {
                content,
                enclosure: None,
            }) => self.format_expression_prepared(owner.expression, Some(content), context, writer),
            None => self.format_expression(owner.expression, context, writer),
        }
    }

    fn format_binary_continuation(
        &mut self,
        chain: BinaryChain,
        context: ExpressionContext,
        mut tail: Option<ExpressionTail<'ast>>,
        mut logical_tail: Option<PreparedLogicalTail>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let parent = match context.position {
            ExpressionPosition::BinaryOperand { operator, .. } => Some(operator),
            _ => None,
        };
        let layout = match context.position {
            ExpressionPosition::ControlCondition => BinaryContinuationLayout::ControlCondition,
            _ => BinaryContinuationLayout::Expression,
        };
        let first = chain.first;
        let mut nodes = std::iter::once(first)
            .chain(chain.ancestors.into_iter().rev())
            .peekable();
        while let Some(entry) = nodes.next() {
            let (parent, tail_owner) = match nodes.peek() {
                Some(next) => (Some(next.binary.operator), TailOwner::ParentOperation),
                None => (parent, context.tail_owner),
            };
            let prepared = if nodes.peek().is_none() {
                logical_tail.take()
            } else {
                None
            };
            tail = self.format_binary_right(
                entry,
                BinaryRightContext {
                    parent,
                    layout,
                    tail_owner,
                },
                tail,
                prepared,
                writer,
            )?;
        }
        Ok(tail)
    }

    pub(super) fn inline_logical_operand(
        &self,
        operator: BinaryOperatorKind,
        operand: Expr,
    ) -> bool {
        if !matches!(operator, BinaryOperatorKind::Logical(_)) {
            return false;
        }
        let mut content = self.expression_content(operand);
        match content.data {
            ExprData::ArrayExpression(array) => return !array.elements(self.ast).is_empty(),
            ExprData::ObjectExpression(object) => return !object.properties(self.ast).is_empty(),
            _ => {}
        }
        if !self.ast.is_jsx() {
            return false;
        }
        loop {
            if matches!(
                content.data,
                ExprData::JsxElement(_) | ExprData::JsxFragment(_)
            ) {
                return true;
            }
            let Some((binary, _)) = self.binary_content_from_prepared(&content) else {
                return false;
            };
            if binary.operator != operator {
                return false;
            }
            if self.has_suppression {
                let Ok(entry) = self.prepare_binary_entry(binary, binary.left.span(self.ast).end)
                else {
                    return false;
                };
                if entry.right_mode() == ContentMode::Verbatim {
                    return false;
                }
            }
            content = self.expression_content(binary.right);
        }
    }

    fn format_binary_right(
        &mut self,
        entry: ChainParent,
        context: BinaryRightContext,
        tail: Option<ExpressionTail<'ast>>,
        prepared: Option<PreparedLogicalTail>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let BinaryRightContext {
            parent,
            layout,
            tail_owner,
        } = context;
        let (steps, prepared) = match prepared {
            Some(PreparedLogicalTail { steps, operand }) => {
                (BinaryRightSteps::Prepared(steps.into_iter()), Some(operand))
            }
            None => (BinaryRightSteps::Source, None),
        };
        let operand =
            self.format_binary_right_prefix(entry, parent, layout, tail, steps, writer)?;
        self.format_binary_final_right(operand, layout, tail_owner, prepared, writer)
    }

    fn format_binary_right_prefix(
        &mut self,
        entry: ChainParent,
        mut parent: Option<BinaryOperatorKind>,
        layout: BinaryContinuationLayout,
        mut tail: Option<ExpressionTail<'ast>>,
        mut steps: BinaryRightSteps,
        writer: &mut Writer<'ast>,
    ) -> Result<BinaryRightOperand<'ast>, FormatError> {
        let mut entry = entry;
        let mut closing_end = None;
        // Same-operator logical operands keep their short-circuit order in one group.
        loop {
            let step = match &mut steps {
                BinaryRightSteps::Source => self.prepare_binary_right_step(entry)?,
                BinaryRightSteps::Prepared(steps) => steps.next(),
            };
            let Some(PreparedRightStep {
                parentheses: boundary,
                mut next,
            }) = step
            else {
                break;
            };
            let right = next.binary;
            if let Some(boundary) = boundary {
                closing_end.get_or_insert(boundary.outer.end);
            }
            let mut owner = OperandOwner::new(right.left);
            let boundary = self.format_binary_operator(
                entry,
                &mut owner,
                OperandSide::Left,
                self.inline_logical_operand(entry.binary.operator, right.left),
                tail,
                writer,
            )?;
            next.left_scope = owner.scope;
            let mode = self.format_binary_operand_boundary(boundary, writer)?;
            if owner.scope == OperandScope::Expression
                && next.left_mode() != ContentMode::Verbatim
                && let Some((left, boundary)) =
                    self.binary_content_from_prepared(&owner.prepare(self).content)
                && left.operator == entry.binary.operator
            {
                let mut chain = self.binary_chain(left)?;
                let mut head_owner = OperandOwner::new(chain.first.binary.left);
                let head_context = self.format_binary_head_comments(
                    left.left,
                    chain.first,
                    &mut head_owner,
                    ExpressionContext::at(ExpressionPosition::BinaryOperand {
                        operator: left.operator,
                        side: OperandSide::Left,
                    }),
                    writer,
                )?;
                chain.first.left_scope = head_owner.scope;
                let head =
                    self.format_binary_head(chain.first, head_context, &mut head_owner, writer)?;
                tail = self.format_binary_continuation(
                    chain,
                    ExpressionContext::at(match layout {
                        BinaryContinuationLayout::ControlCondition => {
                            ExpressionPosition::ControlCondition
                        }
                        BinaryContinuationLayout::Expression => ExpressionPosition::BinaryOperand {
                            operator: entry.binary.operator,
                            side: OperandSide::Right,
                        },
                    })
                    .with_tail_owner(TailOwner::ParentOperation),
                    head,
                    None,
                    writer,
                )?;
                next.left_content_end = boundary.map_or_else(
                    || right.left.span(self.ast).end,
                    |boundary| boundary.inner.end,
                );
            } else {
                let mode = if next.left_mode() == ContentMode::Verbatim {
                    next.left_mode()
                } else {
                    mode
                };
                let context = ExpressionContext::at(ExpressionPosition::BinaryOperand {
                    operator: entry.binary.operator,
                    side: OperandSide::Left,
                })
                .with_content_mode(mode)
                .with_tail_owner(TailOwner::ParentOperation);
                tail = self.format_binary_owned_operand(&mut owner, context, writer)?;
            }
            parent = Some(entry.binary.operator);
            entry = next;
        }
        Ok(BinaryRightOperand {
            entry,
            parent,
            tail,
            closing_end,
        })
    }

    fn format_binary_final_right(
        &mut self,
        operand: BinaryRightOperand<'ast>,
        layout: BinaryContinuationLayout,
        tail_owner: TailOwner,
        prepared: Option<PreparedLogicalOperand>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let BinaryRightOperand {
            entry,
            parent,
            tail,
            closing_end,
        } = operand;
        let binary = entry.binary;
        let (mut owner, inline) = match prepared {
            Some(PreparedLogicalOperand { owner, layout }) => (owner, layout.inline()),
            None => (
                OperandOwner::new(binary.right),
                self.inline_logical_operand(binary.operator, binary.right),
            ),
        };
        let same_kind = |expression| {
            let data = self.expression_content(expression).data;
            BinaryOperatorKind::from_data(data, self.ast)
                .is_some_and(|child| child.same_kind(binary.operator))
        };
        let group_right = !matches!(
            (layout, binary.operator),
            (
                BinaryContinuationLayout::ControlCondition,
                BinaryOperatorKind::Logical(_)
            )
        ) && !parent.is_some_and(|parent| parent.same_kind(binary.operator))
            && (entry.left_scope == OperandScope::Enclosure || !same_kind(binary.left))
            && !BinaryOperatorKind::from_data(owner.prepare(self).content.data, self.ast)
                .is_some_and(|child| child.same_kind(binary.operator));
        let write = |writer: &mut Writer<'ast>| {
            let boundary = self.format_binary_operator(
                entry,
                &mut owner,
                OperandSide::Right,
                inline,
                tail,
                writer,
            )?;
            self.format_binary_right_operand(
                &mut owner,
                binary.operator,
                boundary,
                tail_owner,
                writer,
            )
        };
        let tail = if group_right {
            writer.group(false, write)?
        } else {
            write(writer)?
        };
        if let Some(end) = closing_end {
            self.capture_expression_tail(Span::new(binary.right.span(self.ast).end, end), tail)
        } else {
            Ok(tail)
        }
    }

    fn format_binary_right_operand(
        &mut self,
        owner: &mut OperandOwner,
        operator: BinaryOperatorKind,
        boundary: OperandBoundary<'ast>,
        tail_owner: TailOwner,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let indents = boundary.indents();
        let jsx = !matches!(boundary, OperandBoundary::Constrained { .. })
            && boundary.mode() == ContentMode::Formatted
            && matches!(operator, BinaryOperatorKind::Logical(_))
            && matches!(
                owner.prepare(self).content.data,
                ExprData::JsxElement(_) | ExprData::JsxFragment(_)
            );
        let write_argument = |writer: &mut Writer<'ast>| {
            Self::format_binary_operand_separator(&boundary, writer);
            let write = |formatter: &mut Self, writer: &mut Writer<'ast>| {
                let mode = formatter.format_binary_operand_comments(boundary, writer)?;
                let context = ExpressionContext::at(ExpressionPosition::BinaryOperand {
                    operator,
                    side: OperandSide::Right,
                })
                .with_content_mode(mode)
                .with_tail_owner(tail_owner);
                formatter.format_binary_owned_operand(owner, context, writer)
            };
            if jsx {
                return writer.group(false, |writer| {
                    writer.if_break("(");
                    writer.indent(|writer| {
                        writer.soft_line();
                        let tail = write(self, writer)?;
                        self.format_expression_tail(tail, writer)
                    })?;
                    writer.soft_line();
                    writer.if_break(")");
                    Ok(None)
                });
            }
            write(self, writer)
        };
        if indents {
            writer.indent(write_argument)
        } else {
            write_argument(writer)
        }
    }

    fn format_binary_operator(
        &mut self,
        entry: ChainParent,
        owner: &mut OperandOwner,
        side: OperandSide,
        inline: bool,
        tail: Option<ExpressionTail<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<OperandBoundary<'ast>, FormatError> {
        let right = owner.expression;
        let left_end = tail
            .as_ref()
            .map_or(entry.left_content_end, |tail| tail.source_end);
        let right_start = if entry.suppression.is_some() {
            entry.binary.right.span(self.ast).start
        } else {
            let content = &owner.prepare(self).content;
            let source_opening = content.parentheses.is_some_and(|removed| {
                !matches!(removed.head.binding, CommentBinding::Unbound)
                    || !removed.head.has_comments
                        && self.expression_needs_parentheses(
                            content.data,
                            ExpressionPosition::BinaryOperand {
                                operator: entry.binary.operator,
                                side,
                            },
                            ContentMode::Formatted,
                        ) == Some(true)
            });
            if source_opening {
                let outer_start = right.span(self.ast).start;
                let comments = self.ast.comments();
                if comments.is_empty() {
                    outer_start
                } else {
                    let head_end = self.binary_head_start_from_content(right, content);
                    let first =
                        comments.partition_point(|comment| comment.span.start < outer_start);
                    let head = &comments[first..];
                    if head
                        .first()
                        .is_some_and(|comment| comment.span.end <= head_end)
                    {
                        head_end
                    } else {
                        outer_start
                    }
                }
            } else {
                content.expression.span(self.ast).start
            }
        };
        let boundary = Span::new(left_end, right_start);
        let comments = self.comments.between(boundary)?;
        let operator = entry.binary.operator.as_str();
        if entry.suppression.is_some()
            || self.diagnostic_line_is_sensitive(left_end)
            || self.diagnostic_line_is_sensitive(right_start)
            || comments
                .iter()
                .any(|&comment| self.comment_is_diagnostic_directive(comment))
        {
            let operator_span = entry.suppression.map(|source| source.operator).or_else(|| {
                let start = self.find_token(boundary, comments, operator)?;
                Some(Span::new(start, start + operator.len() as u32))
            });
            let Some(operator_span) = operator_span else {
                return Err(FormatError::unsupported(
                    "binary operator boundary",
                    None,
                    boundary,
                ));
            };
            let operator_start = operator_span.start;
            let comments = self.comments.take(comments.len());
            let split = comments.partition_point(|comment| comment.span.end <= operator_start);
            let (before, after) = comments.split_at(split);
            self.format_comment_run(before, CommentRunLayout::SourceAfter(left_end), writer)?;
            let previous = before.last().map_or(left_end, |comment| comment.span.end);
            let constraint = if entry.suppression.is_some() {
                self.source_line_constraint(previous, operator_start)
            } else {
                self.line_constraint(previous, operator_start)
            };
            constraint.write(writer, FlatSeparator::Space, Writer::space);
            writer.text(operator);
            let operator_end = operator_span.end;
            self.format_comment_run(after, CommentRunLayout::SourceAfter(operator_end), writer)?;
            let previous = after
                .last()
                .map_or(operator_end, |comment| comment.span.end);
            return Ok(OperandBoundary::Constrained {
                constraint: if entry.suppression.is_some() {
                    self.source_line_constraint(previous, right_start)
                } else {
                    self.line_constraint(previous, right_start)
                },
                inline,
                mode: entry
                    .suppression
                    .map_or_else(|| self.content_mode(after), |source| source.right_mode),
            });
        }
        if comments.is_empty() {
            writer.space();
            writer.text(operator);
            return Ok(if inline {
                OperandBoundary::Inline
            } else {
                OperandBoundary::Break
            });
        }
        let Some(operator_start) = self.find_token(boundary, comments, operator) else {
            return Err(FormatError::unsupported(
                "binary operator boundary",
                None,
                boundary,
            ));
        };
        let before_count = comments.partition_point(|comment| comment.span.end <= operator_start);
        let after = &comments[before_count..];
        let mode = self.content_mode(after);
        if self.has_type_casts && !after.is_empty() && mode == ContentMode::Formatted {
            let prepared = owner.prepare(self);
            let binding = self.expression_head_binding(after, right);
            let binding = prepared
                .content
                .parentheses
                .map_or(binding, |removed| binding.combine(removed.head.binding));
            prepared.enclosure = self.generated_enclosure_owner(
                right,
                &prepared.content,
                &NodeHead {
                    comments: after,
                    content_start: right_start,
                    layout: CommentRunLayout::Leading(right_start),
                    gap_end: None,
                },
                binding,
                ExpressionContext::at(ExpressionPosition::BinaryOperand {
                    operator: entry.binary.operator,
                    side,
                })
                .with_content_mode(mode),
            );
            owner.scope = if prepared.enclosure.is_some() {
                OperandScope::Enclosure
            } else {
                OperandScope::Expression
            };
        }
        // A real source target or resolved emitted owner keeps this exterior run
        // next to the operand's opener instead of before the operator.
        let trailing_limit = if before_count < comments.len()
            && (owner.scope == OperandScope::Enclosure || self.source_cast_head(right).is_some())
        {
            before_count
        } else {
            comments.len()
        };
        let comments = self.comments.take(comments.len());
        let mut trailing_count = 0;
        let mut previous_end = left_end;
        for (index, comment) in comments[..trailing_limit].iter().enumerate() {
            if self.line_breaks(previous_end, comment.span.start) != 0 {
                break;
            }
            previous_end = comment.span.end;
            if index < before_count
                || comment.kind == CommentKind::Line
                || self.line_breaks(
                    comment.span.end,
                    comments
                        .get(index + 1)
                        .map_or(right_start, |next| next.span.start),
                ) != 0
            {
                trailing_count = index + 1;
            }
        }
        let (trailing, leading) = comments.split_at(trailing_count);
        self.format_comment_run(trailing, CommentRunLayout::SourceAfter(left_end), writer)?;
        writer.space();
        writer.text(operator);
        let leading_before = leading.partition_point(|comment| comment.span.end <= operator_start);
        let (before, after) = leading.split_at(leading_before);
        let layout = if !inline {
            OperandLayout::Break
        } else if self.leading_comments_break(before, operator_start)
            || self.leading_comments_break(after, right_start)
        {
            OperandLayout::IndentedInline
        } else {
            OperandLayout::Inline
        };
        Ok(OperandBoundary::Comments {
            before,
            after,
            operator_start,
            right_start,
            layout,
            mode,
        })
    }

    fn leading_comments_break(&self, comments: &[Comment], next_start: u32) -> bool {
        let mut comments = comments.iter().peekable();
        while let Some(comment) = comments.next() {
            let following = comments.peek().map_or(next_start, |next| next.span.start);
            if self.line_breaks(comment.span.end, following) != 0 {
                return true;
            }
        }
        false
    }

    fn format_binary_operand_boundary(
        &self,
        boundary: OperandBoundary<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<ContentMode, FormatError> {
        Self::format_binary_operand_separator(&boundary, writer);
        self.format_binary_operand_comments(boundary, writer)
    }

    fn format_binary_operand_separator(
        boundary: &OperandBoundary<'ast>,
        writer: &mut Writer<'ast>,
    ) {
        match boundary {
            OperandBoundary::Constrained {
                constraint, inline, ..
            } => {
                constraint.write(writer, FlatSeparator::Space, |writer| {
                    if *inline {
                        writer.space();
                    } else {
                        writer.line();
                    }
                });
            }
            OperandBoundary::Inline => writer.space(),
            OperandBoundary::Break => writer.line(),
            OperandBoundary::Comments { layout, .. } => match layout {
                OperandLayout::Inline => writer.space(),
                OperandLayout::Break | OperandLayout::IndentedInline => writer.line(),
            },
        }
    }

    fn format_binary_operand_comments(
        &self,
        boundary: OperandBoundary<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<ContentMode, FormatError> {
        let OperandBoundary::Comments {
            before,
            after,
            operator_start,
            right_start,
            mode,
            ..
        } = boundary
        else {
            return Ok(boundary.mode());
        };
        self.format_comment_run(before, CommentRunLayout::Source, writer)?;
        if let Some(last) = before.last() {
            self.comment_gap(last.span.end, operator_start, writer);
        }
        if self.parenthesis_has_type_cast(right_start) {
            self.format_comment_run(after, CommentRunLayout::Leading(right_start), writer)?;
        } else {
            self.format_comment_run(after, CommentRunLayout::Source, writer)?;
            if let Some(last) = after.last() {
                self.comment_gap(last.span.end, right_start, writer);
            }
        }
        Ok(mode)
    }
}
