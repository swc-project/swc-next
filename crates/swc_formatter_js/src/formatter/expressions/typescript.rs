//! TypeScript expression operators keep real operand owners apart from token gaps.

use swc_next_ecma_ast::{
    Ast, BinaryOperator, Comment, CommentKind, Expr, ExprData, GetSpan, Span, TsAsExpression,
    TsInstantiationExpression, TsNonNullExpression, TsType, TsTypeAssertion, TsTypeData,
    TsTypeNameData, TsTypeParameterInstantiation,
};

use super::binary::{BinaryLike, BinaryOperatorKind, OperandSide};
use super::chains::PreparedChain;
use super::parentheses::{ExpressionContent, ResolvedExpressionHead};
use super::{
    AccessKind, ExpressionContext, ExpressionOutput, ExpressionPosition, ParenthesesOwner,
    TailOwner,
};
use crate::formatter::Formatter;
use crate::formatter::comments::{
    CommentBinding, CommentBoundary, CommentRunLayout, ExpressionTail, NodeHead,
};
use crate::formatter::diagnostics::{FlatSeparator, LineConstraint};
use crate::formatter::source::{SourceKeywordSpacing, is_source_whitespace};
use crate::formatter::suppression::{ContentMode, SuppressionPosition};
use crate::formatter::types::generics::TypeAnglePacket;
use crate::formatter::types::{
    BinaryTypeHead, PreparedDelimitedType, PreparedDelimitedTypeHead, PreparedPrecedingTypePacket,
    PreparedType, RecordedTypeHead, SourceTypeOperator, SuppressedTypeHead, SuppressedTypeHeadRole,
    TypeArgumentOwner, TypeContext, TypeLayout, TypeOperatorSlot,
};
use crate::{
    FormatError,
    document::{GroupRequest, Writer, WriterExt},
};

#[derive(Clone, Copy)]
pub(super) enum TypeBinaryOperator {
    As,
    Satisfies,
}

impl TypeBinaryOperator {
    fn token(self) -> &'static str {
        match self {
            Self::As => "as",
            Self::Satisfies => "satisfies",
        }
    }
}

pub(in crate::formatter) struct ExpressionTypeArguments<'ast> {
    pub(super) node: TsTypeParameterInstantiation,
    pub(in crate::formatter) callee_mode: ContentMode,
    pub(super) mode: ContentMode,
    pub(super) access: AccessKind,
    has_comments: bool,
    owner: TypeArgumentOwner,
    prefix: ExpressionTypeArgumentPrefix<'ast>,
}

impl ExpressionTypeArguments<'_> {
    pub(in crate::formatter) fn node(&self) -> TsTypeParameterInstantiation {
        self.node
    }

    pub(in crate::formatter) fn suppress(&mut self) {
        self.mode = ContentMode::Verbatim;
    }

    pub(in crate::formatter) fn has_comments(&self) -> bool {
        self.has_comments
    }
}

enum ExpressionTypeArgumentPrefix<'ast> {
    Source(Span),
    Formatted {
        before: CommentBoundary<'ast>,
        optional: Option<OptionalTypeArgumentPrefix<'ast>>,
    },
}

struct OptionalTypeArgumentPrefix<'ast> {
    token: Span,
    after: CommentBoundary<'ast>,
}

enum TypeExpressionGap<'ast> {
    Source(Span),
    Formatted(CommentBoundary<'ast>),
}

struct TypeAssertionClosing<'ast> {
    closing: u32,
    type_tail: &'ast [Comment],
    operand_head: &'ast [Comment],
}

struct TypeAssertionHead<'ast> {
    body: PreparedDelimitedType<'ast>,
    before: TypeAssertionOpening<'ast>,
    closing: TypeExpressionGap<'ast>,
}

enum TypeAssertionOpening<'ast> {
    Boundary(TypeExpressionGap<'ast>),
    Packet(TypeAnglePacket<'ast>),
}

/// The angle owner consumes the selected boundary and type as one opening.
enum PreparedAssertionOpening<'ast> {
    Source(Span, PreparedDelimitedType<'ast>),
    Packet(TypeAnglePacket<'ast>, PreparedDelimitedType<'ast>),
    Boundary(CommentBoundary<'ast>, PreparedDelimitedType<'ast>),
    Recorded(RecordedTypeHead<'ast>),
}

enum AssertionOperandLayout {
    Direct,
    Enclosable,
}

/// A protected token gap stays source-owned; a relocatable head is prepared
/// through every removed source envelope before either layout candidate prints it.
enum TypeAssertionOperandHead<'ast> {
    Boundary {
        before: TypeExpressionGap<'ast>,
        content: Option<ExpressionContent>,
    },
    Prepared {
        head: ResolvedExpressionHead<'ast>,
        content: TypeAssertionContent<'ast>,
    },
}

enum TypeAssertionContent<'ast> {
    Expression(ExpressionContent),
    Chain {
        chain: Box<PreparedChain<'ast>>,
        expression: Expr,
    },
}

impl TypeAssertionContent<'_> {
    fn expression(&self) -> Expr {
        match self {
            Self::Expression(content) => content.expression,
            Self::Chain { expression, .. } => *expression,
        }
    }
}

struct TypeAssertionOperand<'ast> {
    expression: Expr,
    context: ExpressionContext<'ast>,
    head: TypeAssertionOperandHead<'ast>,
    layout: AssertionOperandLayout,
}

impl TypeAssertionOperandHead<'_> {
    fn has_leading_separator(&self) -> bool {
        matches!(self, Self::Prepared { head, .. }
            if head.head.comments.first().is_some_and(|comment| comment.kind == CommentKind::Line))
    }
}

struct PreparedTypeAssertion<'ast> {
    head: TypeAssertionHead<'ast>,
    operand: TypeAssertionOperand<'ast>,
    grammar_barrier: bool,
}

/// One assertion's source-owned operand and prepared token boundary.
pub(super) struct SourceNonNull<'ast> {
    pub(super) expression: Expr,
    mode: ContentMode,
    prefix: TypeExpressionGap<'ast>,
    has_comments: bool,
}

impl<'ast> SourceNonNull<'ast> {
    pub(super) fn operand_context(
        &self,
        context: ExpressionContext<'ast>,
    ) -> ExpressionContext<'ast> {
        context
            .type_operand(
                ExpressionPosition::NonNullOperand,
                self.mode,
                TailOwner::ParentOperation,
            )
            .with_member_layout(context.member_layout)
            .with_jsx_callback(if self.source_break() {
                super::JsxCallbackContext::Ordinary
            } else {
                context.jsx_callback
            })
            .with_conditional_envelope(context.conditional_envelope.chain_head())
    }

    pub(in crate::formatter) fn has_comments(&self) -> bool {
        self.has_comments
    }

    pub(super) fn source_break(&self) -> bool {
        self.has_comments
            || matches!(&self.prefix, TypeExpressionGap::Formatted(boundary) if boundary.gap().requires_line())
    }
}

struct BinaryTypeBoundary<'ast> {
    operator: TypeBinaryOperator,
    left_mode: ContentMode,
    layout: BinaryTypeLayout<'ast>,
}

struct PreparedBinaryTypeExpression<'ast> {
    boundary: BinaryTypeBoundary<'ast>,
    annotation: PreparedType,
    type_context: TypeContext,
    tail_owner: TailOwner,
}

impl PreparedBinaryTypeExpression<'_> {
    fn operand_mode(&self) -> ContentMode {
        self.boundary.left_mode
    }
}

