//! TypeScript expression operators keep real operand owners apart from token gaps.

use swc_next_ecma_ast::{
    BinaryOperator, Comment, CommentKind, Expr, ExprData, GetSpan, Span, TsAsExpression,
    TsInstantiationExpression, TsNonNullExpression, TsType, TsTypeAssertion, TsTypeData,
    TsTypeNameData, TsTypeParameterInstantiation,
};

use super::binary::{BinaryLike, BinaryOperatorKind, OperandSide};
use super::chains::PreparedChain;
use super::parentheses::{ExpressionContent, ResolvedExpressionHead};
use super::{AccessKind, ExpressionContext, ExpressionPosition, ParenthesesOwner, TailOwner};
use crate::formatter::Formatter;
use crate::formatter::comments::{
    CommentBinding, CommentBoundary, CommentRunLayout, ExpressionTail, NodeHead,
};
use crate::formatter::diagnostics::{FlatSeparator, LineConstraint};
use crate::formatter::source::is_source_whitespace;
use crate::formatter::suppression::{ContentMode, SuppressionPosition};
use crate::formatter::types::{
    BinaryTypeHead, PreparedDelimitedType, PreparedType, TypeContext, TypeLayout,
};
use crate::{
    FormatError,
    document::{GroupRequest, Writer},
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

struct TypeAssertionHead<'ast> {
    body: PreparedDelimitedType<'ast>,
    before: TypeExpressionGap<'ast>,
    closing: TypeExpressionGap<'ast>,
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
    context: ExpressionContext,
    head: TypeAssertionOperandHead<'ast>,
    layout: AssertionOperandLayout,
}

