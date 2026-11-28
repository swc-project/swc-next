//! Semicolon ownership between the current statement and the syntax that follows it.

mod head;
mod left_shift;

pub(super) use head::StatementStart;
pub(super) use left_shift::ShiftContinuation;

use swc_next_ecma_ast::{
    Ast, CommentKind, DeclData, Expr, ExprData, GetSpan, Span, StmtData, TsModuleReferenceData,
    TsTypeNameData, VariableDeclarator,
};

use super::tails::{StatementSourcePacket, StatementSuppressionTail, StatementTailRelation};
use super::{
    Formatter, PreparedStatement, StatementItem, StatementPunctuation, StatementRetention,
    StatementSequenceKind, StatementSourceLine, StatementSourceTerminator,
};
use crate::formatter::diagnostics::LineConstraint;
use crate::formatter::expressions::{
    CopiedRawOwner, ExpressionContext, ExpressionOutput, ExpressionPosition, ExpressionRightEdge,
    StringStatement,
};
use crate::formatter::suppression::ContentMode;
use crate::formatter::types::{RawTypeTerminal, type_tail_is_bare_abstract};
use crate::{FormatError, Semicolons};

impl StatementSequenceKind<'_> {
    pub(super) fn string_statement(self) -> StringStatement {
        match self {
            Self::Case { .. } | Self::Static { .. } => StringStatement::Bare,
            _ => StringStatement::Parenthesized,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum FollowingSyntax {
    Statement,
    ExpressionStatement {
        expression: Expr,
        string_statement: StringStatement,
        mode: ContentMode,
    },
    Keyword,
}

impl FollowingSyntax {
    fn starts_with_verbatim_angle(self, formatter: &Formatter<'_, '_>) -> bool {
        matches!(self, Self::ExpressionStatement { expression, mode: ContentMode::Verbatim, .. }
            if formatter.source(expression.span(formatter.ast)).starts_with('<'))
    }

    fn needs_asi_guard(self, formatter: &Formatter<'_, '_>) -> Result<bool, FormatError> {
        match self {
            Self::ExpressionStatement {
                expression,
                string_statement,
                mode,
            } => formatter.expression_needs_asi_guard(
                expression,
                ExpressionContext::at(ExpressionPosition::Statement(string_statement))
                    .with_content_mode(mode),
            ),
            Self::Statement | Self::Keyword => Ok(false),
        }
    }
}

/// An unresolved optional separator follows the terminal body into a compound statement.
#[derive(Clone, Copy)]
pub(super) enum ContinuationBoundary {
    End,
    Following {
        kind: FollowingSyntax,
        source: Option<StatementSourceTerminator>,
    },
}

/// The source owner that actually reaches a statement's right edge.
pub(in crate::formatter) enum StatementSourceTail {
    Expression(CopiedRawOwner),
    Declarator(VariableDeclarator),
    Statement { item: StatementItem, end: u32 },
}

impl StatementSourceTail {
    pub(in crate::formatter) fn expression(edge: &ExpressionRightEdge) -> Option<Self> {
        match *edge {
            ExpressionRightEdge::SourceFenceCandidate(raw) => Some(Self::Expression(raw)),
            ExpressionRightEdge::Closed | ExpressionRightEdge::Unreported => None,
        }
    }

    fn type_end(self, enclosing: Span, ast: &Ast<'_>) -> Option<(u32, RawTypeTerminal)> {
        Some(match self {
            Self::Expression(raw) => {
                let span = raw.span(ast);
                if enclosing.start > span.start || enclosing.end < span.end {
                    return None;
                }
                (span.end, raw.terminal(ast))
            }
            Self::Declarator(declarator) => {
                (enclosing.end, copied_declarator_terminal(declarator, ast))
            }
            Self::Statement { item, end } => (end, item.raw_type_terminal(ast)),
        })
    }
}

fn copied_declarator_terminal(declarator: VariableDeclarator, ast: &Ast<'_>) -> RawTypeTerminal {
    if let Some(initializer) = declarator.init(ast) {
        return crate::formatter::expressions::raw_tail::raw_expression_terminal(initializer, ast);
    }
    declarator
        .type_annotation(ast)
        .map_or(RawTypeTerminal::Closed, |annotation| {
            crate::formatter::types::raw_type_terminal(annotation.type_annotation(ast), ast)
        })
}

impl ContinuationBoundary {
    pub(super) fn for_child(self, span: Span) -> Self {
        match self {
            Self::End => Self::End,
            Self::Following { kind, source } => Self::Following {
                kind,
                source: source.filter(|candidate| {
                    span.start <= candidate.start()
                        && candidate.start().checked_add(1) == Some(span.end)
                }),
            },
        }
    }

    #[inline(always)]
    pub(super) fn after_source_tail(
        self,
        punctuation: StatementPunctuation,
        ast: &Ast<'_>,
        source_tail: impl FnOnce() -> Option<(Span, StatementSourceTail)>,
    ) -> StatementPunctuation {
        let (
            StatementPunctuation::Canonical(Semicolons::AsNeeded),
            Self::Following { kind, source },
        ) = (punctuation, self)
        else {
            return punctuation;
        };
        let Some((enclosing, tail)) = source_tail() else {
            return punctuation;
        };
        Self::punctuate_source_tail(kind, source, enclosing, tail, ast)
    }

    fn punctuate_source_tail(
        kind: FollowingSyntax,
        source: Option<StatementSourceTerminator>,
        enclosing: Span,
        tail: StatementSourceTail,
        ast: &Ast<'_>,
    ) -> StatementPunctuation {
        let punctuation = StatementPunctuation::Canonical(Semicolons::AsNeeded);
        let Some((content_end, terminal)) = tail.type_end(enclosing, ast) else {
            return punctuation;
        };
        if terminal == RawTypeTerminal::Closed {
            return punctuation;
        }
        if let Some(source) = source.filter(|source| content_end <= source.start()) {
            return StatementPunctuation::Source(source);
        }
        match (kind, terminal) {
            (FollowingSyntax::Keyword, RawTypeTerminal::PostfixQuestion) => punctuation,
            _ => StatementPunctuation::Canonical(Semicolons::Always),
        }
    }

    pub(super) fn after_unterminated_body(
        self,
        formatter: &Formatter<'_, '_>,
        punctuation: StatementPunctuation,
        statement: StatementItem,
    ) -> Result<StatementPunctuation, FormatError> {
        if let (
            StatementPunctuation::Canonical(Semicolons::AsNeeded),
            Self::Following { kind, .. },
        ) = (punctuation, self)
        {
            // A guard on the next head would become part of this copied branch.
            if kind.needs_asi_guard(formatter)? {
                return Ok(StatementPunctuation::Canonical(Semicolons::Always));
            }
            let span = statement.span(formatter.ast);
            return Ok(self
                .for_child(span)
                .after_source_tail(punctuation, formatter.ast, || {
                    Some((
                        span,
                        StatementSourceTail::Statement {
                            item: statement,
                            end: span.end,
                        },
                    ))
                }));
        }
        Ok(punctuation)
    }
}

pub(super) struct StatementGap<'a, 'ast> {
    pub(super) current: &'a PreparedStatement<'ast>,
    pub(super) next: Option<&'a PreparedStatement<'ast>>,
    pub(super) after_next: Option<StatementItem>,
    pub(super) sequence_end: u32,
    pub(super) kind: StatementSequenceKind<'ast>,
}

pub(super) struct StatementEnding {
    pub(super) punctuation: StatementPunctuation,
    pub(super) continuation: ContinuationBoundary,
    pub(super) source_packet: Option<StatementSourcePacket>,
}

impl<'ast> StatementGap<'_, 'ast> {
    fn following_syntax(&self, formatter: &Formatter<'ast, '_>) -> Option<FollowingSyntax> {
        if let Some(next) = self.next {
            if formatter.options.semicolons == Semicolons::AsNeeded
                && (formatter.ast.is_ts()
                    || formatter.has_suppression
                    || self.current.annotation_tail.is_some())
                && let StatementItem::Statement(node) = next.item
                && let StmtData::ExpressionStatement(node) = formatter.ast.stmt_data(node)
            {
                return Some(FollowingSyntax::ExpressionStatement {
                    expression: node.expression(formatter.ast),
                    string_statement: self.kind.string_statement(),
                    mode: self.following_mode(formatter, next),
                });
            }
            return Some(FollowingSyntax::Statement);
        }
        matches!(
            self.kind,
            StatementSequenceKind::Case {
                has_following: true,
                ..
            }
        )
        .then_some(FollowingSyntax::Keyword)
    }

    fn following_mode(
        &self,
        formatter: &Formatter<'ast, '_>,
        next: &PreparedStatement<'ast>,
    ) -> ContentMode {
        if next.has_verbatim_mode(self.current.annotation_tail.as_ref())
            || formatter.has_suppression && self.following_tail_is_verbatim(formatter, next)
        {
            ContentMode::Verbatim
        } else {
            ContentMode::Formatted
        }
    }

    fn following_tail_is_verbatim(
        &self,
        formatter: &Formatter<'ast, '_>,
        next: &PreparedStatement<'ast>,
    ) -> bool {
        let (limit, relation) = self.after_next.map_or(
            (self.sequence_end, StatementTailRelation::Terminal),
            |item| {
                (
                    item.span(formatter.ast).start,
                    StatementTailRelation::Statement,
                )
            },
        );
        formatter
            .prepare_statement_suppression_tail_at(next.item, next.span, limit, relation)
            .is_some_and(|tail| tail.modes.current == ContentMode::Verbatim)
    }

    fn retains_case_terminator(&self) -> bool {
        self.next.is_none()
            && matches!(
                self.kind,
                StatementSequenceKind::Case {
                    retain_last_terminator: true,
                    ..
                }
            )
    }

    #[inline(always)]
    pub(super) fn resolve(
        self,
        formatter: &Formatter<'ast, '_>,
        mode: ContentMode,
        source_line: StatementSourceLine,
        flat_source: bool,
        suppression: Option<&StatementSuppressionTail>,
        shift: &mut ShiftContinuation,
    ) -> Result<StatementEnding, FormatError> {
        // Without a source-owned separator, explicit semicolons need no lookahead.
        if formatter.options.semicolons == Semicolons::Always
            && !source_line.owns()
            && suppression.is_none()
            && self.current.annotation_tail.is_none()
            && !self.retains_case_terminator()
        {
            return Ok(StatementEnding {
                punctuation: StatementPunctuation::Canonical(Semicolons::Always),
                continuation: ContinuationBoundary::End,
                source_packet: None,
            });
        }
        self.resolve_source_or_asi(
            formatter,
            mode,
            source_line,
            flat_source,
            suppression,
            shift,
        )
    }

    fn resolve_source_or_asi(
        self,
        formatter: &Formatter<'ast, '_>,
        mode: ContentMode,
        source_line: StatementSourceLine,
        flat_source: bool,
        suppression: Option<&StatementSuppressionTail>,
        shift: &mut ShiftContinuation,
    ) -> Result<StatementEnding, FormatError> {
        let source = formatter.statement_source_terminator(self.current.span);
        let source_packet = source_line
            .outgoing
            .map(|span| StatementSourcePacket {
                span,
                following: LineConstraint::SameLine,
            })
            .or_else(|| suppression.and_then(|tail| tail.source_packet));
        let following = self.following_syntax(formatter);
        let punctuation = if source_line.owns()
            && (!flat_source || source_line.incoming || source_line.outgoing.is_some())
        {
            source.map_or(
                StatementPunctuation::Canonical(Semicolons::AsNeeded),
                StatementPunctuation::Source,
            )
        } else if self.retains_case_terminator()
            && let Some(source) = source
        {
            StatementPunctuation::Source(source)
        } else if source_packet.is_some() {
            StatementPunctuation::SourcePacket
        } else {
            let retained = if let Some(source) = source
                && (suppression.is_some_and(|tail| tail.terminator.is_some())
                    || self.retains_source_terminator(formatter, mode, following)?)
            {
                Some(source)
            } else {
                None
            };
            self.asi_punctuation(formatter, mode, following, retained, shift)?
        };
        let continuation = match (punctuation, following) {
            (StatementPunctuation::Canonical(Semicolons::AsNeeded), Some(kind)) => {
                let can_use_source = mode == ContentMode::Formatted
                    && (formatter.has_suppression
                        || self
                            .current
                            .item
                            .declaration(formatter.ast)
                            .is_some_and(|decl| {
                                matches!(
                                    formatter.ast.decl_data(decl),
                                    DeclData::VariableDeclaration(_)
                                )
                            }));
                ContinuationBoundary::Following {
                    kind,
                    source: source.filter(|_| can_use_source),
                }
            }
            _ => ContinuationBoundary::End,
        };
        Ok(StatementEnding {
            punctuation,
            continuation,
            source_packet,
        })
    }

    fn retains_source_terminator(
        &self,
        formatter: &Formatter<'ast, '_>,
        mode: ContentMode,
        following: Option<FollowingSyntax>,
    ) -> Result<bool, FormatError> {
        let as_needed = formatter.options.semicolons == Semicolons::AsNeeded;
        if let Some(tail) = &self.current.annotation_tail {
            // A guard after this packet would acquire a new empty-statement owner.
            if tail.suppression.current == ContentMode::Verbatim
                || tail.comments.iter().any(|&comment| {
                    comment.kind == CommentKind::Block
                        && formatter.comment_has_target_annotation(comment)
                })
                || tail.comments.first().is_some_and(|comment| {
                    formatter.source_line_breaks(self.current.span.end, comment.span.start) == 0
                })
            {
                return Ok(true);
            }
            if as_needed
                && let Some(kind) = following
                && kind.needs_asi_guard(formatter)?
            {
                return Ok(true);
            }
        }
        let Some(following) = following.filter(|_| as_needed) else {
            return Ok(false);
        };
        if mode == ContentMode::Verbatim
            && self.current.item.raw_type_terminal(formatter.ast) != RawTypeTerminal::Closed
        {
            return Ok(true);
        }
        // An ignored leading angle must keep its separator outside the copied source.
        Ok(following.starts_with_verbatim_angle(formatter)
            && !self.current.item.closes_type_declaration(formatter.ast))
    }

    fn asi_punctuation(
        &self,
        formatter: &Formatter<'ast, '_>,
        mode: ContentMode,
        following: Option<FollowingSyntax>,
        retained: Option<StatementSourceTerminator>,
        shift: &mut ShiftContinuation,
    ) -> Result<StatementPunctuation, FormatError> {
        let style = formatter.options.semicolons;
        if self
            .next
            .is_some_and(|next| next.retention == StatementRetention::BoundEmpty)
        {
            // The retained empty statement needs a semicolon of its own.
            return Ok(retained.map_or(
                StatementPunctuation::Canonical(Semicolons::Always),
                StatementPunctuation::Source,
            ));
        }
        let punctuation = retained.map_or(
            StatementPunctuation::Canonical(style),
            StatementPunctuation::Source,
        );
        if style == Semicolons::Always {
            return Ok(punctuation);
        }
        let ast = formatter.ast;
        let bare_let = retained.is_none()
            && self.next.is_some()
            && self.current.item.bare_identifier(ast) == Some("let");
        if bare_let {
            return Ok(StatementPunctuation::Canonical(Semicolons::Always));
        }
        if !ast.is_ts() {
            return Ok(punctuation);
        }
        let open_tail = retained.is_none()
            && (following.is_some() && self.current.item.has_open_type_tail(ast)
                || self.next.is_some() && shift.needs_terminator(self, formatter, mode));
        if open_tail || self.typescript_grammar_requires_terminator(formatter, following)? {
            Ok(StatementPunctuation::Canonical(Semicolons::Always))
        } else {
            Ok(punctuation)
        }
    }

    fn typescript_grammar_requires_terminator(
        &self,
        formatter: &Formatter<'ast, '_>,
        following: Option<FollowingSyntax>,
    ) -> Result<bool, FormatError> {
        let ast = formatter.ast;
        let previous = self.current.item;
        if let Some(StatementItem::Statement(next)) = self.next.map(|next| next.item)
            && matches!(ast.stmt_data(next), StmtData::BlockStatement(_))
        {
            return Ok(previous.needs_terminator_before_block(ast)
                || previous.bare_identifier(ast) == Some("global"));
        }
        let Some(
            head @ FollowingSyntax::ExpressionStatement {
                mut expression,
                string_statement,
                mode,
            },
        ) = following
        else {
            return Ok(false);
        };
        if let Some(declaration) = previous.declaration(ast)
            && let DeclData::TsImportEqualsDeclaration(node) = ast.decl_data(declaration)
            && let TsModuleReferenceData::TsTypeName(name) =
                ast.ts_module_reference_data(node.module_reference(ast))
            && let TsTypeNameData::IdentifierReference(name) = ast.ts_type_name_data(name)
            && ast.get_utf8(name.name(ast)) == "require"
        {
            return head.needs_asi_guard(formatter);
        }
        let mut context = ExpressionContext::at(ExpressionPosition::Statement(string_statement))
            .with_content_mode(mode);
        loop {
            let data = if context.is_verbatim() {
                ast.expr_data(expression)
            } else {
                formatter.expression_data(expression)
            };
            if matches!(data, ExprData::NewExpression(_)) {
                return Ok(previous
                    .tail_type(ast, true)
                    .is_some_and(|node| type_tail_is_bare_abstract(node, ast)));
            }
            let Some((child, child_context)) = formatter.expression_head_operand(data, context)?
            else {
                return Ok(false);
            };
            expression = child;
            context = child_context;
        }
    }
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn verbatim_statement_punctuation(
        &self,
        punctuation: StatementPunctuation,
        span: Span,
        content_end: u32,
        following: Option<(StatementItem, ContinuationBoundary)>,
    ) -> StatementPunctuation {
        let StatementPunctuation::Canonical(_) = punctuation else {
            return punctuation;
        };
        if self.has_suppression
            && let Some(source) = self.statement_source_terminator(span)
            && self.source_content_mode(Span::new(content_end, source.start()))
                == ContentMode::Verbatim
        {
            return StatementPunctuation::Source(source);
        }
        let Some((item, boundary)) = following else {
            return punctuation;
        };
        boundary
            .for_child(span)
            .after_source_tail(punctuation, self.ast, || {
                Some((
                    span,
                    StatementSourceTail::Statement {
                        item,
                        end: content_end,
                    },
                ))
            })
    }

    pub(super) fn punctuation_for_expression_output(
        &self,
        punctuation: StatementPunctuation,
        boundary: ContinuationBoundary,
        statement: Span,
        output: &ExpressionOutput<'ast>,
    ) -> StatementPunctuation {
        boundary.after_source_tail(punctuation, self.ast, || {
            StatementSourceTail::expression(&output.right_edge).map(|tail| (statement, tail))
        })
    }
}