enum BinaryTypeLayout<'ast> {
    Source {
        source: SourceTypeOperator,
        spacing: SourceKeywordSpacing,
    },
    SourceRhs {
        prefix: Span,
        spacing: SourceKeywordSpacing,
        after: CommentBoundary<'ast>,
        head: BinaryTypeHead<'ast>,
    },
    Preceding {
        packet: PreparedPrecedingTypePacket<'ast>,
        after: CommentBoundary<'ast>,
        head: BinaryTypeHead<'ast>,
    },
    Suppressed {
        before: CommentBoundary<'ast>,
        head: SuppressedTypeHead<'ast>,
    },
    ConstMarker {
        before: CommentBoundary<'ast>,
        after: CommentBoundary<'ast>,
        head: BinaryTypeHead<'ast>,
        tail: Option<ExpressionTail<'ast>>,
    },
    Formatted {
        before: CommentBoundary<'ast>,
        after: CommentBoundary<'ast>,
        head: BinaryTypeHead<'ast>,
    },
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    /// Postfix chain wrappers retain their source owners while sharing the operand's shape.
    pub(super) fn chain_element_data(&self, mut data: ExprData) -> ExprData {
        loop {
            let expression = match data {
                ExprData::ChainExpression(chain) => chain.expression(self.ast),
                ExprData::TsNonNullExpression(assertion) => assertion.expression(self.ast),
                _ => return data,
            };
            data = self.expression_data(expression);
        }
    }

    /// Collection arguments can keep their call delimiters around a type wrapper.
    pub(super) fn call_argument_is_collection(&self, mut expression: Expr) -> bool {
        loop {
            let content = self.expression_content(expression);
            expression = match content.data {
                ExprData::ArrayExpression(_) | ExprData::ObjectExpression(_) => return true,
                ExprData::TsAsExpression(assertion) => assertion.expression(self.ast),
                ExprData::TsSatisfiesExpression(assertion) => assertion.expression(self.ast),
                ExprData::TsTypeAssertion(assertion) => assertion.expression(self.ast),
                _ => return false,
            };
        }
    }

    /// A short cast neighbor has an atomic type after bounded array/reference peeling.
    pub(super) fn short_argument_type(&self, mut annotation: TsType) -> bool {
        let ast = self.ast;
        for _ in 0..2 {
            let TsTypeData::TsArrayType(array) = ast.ts_type_data(annotation) else {
                break;
            };
            annotation = array.element_type(ast);
        }
        if let TsTypeData::TsTypeReference(reference) = ast.ts_type_data(annotation)
            && let Some(arguments) = reference.type_arguments(ast)
        {
            let parameters = arguments.params(ast);
            if parameters.len() == 1
                && let Some(parameter) = parameters.first()
            {
                annotation = ast.get_node_in_sub_range(parameter);
            }
        }
        match ast.ts_type_data(annotation) {
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
            | TsTypeData::TsLiteralType(_)
            | TsTypeData::TsTemplateLiteralType(_) => true,
            TsTypeData::TsTypeReference(reference) => reference.type_arguments(ast).is_none(),
            _ => false,
        }
    }

    pub(in crate::formatter) fn prepare_heritage_type_arguments(
        &self,
        expression: Expr,
        node: TsTypeParameterInstantiation,
    ) -> Result<ExpressionTypeArguments<'ast>, FormatError> {
        let mut arguments =
            self.prepare_expression_type_arguments(expression, node, AccessKind::Ordinary)?;
        arguments.owner = TypeArgumentOwner::Heritage;
        Ok(arguments)
    }

    pub(in crate::formatter) fn prepare_expression_type_arguments(
        &self,
        expression: Expr,
        node: TsTypeParameterInstantiation,
        access: AccessKind,
    ) -> Result<ExpressionTypeArguments<'ast>, FormatError> {
        let gap = Span::new(expression.span(self.ast).end, node.span(self.ast).start);
        let comments = self.comments.in_range(gap);
        let optional = if access == AccessKind::Optional {
            let Some(start) = self.find_token(gap, comments, "?.") else {
                return Err(FormatError::unsupported(
                    "optional type arguments token",
                    None,
                    gap,
                ));
            };
            Some(Span::new(start, start + 2))
        } else {
            None
        };
        let split = optional.map_or(0, |token| {
            comments.partition_point(|comment| comment.span.end <= token.start)
        });
        let (before, after) = comments.split_at(split);
        let mut callee_mode = ContentMode::Formatted;
        let mut mode = ContentMode::Formatted;
        if self.has_suppression && self.content_mode(comments) == ContentMode::Verbatim {
            for (_, position) in self.suppressed_comment_runs(comments, gap.end) {
                if matches!(position, SuppressionPosition::EndOfLine) {
                    callee_mode = ContentMode::Verbatim;
                } else {
                    mode = ContentMode::Verbatim;
                }
            }
        }
        let prefix = if callee_mode == ContentMode::Verbatim || mode == ContentMode::Verbatim {
            ExpressionTypeArgumentPrefix::Source(gap)
        } else if let Some(token) = optional {
            ExpressionTypeArgumentPrefix::Formatted {
                before: self.prepare_comment_boundary(before, gap.start, token.start),
                optional: Some(OptionalTypeArgumentPrefix {
                    token,
                    after: self.prepare_comment_boundary(after, token.end, gap.end),
                }),
            }
        } else {
            ExpressionTypeArgumentPrefix::Formatted {
                before: self.prepare_comment_boundary(comments, gap.start, gap.end),
                optional: None,
            }
        };
        Ok(ExpressionTypeArguments {
            node,
            callee_mode,
            mode,
            access,
            has_comments: !comments.is_empty(),
            owner: TypeArgumentOwner::Ordinary,
            prefix,
        })
    }

    pub(in crate::formatter) fn format_expression_type_arguments(
        &mut self,
        arguments: ExpressionTypeArguments<'ast>,
        tail: Option<ExpressionTail<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        match arguments.prefix {
            ExpressionTypeArgumentPrefix::Source(gap) => {
                let line_tail = tail.as_ref().is_some_and(|tail| {
                    tail.comments
                        .last()
                        .is_some_and(|comment| comment.kind == CommentKind::Line)
                });
                let needs_line = line_tail
                    && !self
                        .source(gap)
                        .chars()
                        .take_while(|&character| is_source_whitespace(character))
                        .any(|character| matches!(character, '\r' | '\n'));
                self.format_expression_tail(tail, writer)?;
                if needs_line {
                    // The removed callee pair supplied the line before this
                    // protected gap. Only CR/LF in literal text flushes a suffix.
                    writer.hard_line();
                }
                self.format_verbatim(gap, writer)?;
            }
            ExpressionTypeArgumentPrefix::Formatted { before, optional } => {
                let before = if let Some(tail) = tail {
                    // The real angle suffix owns one source-ordered packet,
                    // including comments exposed by removed callee pairs.
                    let end = optional
                        .as_ref()
                        .map_or(arguments.node.span(self.ast).start, |optional| {
                            optional.token.start
                        });
                    let comments = self.comments.between(Span::new(tail.source_end, end))?;
                    self.prepare_comment_boundary(comments, tail.source_end, end)
                } else {
                    before
                };
                if optional.is_none()
                    && !before.is_empty()
                    && matches!(
                        self.comment_boundary_binding(&before, arguments.node.span(self.ast)),
                        CommentBinding::Unbound
                    )
                {
                    // Keep one source-ordered packet. Its first comment selects
                    // a callee line tail or an angle-list block head.
                    if before.comments()[0].kind == CommentKind::Line {
                        writer.space();
                    }
                    self.format_comment_boundary_with_layout(
                        &before,
                        CommentRunLayout::FlatLeading(arguments.node.span(self.ast).start),
                        writer,
                    )?;
                } else {
                    self.format_type_expression_gap(
                        TypeExpressionGap::Formatted(before),
                        FlatSeparator::Empty,
                        writer,
                    )?;
                }
                if let Some(optional) = optional {
                    writer.text("?.");
                    self.format_type_expression_gap(
                        TypeExpressionGap::Formatted(optional.after),
                        FlatSeparator::Empty,
                        writer,
                    )?;
                }
            }
        }
        self.format_type_arguments_with_owner(
            arguments.node,
            arguments.mode,
            arguments.owner,
            writer,
        )
    }

    pub(super) fn format_instantiation(
        &mut self,
        node: TsInstantiationExpression,
        context: ExpressionContext<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let expression = node.expression(self.ast);
        let arguments = self.prepare_expression_type_arguments(
            expression,
            node.type_arguments(self.ast),
            AccessKind::Ordinary,
        )?;
        let tail = self.format_expression(
            expression,
            context.type_operand(
                ExpressionPosition::InstantiationOperand,
                arguments.callee_mode,
                TailOwner::ParentOperation,
            ),
            writer,
        )?;
        self.format_expression_type_arguments(arguments, tail, writer)?;
        Ok(None)
    }

    #[inline]
    pub(super) fn ts_binary_operand_needs_parentheses(
        &self,
        data: ExprData,
        operator: BinaryOperatorKind,
        side: OperandSide,
        mode: ContentMode,
    ) -> bool {
        self.ast.is_ts() && self.instantiation_operand_needs_parentheses(data, operator, side, mode)
    }

    fn instantiation_operand_needs_parentheses(
        &self,
        data: ExprData,
        operator: BinaryOperatorKind,
        side: OperandSide,
        mode: ContentMode,
    ) -> bool {
        let BinaryOperatorKind::Binary(operator) = operator else {
            return false;
        };
        if side == OperandSide::Right && operator == BinaryOperator::LeftShift {
            // TypeScript can read `x << (f<T>)` as the start of a type argument list.
            return false;
        }
        // Group the operand that owns the type arguments before flattening its parent chain.
        if mode == ContentMode::Formatted && self.is_instantiation_operand(data) {
            let bare_rhs = side == OperandSide::Right
                && matches!(operator, BinaryOperator::In | BinaryOperator::Instanceof)
                && matches!(data, ExprData::TsInstantiationExpression(_));
            return !bare_rhs;
        }
        side == OperandSide::Left
            && instantiation_tail_conflicts(operator)
            && self.binary_left_exposes_type_arguments(data, operator, mode)
    }

    fn binary_left_exposes_type_arguments(
        &self,
        mut data: ExprData,
        operator: BinaryOperator,
        mut mode: ContentMode,
    ) -> bool {
        // A line break after `>` can make TypeScript read `a < b >` as `a<b>`.
        if operator == BinaryOperator::GreaterThan
            && matches!(data, ExprData::BinaryExpression(left)
                if left.operator(self.ast) == BinaryOperator::LessThan)
        {
            return true;
        }
        // Follow the emitted right edge. Once a subtree is copied, only its
        // source pairs can protect the generic suffix from the next operator.
        loop {
            let check_suppression = mode == ContentMode::Formatted && self.has_suppression;
            let mut child_mode = mode;
            let (operand, position) = match data {
                ExprData::TsInstantiationExpression(_) => return true,
                ExprData::UnaryExpression(node) => {
                    if mode == ContentMode::Formatted
                        && !self
                            .source_comments_in_range(node.span(self.ast))
                            .is_empty()
                    {
                        let Some(operand_mode) = self.unary_tail_operand_mode(node) else {
                            return false;
                        };
                        child_mode = operand_mode;
                    }
                    (
                        node.argument(self.ast),
                        ExpressionPosition::UnaryArgument(node.operator(self.ast)),
                    )
                }
                ExprData::AwaitExpression(node) => {
                    let argument = node.argument(self.ast);
                    if check_suppression {
                        let gap =
                            Span::new(node.span(self.ast).start + 5, argument.span(self.ast).start);
                        child_mode = self.source_content_mode(gap);
                    }
                    (argument, ExpressionPosition::AwaitArgument)
                }
                ExprData::TsTypeAssertion(node) => {
                    if check_suppression {
                        let Ok(closing) = self.type_assertion_closing(node) else {
                            return true;
                        };
                        child_mode = self.content_mode(closing.operand_head);
                    }
                    (
                        node.expression(self.ast),
                        ExpressionPosition::TypeAssertionOperand,
                    )
                }
                ExprData::ChainExpression(node) => {
                    (node.expression(self.ast), ExpressionPosition::Nested)
                }
                _ => {
                    let Some(binary) = BinaryLike::from_data(data, self.ast) else {
                        return false;
                    };
                    if check_suppression {
                        let Ok(operand_mode) = self.binary_right_mode(binary) else {
                            return true;
                        };
                        child_mode = operand_mode;
                    }
                    (
                        binary.right,
                        ExpressionPosition::BinaryOperand {
                            operator: binary.operator,
                            side: OperandSide::Right,
                        },
                    )
                }
            };
            if mode == ContentMode::Verbatim {
                data = self.ast.expr_data(operand);
                continue;
            }
            let child = match child_mode {
                ContentMode::Formatted => self.expression_data(operand),
                ContentMode::Verbatim => self.ast.expr_data(operand),
            };
            let content = self.source_parentheses_head_content(
                child,
                ExpressionContext::at(position).with_content_mode(child_mode),
            );
            data = content.data;
            mode = content.mode;
            if self.expression_needs_parentheses(data, position, mode) != Some(false) {
                return false;
            }
        }
    }

    fn is_instantiation_operand(&self, mut data: ExprData) -> bool {
        loop {
            let operand = match data {
                ExprData::TsInstantiationExpression(_) => return true,
                ExprData::UnaryExpression(node) => node.argument(self.ast),
                ExprData::AwaitExpression(node) => node.argument(self.ast),
                ExprData::YieldExpression(node) => {
                    let Some(argument) = node.argument(self.ast) else {
                        return false;
                    };
                    argument
                }
                ExprData::TsTypeAssertion(node) => node.expression(self.ast),
                _ => return false,
            };
            data = self.ast.expr_data(operand);
            while let ExprData::ParenthesizedExpression(pair) = data {
                data = self.ast.expr_data(pair.expression(self.ast));
            }
        }
    }

    pub(super) fn shift_suspension_needs_grammar_barrier(
        &self,
        data: ExprData,
        position: ExpressionPosition,
    ) -> bool {
        // The ordinary suspension pair alone lets TypeScript misread `x << (yield f<T>)`.
        is_left_shift_right_operand(position)
            && matches!(
                data,
                ExprData::AwaitExpression(_) | ExprData::YieldExpression(_)
            )
            && self.ast.is_ts()
            && self.is_instantiation_operand(data)
    }

    pub(super) fn format_binary_type_expression_output(
        &mut self,
        expression: Expr,
        annotation: TsType,
        operator: TypeBinaryOperator,
        context: ExpressionContext<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<ExpressionOutput<'ast>, FormatError> {
        let source =
            self.prepare_binary_type_expression(expression, annotation, operator, &context)?;
        writer.group(false, |writer| {
            let receiver = (context.is_call_or_member_receiver()
                || matches!(context.position, ExpressionPosition::NewCallee { .. }))
                && context.parentheses_owner == ParenthesesOwner::Expression;
            let closing = if receiver {
                self.expression_closing_constraint(annotation.span(self.ast).end)
            } else {
                LineConstraint::Unconstrained
            };
            let write = |writer: &mut Writer<'ast>| {
                let tail = self.format_expression(
                    expression,
                    context
                        .type_operand(
                            ExpressionPosition::TypeBinaryOperand,
                            source.operand_mode(),
                            TailOwner::ParentOperation,
                        )
                        .with_conditional_envelope(context.conditional_envelope.terminal_head()),
                    writer,
                )?;
                self.format_binary_type_expression_suffix(source, tail, writer)
            };
            if receiver {
                let tail = writer.indent(|writer| {
                    writer.soft_line();
                    write(writer)
                })?;
                closing.write(writer, FlatSeparator::Empty, Writer::soft_line);
                Ok(tail)
            } else {
                write(writer)
            }
        })
    }

    fn prepare_binary_type_expression(
        &self,
        expression: Expr,
        annotation: TsType,
        operator: TypeBinaryOperator,
        context: &ExpressionContext<'ast>,
    ) -> Result<PreparedBinaryTypeExpression<'ast>, FormatError> {
        let mut annotation = self.prepare_type(annotation, ContentMode::Formatted);
        let tail_owner = context.tail_owner;
        let boundary = self.prepare_binary_type_boundary(expression, &mut annotation, operator)?;
        Ok(PreparedBinaryTypeExpression {
            boundary,
            annotation,
            type_context: TypeContext::TypeExpressionRhs {
                before_keyword: matches!(context.position, ExpressionPosition::TypeBinaryOperand),
            },
            tail_owner,
        })
    }

    fn format_binary_type_expression_suffix(
        &mut self,
        source: PreparedBinaryTypeExpression<'ast>,
        tail: Option<ExpressionTail<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<ExpressionOutput<'ast>, FormatError> {
        self.format_expression_tail(tail, writer)?;
        self.format_binary_type_right_output(
            source.boundary,
            source.annotation,
            source.type_context,
            source.tail_owner == TailOwner::ParentOperation,
            writer,
        )
    }

    fn prepare_binary_type_boundary(
        &self,
        expression: Expr,
        right: &mut PreparedType,
        operator: TypeBinaryOperator,
    ) -> Result<BinaryTypeBoundary<'ast>, FormatError> {
        let gap = Span::new(
            expression.span(self.ast).end,
            right.node.span(self.ast).start,
        );
        let comments = self.comments.in_range(gap);
        let token = operator.token();
        let Some(start) = self.find_token(gap, comments, token) else {
            return Err(FormatError::unsupported(
                "type expression operator",
                None,
                gap,
            ));
        };
        let operator_span = Span::new(start, start + token.len() as u32);
        let split = comments.partition_point(|comment| comment.span.end <= start);
        let (before, after) = comments.split_at(split);
        let mut left_mode = ContentMode::Formatted;
        let mut source = self.has_source_only_comment_gap(operator_span.end, after);
        let mut suppressed_head = None;
        let mut preceding_packet = None;
        if self.has_suppression && self.content_mode(comments) == ContentMode::Verbatim {
            let const_marker =
                matches!(operator, TypeBinaryOperator::As) && self.is_const_marker_type(*right);
            let packet_owner = (!const_marker
                && matches!(
                    self.ast.expr_data(expression),
                    ExprData::IdentifierReference(_)
                ))
            .then(|| expression.span(self.ast));
            let attachment = self.type_attachment_head(right.node);
            let before_head = self.prepare_type_suppression_head(&attachment, before);
            for (run, position) in self.suppressed_comment_runs(before, start) {
                let selection = self.prepare_type_suppression_run(
                    &attachment,
                    before_head,
                    run,
                    position,
                    super::super::types::TypeSuppressionContext::ExpressionBoundary,
                );
                selection.retain_source_head(right, self.ast);
                if !matches!(position, SuppressionPosition::OwnLine)
                    && let Some(owner) = packet_owner
                    && let Some(packet) = self.prepare_preceding_type_operator(
                        comments,
                        run,
                        owner,
                        right,
                        &selection,
                        TypeOperatorSlot::new(operator_span, LineConstraint::Unconstrained),
                    )
                {
                    left_mode = ContentMode::Verbatim;
                    preceding_packet = Some(packet);
                    continue;
                }
                source = true;
                if matches!(position, SuppressionPosition::OwnLine) {
                    self.select_type_head_with_attachment(right, &attachment, position);
                } else {
                    left_mode = ContentMode::Verbatim;
                }
            }
            let after_head = self.prepare_type_suppression_head(&attachment, after);
            for (run, position) in self.suppressed_comment_runs(after, gap.end) {
                let resolved = self.resolve_type_suppression_run(
                    &attachment,
                    after_head,
                    run,
                    position,
                    super::super::types::TypeSuppressionContext::ExpressionBoundary,
                );
                let selection = &resolved.selection;
                let selected = selection.target();
                selection.retain_source_head(right, self.ast);
                if let Some(owner) = packet_owner
                    && let Some(packet) = self.prepare_preceding_type_operator(
                        comments,
                        run,
                        owner,
                        right,
                        selection,
                        TypeOperatorSlot::new(operator_span, LineConstraint::Unconstrained),
                    )
                {
                    left_mode = ContentMode::Verbatim;
                    preceding_packet = Some(packet);
                    continue;
                }
                if let (Some(owner), Some(prefix)) = (packet_owner, resolved.preceding_prefix)
                    && let Some(packet) = self.prepare_preceding_type_prefix(
                        comments,
                        run,
                        owner,
                        right,
                        prefix,
                        operator_span,
                    )
                {
                    left_mode = ContentMode::Verbatim;
                    preceding_packet = Some(packet);
                    continue;
                }
                source = true;
                if selected.preceding() {
                    left_mode = ContentMode::Verbatim;
                }
                if !selected.following() {
                    continue;
                }
                // The const assertion marker has no attachable type node.
                // Its comments select the expression on the left instead.
                if const_marker {
                    left_mode = ContentMode::Verbatim;
                } else {
                    self.select_type_head_with_attachment(right, &attachment, position);
                }
            }
            self.finalize_type_head(right, &attachment, comments);
            if left_mode == ContentMode::Formatted && !const_marker {
                suppressed_head = self.prepare_suppressed_type_head(
                    *right,
                    &attachment,
                    gap,
                    before,
                    after,
                    SuppressedTypeHeadRole::ExpressionBoundary,
                );
            }
        }
        if !source {
            *right = self.normalize_type_head(*right)?;
        }
        let layout = if let Some((packet, after)) = preceding_packet {
            let operator_end = start + token.len() as u32;
            let head =
                self.prepare_binary_type_head(*right, TypeContext::Root, &after, operator_end);
            BinaryTypeLayout::Preceding {
                packet,
                after,
                head,
            }
        } else if let Some(head) = suppressed_head {
            BinaryTypeLayout::Suppressed {
                before: self.prepare_comment_boundary(before, gap.start, start),
                head,
            }
        } else if source {
            let source = SourceTypeOperator::new(gap, operator_span);
            let spacing = SourceKeywordSpacing::new(gap, operator_span)
                .with_copied_left(left_mode == ContentMode::Verbatim);
            if let Some(after) = self.prepare_type_rhs_boundary(source, *right) {
                let head = self.prepare_binary_type_head(
                    *right,
                    TypeContext::Root,
                    &after,
                    operator_span.end,
                );
                BinaryTypeLayout::SourceRhs {
                    prefix: source.prefix,
                    spacing,
                    after,
                    head,
                }
            } else {
                BinaryTypeLayout::Source { source, spacing }
            }
        } else if let Some((after, tail)) =
            self.prepare_const_marker_comments(*right, operator, after, gap, operator_span.end)
        {
            let head =
                self.prepare_binary_type_head(*right, TypeContext::Root, &after, operator_span.end);
            BinaryTypeLayout::ConstMarker {
                before: self.prepare_comment_boundary(before, gap.start, start),
                after,
                head,
                tail,
            }
        } else {
            let operator_end = start + token.len() as u32;
            let after = self.prepare_comment_boundary(after, operator_end, gap.end);
            let head =
                self.prepare_binary_type_head(*right, TypeContext::Root, &after, operator_end);
            BinaryTypeLayout::Formatted {
                before: self.prepare_comment_boundary(before, gap.start, start),
                after,
                head,
            }
        };
        Ok(BinaryTypeBoundary {
            operator,
            left_mode,
            layout,
        })
    }

    /// Prepare the packet between a terminal const assertion and its semicolon.
    /// The assertion retains the comments between its own keywords.
    pub(in crate::formatter) fn prepare_const_assertion_statement_tail(
        &self,
        assertion: TsAsExpression,
        statement_span: Span,
    ) -> Option<ExpressionTail<'ast>> {
        let mut right =
            self.prepare_type(assertion.type_annotation(self.ast), ContentMode::Formatted);
        if !self.is_const_marker_type(right) {
            return None;
        }
        let boundary = self
            .prepare_binary_type_boundary(
                assertion.expression(self.ast),
                &mut right,
                TypeBinaryOperator::As,
            )
            .ok()?;
        let BinaryTypeLayout::ConstMarker { tail, .. } = boundary.layout else {
            return None;
        };
        if tail.as_ref().is_some_and(|tail| {
            !self.inline_unbound_const_tail(tail, right.node.span(self.ast).start)
        }) {
            return None;
        }
        let end = assertion.span(self.ast).end;
        let next_start = self
            .statement_source_terminator(statement_span)
            .map_or(end, |terminator| terminator.start());
        let comments = self.comments.in_range(Span::new(end, next_start));
        if tail.is_none() && comments.is_empty() {
            return None;
        }
        // A multiline block in this slot needs the explicit before-semicolon owner.
        if comments
            .first()
            .is_some_and(|first| self.line_breaks(first.span.start, first.span.end) == 0)
        {
            return None;
        }
        let suffix = ExpressionTail {
            source_end: end,
            comments,
        };
        self.inline_unbound_const_tail(&suffix, next_start)
            .then_some(suffix)
    }

    fn inline_unbound_const_tail(&self, tail: &ExpressionTail<'ast>, next_start: u32) -> bool {
        tail.comments
            .first()
            .is_none_or(|first| self.line_breaks(tail.source_end, first.span.start) == 0)
            && tail
                .comments
                .iter()
                .all(|comment| comment.kind == CommentKind::Block)
            && matches!(
                self.comment_binding(tail.comments, Span::new(tail.source_end, next_start), None,),
                CommentBinding::Unbound
            )
    }

    /// Keep the marker's head as one lexical packet. Its suffix facts are
    /// used only to decide whether a separate statement tail can be moved.
    fn prepare_const_marker_comments(
        &self,
        right: PreparedType,
        operator: TypeBinaryOperator,
        comments: &'ast [Comment],
        gap: Span,
        operator_end: u32,
    ) -> Option<(CommentBoundary<'ast>, Option<ExpressionTail<'ast>>)> {
        if !matches!(operator, TypeBinaryOperator::As)
            || !self.is_const_marker_type(right)
            || self.diagnostic_range_is_sensitive(gap)
        {
            return None;
        }
        let split = comments
            .iter()
            .take_while(|comment| {
                comment.kind == CommentKind::Block
                    && self.line_breaks(comment.span.start, comment.span.end) == 0
            })
            .count();
        for (index, &comment) in comments.iter().enumerate() {
            let semantics = self.comment_semantics(comment);
            // A bound annotation needs its explicit source placement even
            // when the const assertion is the end of the statement.
            if semantics.suppression
                || semantics.diagnostic
                || semantics.file_directive
                || index >= split && semantics.has_target_annotation()
            {
                return None;
            }
        }
        let (prefix, suffix) = comments.split_at(split);
        let source_end = prefix
            .last()
            .map_or(operator_end, |comment| comment.span.end);
        let tail = (!suffix.is_empty()).then_some(ExpressionTail {
            source_end,
            comments: suffix,
        });
        Some((
            self.prepare_comment_boundary(comments, operator_end, gap.end),
            tail,
        ))
    }

    fn is_const_marker_type(&self, right: PreparedType) -> bool {
        if right.source_owner.is_some() {
            return false;
        }
        let TsTypeData::TsTypeReference(reference) = right.data else {
            return false;
        };
        if reference.type_arguments(self.ast).is_some() {
            return false;
        }
        let TsTypeNameData::IdentifierReference(name) =
            self.ast.ts_type_name_data(reference.type_name(self.ast))
        else {
            return false;
        };
        self.ast.get_utf8(name.name(self.ast)) == "const"
    }

    fn format_binary_type_right_output(
        &mut self,
        boundary: BinaryTypeBoundary<'ast>,
        node: PreparedType,
        context: TypeContext,
        continuation: bool,
        writer: &mut Writer<'ast>,
    ) -> Result<ExpressionOutput<'ast>, FormatError> {
        match boundary.layout {
            BinaryTypeLayout::Preceding {
                packet,
                after,
                head,
            } => {
                self.format_preceding_type_packet(&packet, writer)?;
                writer.space();
                writer.text(boundary.operator.token());
                writer.space();
                if continuation {
                    self.format_binary_type_rhs_before_continuation(
                        node, context, &after, head, writer,
                    )
                } else {
                    self.format_binary_type_rhs_head_output(node, context, &after, head, writer)
                }
            }
            BinaryTypeLayout::Source { source, spacing } => {
                self.format_source_type_operator_rhs_output(source, spacing, node, context, writer)
            }
            BinaryTypeLayout::SourceRhs {
                prefix,
                spacing,
                after,
                head,
            } => {
                spacing.write_before(writer);
                self.format_source_gap(prefix, writer)?;
                writer.space();
                if continuation {
                    self.format_binary_type_rhs_before_continuation(
                        node, context, &after, head, writer,
                    )
                } else {
                    self.format_binary_type_rhs_head_output(node, context, &after, head, writer)
                }
            }
            BinaryTypeLayout::Suppressed { before, head } => {
                self.format_comment_boundary(&before, writer)?;
                before.gap().write_separator(writer);
                writer.text(boundary.operator.token());
                writer.space();
                self.format_suppressed_type_content_output(
                    node,
                    &head,
                    context,
                    TypeLayout::OpeningProvided,
                    writer,
                )
            }
            BinaryTypeLayout::ConstMarker {
                before,
                after,
                head,
                ..
            }
            | BinaryTypeLayout::Formatted {
                before,
                after,
                head,
            } => {
                self.format_comment_boundary(&before, writer)?;
                before.gap().write_separator(writer);
                writer.text(boundary.operator.token());
                writer.text(" ");
                if continuation {
                    self.format_binary_type_rhs_before_continuation(
                        node, context, &after, head, writer,
                    )
                } else {
                    self.format_binary_type_rhs_head_output(node, context, &after, head, writer)
                }
            }
        }
    }

    pub(super) fn source_non_null(
        &self,
        node: TsNonNullExpression,
    ) -> Result<SourceNonNull<'ast>, FormatError> {
        let expression = node.expression(self.ast);
        let suffix = Span::new(expression.span(self.ast).end, node.span(self.ast).end);
        let comments = self.comments.in_range(suffix);
        let Some(operator) = self.find_token(suffix, comments, "!") else {
            return Err(self.unsupported("non-null assertion token", node));
        };
        let gap = Span::new(suffix.start, operator);
        let mode = self.content_mode(comments);
        let prefix = if mode == ContentMode::Verbatim {
            TypeExpressionGap::Source(gap)
        } else {
            TypeExpressionGap::Formatted(
                self.prepare_comment_boundary(comments, gap.start, gap.end),
            )
        };
        Ok(SourceNonNull {
            expression,
            mode,
            prefix,
            has_comments: !comments.is_empty(),
        })
    }

    pub(super) fn format_source_non_null(
        &mut self,
        source: SourceNonNull<'ast>,
        tail: Option<ExpressionTail<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        self.format_expression_tail(tail, writer)?;
        self.format_type_expression_gap(source.prefix, FlatSeparator::Empty, writer)?;
        writer.text("!");
        Ok(())
    }

    pub(super) fn format_non_null(
        &mut self,
        node: TsNonNullExpression,
        context: ExpressionContext<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let source = self.source_non_null(node)?;
        let tail =
            self.format_expression(source.expression, source.operand_context(context), writer)?;
        self.format_source_non_null(source, tail, writer)?;
        Ok(None)
    }

    pub(super) fn format_type_assertion_output(
        &mut self,
        node: TsTypeAssertion,
        context: ExpressionContext<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<ExpressionOutput<'ast>, FormatError> {
        let PreparedTypeAssertion {
            head,
            operand,
            grammar_barrier,
        } = self.prepare_type_assertion(node, context)?;
        writer.group(false, |writer| {
            if grammar_barrier {
                writer.text("(");
            }
            let tail = self.format_prepared_type_assertion(head, operand, writer)?;
            if grammar_barrier {
                writer.text(")");
                Ok(ExpressionOutput::closed(tail.pending))
            } else {
                Ok(tail)
            }
        })
    }

    fn format_prepared_type_assertion(
        &mut self,
        head: TypeAssertionHead<'ast>,
        operand: TypeAssertionOperand<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<ExpressionOutput<'ast>, FormatError> {
        if matches!(operand.layout, AssertionOperandLayout::Direct) {
            writer.group(false, |writer| {
                self.format_type_assertion_head(head, writer)
            })?;
            if operand.head.has_leading_separator() {
                writer.space();
            }
            return self.format_type_assertion_operand(operand, writer);
        }
        let (head, ()) = writer
            .record_group_with_value(|writer| self.format_type_assertion_head(head, writer))?;
        let leading_separator = operand.head.has_leading_separator();
        let (operand, tail) = writer
            .record_with_value(|writer| self.format_type_assertion_operand(operand, writer))?;
        let direct = writer.join_fragments(head.grouped(), leading_separator, operand);
        let enclosed = writer.conditional_group_fragment(
            head.grouped(),
            GroupRequest::Expanded,
            "(",
            operand,
            ")",
        );
        writer.write_alternatives(&[direct, enclosed], direct);
        // Ordinary pending comments retain their enclosing expression owner.
        // Each candidate finishes its operand before returning the same lease.
        Ok(tail)
    }

    fn type_assertion_closing(
        &self,
        node: TsTypeAssertion,
    ) -> Result<TypeAssertionClosing<'ast>, FormatError> {
        let gap = Span::new(
            node.type_annotation(self.ast).span(self.ast).end,
            node.expression(self.ast).span(self.ast).start,
        );
        let comments = self.source_comments_in_range(gap);
        let closing = self
            .find_token(gap, comments, ">")
            .ok_or_else(|| self.unsupported("type assertion closing token", node))?;
        let split = comments.partition_point(|comment| comment.span.end <= closing);
        let (type_tail, operand_head) = comments.split_at(split);
        Ok(TypeAssertionClosing {
            closing,
            type_tail,
            operand_head,
        })
    }

    fn prepare_type_assertion(
        &self,
        node: TsTypeAssertion,
        context: ExpressionContext<'ast>,
    ) -> Result<PreparedTypeAssertion<'ast>, FormatError> {
        let span = node.span(self.ast);
        let expression = node.expression(self.ast);
        let expression_span = expression.span(self.ast);
        let annotation = node.type_annotation(self.ast);
        let annotation_span = annotation.span(self.ast);
        let mut type_node = self.prepare_type(annotation, ContentMode::Formatted);
        let before = Span::new(span.start + 1, annotation_span.start);
        let leading = self.comments.in_range(before);
        let mut attachment = None;
        if self.has_suppression && self.content_mode(leading) == ContentMode::Verbatim {
            let attachment =
                attachment.get_or_insert_with(|| self.type_attachment_head(type_node.node));
            let head = self.prepare_type_suppression_head(attachment, leading);
            for (_, position) in self.suppressed_comment_runs(leading, before.end) {
                self.select_type_head_with_retention(&mut type_node, attachment, head, position);
            }
        }
        let TypeAssertionClosing {
            closing,
            type_tail,
            operand_head,
        } = self.type_assertion_closing(node)?;
        let type_source =
            self.has_suppression && self.content_mode(type_tail) == ContentMode::Verbatim;
        if type_source {
            let attachment =
                attachment.get_or_insert_with(|| self.type_attachment_head(type_node.node));
            self.select_type_head_with_attachment(
                &mut type_node,
                attachment,
                SuppressionPosition::Inline,
            );
        }
        if let Some(attachment) = &attachment {
            self.finalize_type_head(&mut type_node, attachment, leading);
        }
        let packet = attachment.as_ref().and_then(|attachment| {
            self.prepare_type_angle_packet(type_node, attachment, before, leading)
        });
        let before = match packet {
            Some(packet) => TypeAssertionOpening::Packet(packet),
            None => {
                let boundary = match self.prepare_type_expression_gap(
                    before,
                    leading,
                    type_node.source_owner.is_some(),
                ) {
                    TypeExpressionGap::Source(span) => {
                        TypeExpressionGap::Source(self.type_angle_source_head(span, leading))
                    }
                    boundary => boundary,
                };
                TypeAssertionOpening::Boundary(boundary)
            }
        };
        let body = self.prepare_delimited_type(type_node)?;
        let after_type = self.prepare_type_expression_gap(
            Span::new(annotation_span.end, closing),
            type_tail,
            type_source,
        );
        let mode = self.content_mode(operand_head);
        let after_angle = self.prepare_type_expression_gap(
            Span::new(closing + 1, expression_span.start),
            operand_head,
            mode == ContentMode::Verbatim,
        );
        let operand_context = context.type_operand(
            ExpressionPosition::TypeAssertionOperand,
            mode,
            context.tail_owner,
        );
        // Keep protected operands direct before preparing optional document variants.
        let content = (mode == ContentMode::Formatted).then(|| self.expression_content(expression));
        let direct_body = content.as_ref().is_some_and(|content| {
            matches!(
                content.data,
                ExprData::ArrayExpression(_)
                    | ExprData::ObjectExpression(_)
                    // The prepared peel stops at this actual protected source pair.
                    | ExprData::ParenthesizedExpression(_)
            )
        });
        let head = match (after_angle, content) {
            (TypeExpressionGap::Formatted(boundary), Some(content)) => {
                let binding = self.expression_head_binding(operand_head, expression);
                let binding = content
                    .parentheses
                    .map_or(binding, |removed| binding.combine(removed.head.binding));
                if matches!(binding, CommentBinding::SourceBound)
                    || self.diagnostic_range_is_sensitive(span)
                {
                    TypeAssertionOperandHead::Boundary {
                        before: TypeExpressionGap::Formatted(boundary),
                        content: Some(content),
                    }
                } else if matches!(
                    content.data,
                    ExprData::CallExpression(_)
                        | ExprData::MemberExpression(_)
                        | ExprData::TsNonNullExpression(_)
                        | ExprData::ChainExpression(_)
                ) && !self.comments.in_range(expression_span).is_empty()
                {
                    let expression = content.expression;
                    let (chain, head) = self.prepare_assertion_chain(
                        content,
                        operand_context,
                        closing + 1,
                        binding,
                    )?;
                    TypeAssertionOperandHead::Prepared {
                        head,
                        content: TypeAssertionContent::Chain {
                            chain: Box::new(chain),
                            expression,
                        },
                    }
                } else {
                    let start = content.expression.span(self.ast).start;
                    let comments = self.comments.in_range(Span::new(closing + 1, start));
                    TypeAssertionOperandHead::Prepared {
                        head: ResolvedExpressionHead {
                            head: NodeHead {
                                comments,
                                content_start: start,
                                layout: CommentRunLayout::FlatLeading(start),
                                gap_end: None,
                            },
                            binding,
                        },
                        content: TypeAssertionContent::Expression(content),
                    }
                }
            }
            (before, content) => TypeAssertionOperandHead::Boundary { before, content },
        };
        let shift_operand = is_left_shift_right_operand(context.position);
        let layout = if shift_operand
            || mode == ContentMode::Verbatim
            || direct_body
            || self.diagnostic_range_is_sensitive(span)
            || matches!(head, TypeAssertionOperandHead::Boundary { .. })
        {
            AssertionOperandLayout::Direct
        } else {
            let movable = match &head {
                TypeAssertionOperandHead::Prepared { head, content } => head
                    .binding
                    .allows_relocation(Some(content.expression().span(self.ast))),
                TypeAssertionOperandHead::Boundary { .. } => false,
            };
            if movable {
                AssertionOperandLayout::Enclosable
            } else {
                AssertionOperandLayout::Direct
            }
        };
        let operand = TypeAssertionOperand {
            expression,
            context: operand_context,
            head,
            layout,
        };
        let grammar_barrier = if shift_operand {
            self.type_assertion_operand_needs_grammar_barrier(&operand, closing + 1)?
        } else {
            false
        };
        Ok(PreparedTypeAssertion {
            head: TypeAssertionHead {
                body,
                before,
                closing: after_type,
            },
            operand,
            grammar_barrier,
        })
    }

    fn type_assertion_operand_needs_grammar_barrier(
        &self,
        operand: &TypeAssertionOperand<'ast>,
        post_angle_start: u32,
    ) -> Result<bool, FormatError> {
        let (mut data, mut context) = match &operand.head {
            TypeAssertionOperandHead::Prepared {
                content: TypeAssertionContent::Chain { chain, .. },
                ..
            } => {
                let (expression, context) = chain.emitted_head();
                (self.ast.expr_data(expression), context)
            }
            TypeAssertionOperandHead::Prepared {
                content: TypeAssertionContent::Expression(content),
                ..
            }
            | TypeAssertionOperandHead::Boundary {
                content: Some(content),
                ..
            } => (content.data, operand.context),
            TypeAssertionOperandHead::Boundary { content: None, .. } => {
                (self.ast.expr_data(operand.expression), operand.context)
            }
        };
        let comments_require_line = |comments: &[Comment], content_start, layout| {
            let mut source_end = match layout {
                CommentRunLayout::SourceAfter(start) => Some(start),
                _ => None,
            };
            let mut comments = comments.iter().peekable();
            while let Some(comment) = comments.next() {
                let next = comments
                    .peek()
                    .map_or(content_start, |comment| comment.span.start);
                if comment.kind == CommentKind::Line
                    || self.line_breaks(comment.span.start, comment.span.end) != 0
                    || source_end.is_some_and(|end| self.line_breaks(end, comment.span.start) != 0)
                    || matches!(
                        self.line_constraint(comment.span.end, next),
                        LineConstraint::NewLine(_)
                    )
                {
                    return true;
                }
                if matches!(layout, CommentRunLayout::Leading(_)) {
                    let breaks = self.line_breaks(comment.span.end, next);
                    if breaks > 1
                        || breaks == 1 && self.layout_position_starts_line(comment.span.start)
                    {
                        return true;
                    }
                }
                source_end = source_end.map(|_| comment.span.end);
            }
            false
        };
        let required_line = match &operand.head {
            TypeAssertionOperandHead::Boundary {
                before: TypeExpressionGap::Source(span),
                ..
            } => self.line_breaks(span.start, span.end) != 0,
            TypeAssertionOperandHead::Boundary {
                before: TypeExpressionGap::Formatted(boundary),
                ..
            } => {
                boundary.gap().requires_line()
                    || comments_require_line(
                        boundary.comments(),
                        operand.expression.span(self.ast).start,
                        CommentRunLayout::SourceAfter(post_angle_start),
                    )
            }
            TypeAssertionOperandHead::Prepared { head, .. } => comments_require_line(
                head.head.comments,
                head.head.content_start,
                head.head.layout,
            ),
        };
        if required_line {
            return Ok(true);
        }
        // TypeScript's speculative type-argument parser accepts these heads
        // after `>`. A second complete assertion pair separates the real shift.
        loop {
            let content = self.source_parentheses_head_content(data, context);
            data = content.data;
            context.content_mode = content.mode;
            if content.leading().is_some_and(|leading| {
                comments_require_line(
                    self.comments.in_range(leading),
                    leading.end,
                    CommentRunLayout::Leading(leading.end),
                )
            }) {
                return Ok(true);
            }
            if self.expression_is_parenthesized(data, context) {
                return Ok(true);
            }
            if context.content_mode == ContentMode::Verbatim {
                return Ok(self.raw_type_assertion_head_needs_grammar_barrier(data));
            }
            if matches!(
                data,
                ExprData::ParenthesizedExpression(_)
                    | ExprData::TemplateLiteral(_)
                    | ExprData::RegExpLiteral(_)
            ) {
                return Ok(true);
            }
            let Some((expression, child_context)) = self.expression_head_operand(data, context)?
            else {
                return Ok(false);
            };
            context = child_context;
            data = match context.content_mode {
                ContentMode::Verbatim => self.ast.expr_data(expression),
                ContentMode::Formatted => {
                    let content = self.expression_content(expression);
                    let head = self.comments.in_range(Span::new(
                        expression.span(self.ast).start,
                        content.expression.span(self.ast).start,
                    ));
                    if comments_require_line(
                        head,
                        content.expression.span(self.ast).start,
                        CommentRunLayout::Leading(content.expression.span(self.ast).start),
                    ) {
                        return Ok(true);
                    }
                    content.data
                }
            };
        }
    }

    pub(super) fn source_shift_operand_needs_grammar_barrier(
        &self,
        data: ExprData,
        context: ExpressionContext<'ast>,
    ) -> Result<bool, FormatError> {
        if !is_left_shift_right_operand(context.position) {
            return Ok(false);
        }
        let child = match data {
            ExprData::ParenthesizedExpression(parenthesized) => {
                // The raw source pair and its actual parent already enclose the operand twice.
                if context.parentheses_owner == ParenthesesOwner::Parent {
                    return Ok(false);
                }
                self.ast.expr_data(parenthesized.expression(self.ast))
            }
            other => other,
        };
        let ExprData::TsTypeAssertion(node) = child else {
            return Ok(self.shift_suspension_needs_grammar_barrier(child, context.position));
        };
        let expression = node.expression(self.ast);
        let expression_span = expression.span(self.ast);
        let suffix = Span::new(
            node.type_annotation(self.ast).span(self.ast).end,
            expression_span.start,
        );
        let comments = self.comments.in_range(suffix);
        let Some(closing) = self.find_token(suffix, comments, ">") else {
            return Err(self.unsupported("type assertion closing token", node));
        };
        Ok(
            self.source_line_breaks(closing + 1, expression_span.start) != 0
                || self
                    .raw_type_assertion_head_needs_grammar_barrier(self.ast.expr_data(expression)),
        )
    }

    /// A raw subtree keeps its actual openings and never re-enters formatted owners.
    fn raw_type_assertion_head_needs_grammar_barrier(&self, mut data: ExprData) -> bool {
        loop {
            let expression = match data {
                ExprData::ParenthesizedExpression(_)
                | ExprData::TemplateLiteral(_)
                | ExprData::RegExpLiteral(_) => return true,
                ExprData::MemberExpression(member) => member.object(self.ast),
                ExprData::CallExpression(call) => call.callee(self.ast),
                ExprData::TaggedTemplateExpression(template) => template.tag(self.ast),
                ExprData::TsNonNullExpression(node) => node.expression(self.ast),
                ExprData::TsInstantiationExpression(node) => node.expression(self.ast),
                ExprData::TsAsExpression(node) => node.expression(self.ast),
                ExprData::TsSatisfiesExpression(node) => node.expression(self.ast),
                ExprData::ChainExpression(chain) => chain.expression(self.ast),
                _ => return false,
            };
            data = self.ast.expr_data(expression);
        }
    }

    pub(super) fn left_shift_parentheses_position(
        &self,
        expression: Expr,
        content: Option<&ExpressionContent>,
        position: ExpressionPosition,
    ) -> ExpressionPosition {
        if !is_left_shift_right_operand(position) {
            return ExpressionPosition::Nested;
        }
        let data = content.map_or_else(|| self.expression_data(expression), |content| content.data);
        if matches!(data, ExprData::TsTypeAssertion(_))
            || self.shift_suspension_needs_grammar_barrier(data, position)
        {
            position
        } else {
            ExpressionPosition::Nested
        }
    }

    /// The actual angle caller owns the opening gap and the type body's indentation.
    fn format_type_assertion_head(
        &mut self,
        head: TypeAssertionHead<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let context = TypeContext::AngleAssertion;
        let opening = match head.before {
            TypeAssertionOpening::Packet(packet) => {
                PreparedAssertionOpening::Packet(packet, head.body)
            }
            TypeAssertionOpening::Boundary(TypeExpressionGap::Source(span)) => {
                PreparedAssertionOpening::Source(span, head.body)
            }
            TypeAssertionOpening::Boundary(TypeExpressionGap::Formatted(boundary)) => {
                match self.prepare_delimited_type_head(head.body, boundary, context, writer)? {
                    PreparedDelimitedTypeHead::Recorded(head) => {
                        PreparedAssertionOpening::Recorded(head)
                    }
                    PreparedDelimitedTypeHead::Boundary(body, boundary) => {
                        PreparedAssertionOpening::Boundary(boundary, body)
                    }
                }
            }
        };
        writer.text("<");
        let closing = writer.indent(|writer| {
            let tail = match opening {
                PreparedAssertionOpening::Packet(packet, body) => {
                    writer.soft_line();
                    self.format_type_angle_packet(packet, writer)?;
                    self.format_prepared_delimited_type(
                        body,
                        context,
                        TypeLayout::CallerIndented,
                        writer,
                    )?
                }
                PreparedAssertionOpening::Source(span, body) => {
                    self.format_verbatim(span, writer)?;
                    self.format_prepared_delimited_type(
                        body,
                        context,
                        TypeLayout::CallerIndented,
                        writer,
                    )?
                }
                PreparedAssertionOpening::Boundary(boundary, body) => {
                    if boundary.is_empty() && body.starts_with_type_parameters(self.ast) {
                        writer.space();
                    }
                    if let Some(first) = boundary.comments().first() {
                        self.line_constraint(boundary.source_end(), first.span.start)
                            .write(writer, FlatSeparator::Empty, Writer::soft_line);
                        self.format_comment_boundary_with_layout(
                            &boundary,
                            CommentRunLayout::FlatLeading(body.source_start()),
                            writer,
                        )?;
                    } else {
                        boundary
                            .gap()
                            .write(writer, FlatSeparator::Empty, Writer::soft_line);
                    }
                    self.format_prepared_delimited_type(
                        body,
                        context,
                        TypeLayout::CallerIndented,
                        writer,
                    )?
                }
                PreparedAssertionOpening::Recorded(head) => {
                    writer.soft_line();
                    self.format_recorded_type_head(
                        head,
                        context,
                        TypeLayout::CallerIndented,
                        writer,
                    )?
                }
            };
            self.format_expression_tail(tail, writer)?;
            match head.closing {
                TypeExpressionGap::Source(span) => {
                    self.format_verbatim(span, writer)?;
                    Ok(None)
                }
                TypeExpressionGap::Formatted(boundary) => {
                    self.format_comment_boundary(&boundary, writer)?;
                    Ok(Some(boundary.gap()))
                }
            }
        })?;
        if let Some(closing) = closing {
            closing.write(writer, FlatSeparator::Empty, Writer::soft_line);
        }
        writer.text(">");
        Ok(())
    }

    fn format_type_assertion_operand(
        &mut self,
        operand: TypeAssertionOperand<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<ExpressionOutput<'ast>, FormatError> {
        match operand.head {
            TypeAssertionOperandHead::Prepared { head, content } => {
                self.comments.claim(head.head.comments)?;
                match content {
                    TypeAssertionContent::Expression(content) => self
                        .format_prepared_expression_with_resolved_head_and_raw_end(
                            operand.expression,
                            content,
                            head,
                            operand.context,
                            writer,
                        ),
                    TypeAssertionContent::Chain { mut chain, .. } => {
                        let start = self.format_node_head(
                            head.head,
                            operand.context.expression_start,
                            None,
                            writer,
                        )?;
                        chain.set_expression_start(start);
                        self.format_prepared_chain(*chain, writer)
                            .map(ExpressionOutput::unreported)
                    }
                }
            }
            TypeAssertionOperandHead::Boundary { before, content } => {
                match before {
                    TypeExpressionGap::Formatted(boundary)
                        if boundary.comments().first().is_some_and(|comment| {
                            comment.kind == CommentKind::Block
                                && self
                                    .source_line_breaks(boundary.source_end(), comment.span.start)
                                    == 0
                        }) =>
                    {
                        self.format_comment_boundary_after_separator(&boundary, writer)?;
                        boundary
                            .gap()
                            .write(writer, FlatSeparator::Space, Writer::space);
                    }
                    before => {
                        self.format_type_expression_gap(before, FlatSeparator::Empty, writer)?;
                    }
                }
                self.format_expression_prepared_output(
                    operand.expression,
                    content,
                    operand.context,
                    writer,
                )
            }
        }
    }

    fn prepare_type_expression_gap(
        &self,
        gap: Span,
        comments: &'ast [Comment],
        source: bool,
    ) -> TypeExpressionGap<'ast> {
        if source {
            TypeExpressionGap::Source(gap)
        } else {
            TypeExpressionGap::Formatted(
                self.prepare_comment_boundary(comments, gap.start, gap.end),
            )
        }
    }

    fn format_type_expression_gap(
        &mut self,
        gap: TypeExpressionGap<'ast>,
        separator: FlatSeparator,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        match gap {
            TypeExpressionGap::Source(span) => self.format_verbatim(span, writer),
            TypeExpressionGap::Formatted(boundary) => {
                self.format_comment_boundary(&boundary, writer)?;
                boundary.gap().write(writer, separator, |_| {});
                Ok(())
            }
        }
    }
}