impl TypeAssertionOperandHead<'_> {
    fn has_leading_separator(&self) -> bool {
        matches!(self, Self::Prepared { head, .. } if !head.head.comments.is_empty())
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

impl SourceNonNull<'_> {
    pub(super) fn operand_context(&self, context: ExpressionContext) -> ExpressionContext {
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

enum BinaryTypeLayout<'ast> {
    Source(Span),
    ConstMarker {
        before: CommentBoundary<'ast>,
        prefix: &'ast [Comment],
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
            data = self.expression_content(expression).data;
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
        self.format_type_arguments(arguments.node, arguments.mode, writer)
    }

    pub(super) fn format_instantiation(
        &mut self,
        node: TsInstantiationExpression,
        context: ExpressionContext,
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

    pub(super) fn instantiation_tail_needs_parentheses(
        &self,
        mut data: ExprData,
        operator: BinaryOperatorKind,
        mut mode: ContentMode,
    ) -> bool {
        if !matches!(
            operator,
            BinaryOperatorKind::Binary(
                BinaryOperator::Add
                    | BinaryOperator::Subtract
                    | BinaryOperator::Modulo
                    | BinaryOperator::LessThan
                    | BinaryOperator::GreaterThan
                    | BinaryOperator::GreaterThanOrEqual
                    | BinaryOperator::RightShift
                    | BinaryOperator::UnsignedRightShift,
            )
        ) {
            return false;
        }
        // Expression-start tokens (`+`, `-`, and Babel's `%`) can turn the suffix
        // into relational syntax. Single angles and right-angle operators also
        // conflict with parsing the type-argument close. Follow only the printed
        // right spine, stopping at a grammar pair that already closes it.
        loop {
            let (expression, position) = match data {
                ExprData::TsInstantiationExpression(_) => return true,
                ExprData::UnaryExpression(unary) => (
                    unary.argument(self.ast),
                    ExpressionPosition::UnaryArgument(unary.operator(self.ast)),
                ),
                ExprData::TsTypeAssertion(assertion) => (
                    assertion.expression(self.ast),
                    ExpressionPosition::TypeAssertionOperand,
                ),
                ExprData::ChainExpression(chain) => {
                    (chain.expression(self.ast), ExpressionPosition::Nested)
                }
                _ => {
                    let Some(binary) = BinaryLike::from_data(data, self.ast) else {
                        return false;
                    };
                    (
                        binary.right,
                        ExpressionPosition::BinaryOperand {
                            operator: binary.operator,
                            side: OperandSide::Right,
                        },
                    )
                }
            };
            let next = match mode {
                ContentMode::Formatted => self.expression_content(expression).data,
                ContentMode::Verbatim => self.ast.expr_data(expression),
            };
            let content = self.source_parentheses_head_content(
                next,
                ExpressionContext::at(position).with_content_mode(mode),
            );
            data = content.data;
            mode = content.mode;
            if self.expression_needs_parentheses(data, position, mode) != Some(false) {
                return false;
            }
        }
    }

    /// These identifier tokens could expose a declaration or reserved statement head.
    pub(super) fn type_binary_head_needs_parentheses(&self, data: ExprData) -> bool {
        let ExprData::IdentifierReference(identifier) = data else {
            return false;
        };
        matches!(
            self.ast.get_utf8(identifier.name(self.ast)),
            "await" | "interface" | "module" | "namespace" | "using" | "yield" | "let" | "type"
        )
    }

    pub(super) fn format_binary_type_expression(
        &mut self,
        expression: Expr,
        annotation: TsType,
        operator: TypeBinaryOperator,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let mut type_node = self.prepare_type(annotation, ContentMode::Formatted);
        let boundary = self.prepare_binary_type_boundary(
            expression,
            &mut type_node,
            operator,
            context.tail_owner,
        )?;
        writer.group(false, |writer| {
            let write = |writer: &mut Writer<'ast>| {
                let tail = self.format_expression(
                    expression,
                    context
                        .type_operand(
                            ExpressionPosition::TypeBinaryOperand,
                            boundary.left_mode,
                            TailOwner::ParentOperation,
                        )
                        .with_conditional_envelope(context.conditional_envelope.terminal_head()),
                    writer,
                )?;
                self.format_expression_tail(tail, writer)?;
                self.format_binary_type_right(boundary, type_node, writer)
            };
            let receiver = context.is_call_or_member_receiver()
                || matches!(context.position, ExpressionPosition::NewCallee);
            if receiver && context.parentheses_owner == ParenthesesOwner::Expression {
                let tail = writer.indent(|writer| {
                    writer.soft_line();
                    write(writer)
                })?;
                writer.soft_line();
                Ok(tail)
            } else {
                write(writer)
            }
        })
    }

    fn prepare_binary_type_boundary(
        &self,
        expression: Expr,
        right: &mut PreparedType,
        operator: TypeBinaryOperator,
        tail_owner: TailOwner,
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
        let split = comments.partition_point(|comment| comment.span.end <= start);
        let (before, after) = comments.split_at(split);
        let mut left_mode = ContentMode::Formatted;
        let mut source = false;
        if self.has_suppression && self.content_mode(comments) == ContentMode::Verbatim {
            for (_, position) in self.suppressed_comment_runs(before, start) {
                source = true;
                if matches!(position, SuppressionPosition::OwnLine) {
                    self.select_expression_type_head(right, position);
                } else {
                    left_mode = ContentMode::Verbatim;
                }
            }
            for (_, position) in self.suppressed_comment_runs(after, gap.end) {
                source = true;
                if matches!(position, SuppressionPosition::EndOfLine) {
                    left_mode = ContentMode::Verbatim;
                } else {
                    self.select_expression_type_head(right, position);
                }
            }
        }
        let layout = if source {
            BinaryTypeLayout::Source(gap)
        } else if let Some((prefix, tail)) = self.prepare_const_marker_comments(
            *right,
            operator,
            after,
            gap,
            start + token.len() as u32,
        ) {
            BinaryTypeLayout::ConstMarker {
                before: self.prepare_comment_boundary(before, gap.start, start),
                prefix,
                tail,
            }
        } else {
            let operator_end = start + token.len() as u32;
            let after = self.prepare_comment_boundary(after, operator_end, gap.end);
            let head = self.prepare_binary_type_head(*right, &after, operator_end, tail_owner);
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

    /// The terminal const assertion and its statement share the same pending packet.
    /// A source packet before the semicolon is prepared here as well, so a moved
    /// operator packet has the same owner when its output is formatted again.
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
                TailOwner::Expression,
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
        // The marker's deferred Block packet starts with a mandatory comment
        // line. Flat source suffixes keep the canonical statement-tail owner.
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

    /// `const` is an assertion marker, not an attachable type reference.
    /// Its inline blocks belong to the operand; a remaining ordinary run is
    /// deferred to the assertion's parent, which owns closing punctuation.
    fn prepare_const_marker_comments(
        &self,
        right: PreparedType,
        operator: TypeBinaryOperator,
        comments: &'ast [Comment],
        gap: Span,
        operator_end: u32,
    ) -> Option<(&'ast [Comment], Option<ExpressionTail<'ast>>)> {
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
            // Inline annotations remain before a grammar keyword on both sides
            // of the move. A suffix could expose a following expression target.
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
        Some((prefix, tail))
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

    fn format_binary_type_right(
        &mut self,
        boundary: BinaryTypeBoundary<'ast>,
        node: PreparedType,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        match boundary.layout {
            BinaryTypeLayout::Source(gap) => {
                self.format_verbatim(gap, writer)?;
                self.format_prepared_type_with_layout(
                    node,
                    TypeContext::Root,
                    TypeLayout::OpeningProvided,
                    writer,
                )
            }
            BinaryTypeLayout::ConstMarker {
                before,
                prefix,
                tail,
            } => {
                self.format_comment_boundary(&before, writer)?;
                before
                    .gap()
                    .write(writer, FlatSeparator::Space, Writer::reserved_space);
                self.comments.claim(prefix)?;
                self.format_inline_trailing_comments(prefix, writer)?;
                writer.reserved_space();
                writer.text("as const");
                Ok(tail)
            }
            BinaryTypeLayout::Formatted {
                before,
                after,
                head,
            } => {
                self.format_comment_boundary(&before, writer)?;
                before
                    .gap()
                    .write(writer, FlatSeparator::Space, Writer::reserved_space);
                writer.text(boundary.operator.token());
                writer.reserved_space();
                self.format_binary_type_rhs_head(node, &after, head, writer)
            }
        }
    }

    /// The expression caller's whole type owner excludes removable type parentheses.
    /// Own-line union comments are consumed by its real first member instead.
    fn select_expression_type_head(
        &self,
        prepared: &mut PreparedType,
        position: SuppressionPosition,
    ) {
        let mut node = prepared.node;
        let mut data = prepared.data;
        while let TsTypeData::TsParenthesizedType(parentheses) = data {
            node = parentheses.type_annotation(self.ast);
            data = self.ast.ts_type_data(node);
        }
        if prepared.source_owner == Some(node) {
            return;
        }
        let owner = if matches!(position, SuppressionPosition::OwnLine)
            && let TsTypeData::TsUnionType(union) = data
        {
            union
                .types(self.ast)
                .first()
                .map_or(node, |id| self.ast.get_node_in_sub_range(id))
        } else {
            node
        };
        prepared.source_owner = Some(owner);
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
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let source = self.source_non_null(node)?;
        let tail =
            self.format_expression(source.expression, source.operand_context(context), writer)?;
        self.format_source_non_null(source, tail, writer)?;
        Ok(None)
    }

    pub(super) fn format_type_assertion(
        &mut self,
        node: TsTypeAssertion,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
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
            }
            Ok(tail)
        })
    }

    fn format_prepared_type_assertion(
        &mut self,
        head: TypeAssertionHead<'ast>,
        operand: TypeAssertionOperand<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
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
        writer.write_alternatives(direct, &[direct, enclosed], direct);
        // Ordinary pending comments retain their enclosing expression owner.
        // Each candidate finishes its operand before returning the same lease.
        Ok(tail)
    }

    fn prepare_type_assertion(
        &self,
        node: TsTypeAssertion,
        context: ExpressionContext,
    ) -> Result<PreparedTypeAssertion<'ast>, FormatError> {
        let span = node.span(self.ast);
        let expression = node.expression(self.ast);
        let expression_span = expression.span(self.ast);
        let annotation = node.type_annotation(self.ast);
        let annotation_span = annotation.span(self.ast);
        let mut type_node = self.prepare_type(annotation, ContentMode::Formatted);
        let before = Span::new(span.start + 1, annotation_span.start);
        let leading = self.comments.in_range(before);
        if self.has_suppression && self.content_mode(leading) == ContentMode::Verbatim {
            for (_, position) in self.suppressed_comment_runs(leading, before.end) {
                self.select_expression_type_head(&mut type_node, position);
            }
        }
        let suffix = Span::new(annotation_span.end, expression_span.start);
        let comments = self.comments.in_range(suffix);
        let Some(closing) = self.find_token(suffix, comments, ">") else {
            return Err(self.unsupported("type assertion closing token", node));
        };
        let split = comments.partition_point(|comment| comment.span.end <= closing);
        let (type_tail, operand_head) = comments.split_at(split);
        let type_source =
            self.has_suppression && self.content_mode(type_tail) == ContentMode::Verbatim;
        if type_source {
            self.select_expression_type_head(&mut type_node, SuppressionPosition::Inline);
        }
        let before =
            self.prepare_type_expression_gap(before, leading, type_node.source_owner.is_some());
        let body = self.prepare_delimited_type(type_node)?;
        let after_type = self.prepare_type_expression_gap(
            Span::new(suffix.start, closing),
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
                        || breaks == 1 && self.source_position_starts_line(comment.span.start)
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

    pub(super) fn source_type_assertion_needs_grammar_barrier(
        &self,
        data: ExprData,
        context: ExpressionContext,
    ) -> Result<bool, FormatError> {
        if !is_left_shift_right_operand(context.position) {
            return Ok(false);
        }
        let node = match data {
            ExprData::TsTypeAssertion(node) => node,
            ExprData::ParenthesizedExpression(parenthesized) => {
                // The raw source pair and its actual parent already enclose the assertion twice.
                if context.parentheses_owner == ParenthesesOwner::Parent {
                    return Ok(false);
                }
                let ExprData::TsTypeAssertion(node) =
                    self.ast.expr_data(parenthesized.expression(self.ast))
                else {
                    return Ok(false);
                };
                node
            }
            _ => return Ok(false),
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
        Ok(self.line_breaks(closing + 1, expression_span.start) != 0
            || self.raw_type_assertion_head_needs_grammar_barrier(self.ast.expr_data(expression)))
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

    pub(super) fn type_assertion_parentheses_position(
        &self,
        expression: Expr,
        content: Option<&ExpressionContent>,
        position: ExpressionPosition,
    ) -> ExpressionPosition {
        if !is_left_shift_right_operand(position) {
            return ExpressionPosition::Nested;
        }
        let data = content.map_or_else(
            || self.expression_content(expression).data,
            |content| content.data,
        );
        if matches!(data, ExprData::TsTypeAssertion(_)) {
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
        writer.text("<");
        let closing = writer.indent(|writer| {
            let tail = match head.before {
                TypeExpressionGap::Source(span) => {
                    self.format_verbatim(span, writer)?;
                    self.format_prepared_delimited_type(
                        head.body,
                        TypeContext::Root,
                        TypeLayout::CallerIndented,
                        writer,
                    )?
                }
                TypeExpressionGap::Formatted(boundary) => self
                    .format_prepared_delimited_type_after_comments(
                        head.body,
                        &boundary,
                        TypeContext::Root,
                        TypeLayout::CallerIndented,
                        writer,
                    )?,
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
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        match operand.head {
            TypeAssertionOperandHead::Prepared { head, content } => {
                self.comments.claim(head.head.comments)?;
                match content {
                    TypeAssertionContent::Expression(content) => self
                        .format_prepared_expression_with_resolved_head(
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
                    }
                }
            }
            TypeAssertionOperandHead::Boundary { before, content } => {
                self.format_type_expression_gap(before, FlatSeparator::Empty, writer)?;
                self.format_expression_prepared_with_array_comments(
                    operand.expression,
                    content,
                    operand.context,
                    None,
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
) -> Option<bool> {
    match data {
        ExprData::TsAsExpression(_) | ExprData::TsSatisfiesExpression(_) => Some(matches!(
            position,
            ExpressionPosition::UnaryArgument(_)
                | ExpressionPosition::ParenthesizedUnaryArgument(_)
                | ExpressionPosition::AwaitArgument
                | ExpressionPosition::YieldArgument
                | ExpressionPosition::SpreadArgument
                | ExpressionPosition::BinaryOperand { .. }
                | ExpressionPosition::MemberObject(_)
                | ExpressionPosition::MemberAssignmentObject
                | ExpressionPosition::AssignmentTarget
                | ExpressionPosition::UpdateArgument
                | ExpressionPosition::CallCallee { .. }
                | ExpressionPosition::TemplateTag
                | ExpressionPosition::NewCallee
                | ExpressionPosition::Superclass
                | ExpressionPosition::ConditionalTest
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
                | ExpressionPosition::NewCallee
                | ExpressionPosition::Superclass
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
        ExprData::TsInstantiationExpression(_)
            if matches!(
                position,
                ExpressionPosition::MemberObject(_)
                    | ExpressionPosition::MemberAssignmentObject
                    | ExpressionPosition::CallCallee { .. }
                    | ExpressionPosition::TemplateTag
                    | ExpressionPosition::NewCallee
                    | ExpressionPosition::Superclass
                    | ExpressionPosition::NonNullOperand
                    | ExpressionPosition::InstantiationOperand
                    | ExpressionPosition::BinaryOperand {
                        operator: BinaryOperatorKind::Binary(_),
                        side: OperandSide::Left,
                    }
            ) =>
        {
            Some(true)
        }
        _ => None,
    }
}