fn is_left_shift_right_operand(position: ExpressionPosition) -> bool {
    matches!(
        position,
        ExpressionPosition::BinaryOperand {
            operator: BinaryOperatorKind::Binary(BinaryOperator::LeftShift),
            side: OperandSide::Right,
        }
    )
}

pub(super) fn type_expression_needs_parentheses(
    data: ExprData,
    position: ExpressionPosition,
    ast: &Ast<'_>,
) -> Option<bool> {
    match data {
        ExprData::TsAsExpression(_) | ExprData::TsSatisfiesExpression(_) => Some(matches!(
            position,
            ExpressionPosition::UnaryArgument(_)
                | ExpressionPosition::ParenthesizedUnaryArgument(_)
                | ExpressionPosition::AwaitArgument
                | ExpressionPosition::SpreadArgument
                | ExpressionPosition::BinaryOperand { .. }
                | ExpressionPosition::MemberObject(_)
                | ExpressionPosition::MemberAssignmentObject
                | ExpressionPosition::AssignmentTarget
                | ExpressionPosition::UpdateArgument
                | ExpressionPosition::CallCallee { .. }
                | ExpressionPosition::TemplateTag
                | ExpressionPosition::NewCallee { .. }
                | ExpressionPosition::Superclass { .. }
                | ExpressionPosition::ConditionalTest
                | ExpressionPosition::ConditionalConsequent
                | ExpressionPosition::ConditionalAlternate
                | ExpressionPosition::TypeAssertionOperand
                | ExpressionPosition::NonNullOperand
                | ExpressionPosition::InstantiationOperand
        )),
        ExprData::TsTypeAssertion(_) => Some(matches!(
            position,
            ExpressionPosition::UnaryArgument(_)
                | ExpressionPosition::AwaitArgument
                | ExpressionPosition::SpreadArgument
                | ExpressionPosition::TypeAssertionOperand
                | ExpressionPosition::MemberObject(_)
                | ExpressionPosition::MemberAssignmentObject
                | ExpressionPosition::AssignmentTarget
                | ExpressionPosition::UpdateArgument
                | ExpressionPosition::CallCallee { .. }
                | ExpressionPosition::TemplateTag
                | ExpressionPosition::NewCallee { .. }
                | ExpressionPosition::Superclass { .. }
                | ExpressionPosition::TypeBinaryOperand
                | ExpressionPosition::NonNullOperand
                | ExpressionPosition::InstantiationOperand
                | ExpressionPosition::BinaryOperand {
                    operator: BinaryOperatorKind::Binary(
                        BinaryOperator::BitwiseOr
                            | BinaryOperator::BitwiseXor
                            | BinaryOperator::BitwiseAnd
                            | BinaryOperator::LeftShift
                            | BinaryOperator::RightShift
                            | BinaryOperator::UnsignedRightShift,
                    ),
                    ..
                }
                | ExpressionPosition::BinaryOperand {
                    operator: BinaryOperatorKind::Binary(BinaryOperator::Exponent),
                    side: OperandSide::Left,
                }
        )),
        ExprData::TsInstantiationExpression(node) => Some(match position {
            ExpressionPosition::MemberObject(_)
            | ExpressionPosition::MemberAssignmentObject
            | ExpressionPosition::TemplateTag
            | ExpressionPosition::NonNullOperand
            | ExpressionPosition::InstantiationOperand => true,
            ExpressionPosition::CallCallee {
                access: AccessKind::Ordinary,
                has_type_arguments,
                ..
            }
            | ExpressionPosition::NewCallee { has_type_arguments }
            | ExpressionPosition::Superclass { has_type_arguments } => {
                has_type_arguments || has_nested_instantiation(node.expression(ast), ast)
            }
            _ => false,
        }),
        _ => None,
    }
}

// An outer call, new, or heritage may absorb the last angle list. For
// nested instantiations that would turn two separate instantiations into one.
fn has_nested_instantiation(mut expression: Expr, ast: &Ast<'_>) -> bool {
    loop {
        match ast.expr_data(expression) {
            ExprData::ParenthesizedExpression(pair) => expression = pair.expression(ast),
            ExprData::ChainExpression(chain) => expression = chain.expression(ast),
            ExprData::TsInstantiationExpression(_) => return true,
            _ => return false,
        }
    }
}

fn instantiation_tail_conflicts(operator: BinaryOperator) -> bool {
    matches!(
        operator,
        BinaryOperator::Add
            | BinaryOperator::Subtract
            | BinaryOperator::LessThan
            | BinaryOperator::GreaterThan
            | BinaryOperator::GreaterThanOrEqual
            | BinaryOperator::RightShift
            | BinaryOperator::UnsignedRightShift
    )
}

#[cfg(test)]
mod tests {
    use swc_next_ecma_ast::{ExprData, StmtData};

    use super::{ContentMode, Formatter};
    use crate::formatter::with_first_statement;

    #[test]
    fn binary_operand_comments() {
        with_first_statement(
            "(await /* prettier-ignore */ (fn<T>)) - b;",
            |formatter, statement| {
                let ast = formatter.ast;
                let StmtData::ExpressionStatement(statement) = ast.stmt_data(statement) else {
                    unreachable!()
                };
                let ExprData::BinaryExpression(binary) = ast.expr_data(statement.expression(ast))
                else {
                    unreachable!()
                };
                let left = formatter.expression_data(binary.left(ast));
                let exposed = |f: &Formatter<'_, '_>| {
                    f.binary_left_exposes_type_arguments(
                        left,
                        binary.operator(ast),
                        ContentMode::Formatted,
                    )
                };

                assert!(!exposed(formatter));
                formatter.comments.claim(ast.comments()).unwrap();
                assert!(!exposed(formatter));
            },
        );
    }
}
