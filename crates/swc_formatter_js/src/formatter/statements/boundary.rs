//! Semicolon ownership between the current statement and the syntax that follows it.

use swc_next_ecma_ast::{
    CommentKind, DeclData, Expr, ExprData, GetSpan, Span, StmtData, TsModuleReferenceData,
    TsTypeNameData,
};

use super::tails::{StatementSourcePacket, StatementSuppressionTail, StatementTailRelation};
use super::{
    Formatter, PreparedStatement, StatementBoundary, StatementItem, StatementPunctuation,
    StatementSequenceKind, StatementSourceLine, StatementSourceTerminator,
};
use crate::formatter::diagnostics::LineConstraint;
use crate::formatter::expressions::{
    ExpressionContext, ExpressionOutput, ExpressionPosition, ExpressionRightEdge, ExpressionStart,
    StringStatement,
};
use crate::formatter::suppression::ContentMode;
use crate::formatter::types::{RawTypeTerminal, type_tail_is_bare_abstract};
use crate::{FormatError, Semicolons};

impl StatementBoundary {
    pub(super) fn expression_start(
        previous: Option<Self>,
        line: StatementSourceLine,
        semicolons: Semicolons,
    ) -> ExpressionStart {
        if semicolons == Semicolons::AsNeeded
            && !line.incoming
            && !previous.is_some_and(Self::guards_expression)
        {
            ExpressionStart::StatementPending
        } else {
            ExpressionStart::StatementGuarded
        }
    }

    pub(super) fn needs_same_line_terminator(
        self,
        formatter: &Formatter<'_, '_>,
        constraint: LineConstraint,
        next: StatementItem,
    ) -> bool {
        constraint == LineConstraint::SameLine
            && formatter.options.semicolons == Semicolons::AsNeeded
            && !self.has_terminator()
            && next.is_nonempty(formatter.ast)
    }
}

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

impl ContinuationBoundary {
    pub(super) fn is_following(self) -> bool {
        matches!(self, Self::Following { .. })
    }

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

    pub(super) fn after_raw_type(
        self,
        punctuation: StatementPunctuation,
        content_end: u32,
        terminal: RawTypeTerminal,
    ) -> StatementPunctuation {
        let (
            StatementPunctuation::Canonical(Semicolons::AsNeeded),
            Self::Following { kind, source },
        ) = (punctuation, self)
        else {
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
            return Ok(self.for_child(span).after_raw_type(
                punctuation,
                span.end,
                statement.raw_type_terminal(formatter.ast),
            ));
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
                    mode: if next.has_verbatim_mode(self.current.annotation_tail.as_ref())
                        || formatter.has_suppression
                            && self.following_tail_is_verbatim(formatter, next)
                    {
                        ContentMode::Verbatim
                    } else {
                        ContentMode::Formatted
                    },
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

    pub(super) fn resolve(
        self,
        formatter: &Formatter<'ast, '_>,
        mode: ContentMode,
        source_line: StatementSourceLine,
        flat_source: bool,
        suppression: Option<&StatementSuppressionTail>,
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
        } else if matches!(
            self.kind,
            StatementSequenceKind::Case {
                retain_last_terminator: true,
                ..
            }
        ) && self.next.is_none()
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
            self.asi_punctuation(formatter, following, retained)?
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
        following: Option<FollowingSyntax>,
        retained: Option<StatementSourceTerminator>,
    ) -> Result<StatementPunctuation, FormatError> {
        let style = formatter.options.semicolons;
        if self
            .next
            .is_some_and(|next| !next.empty_binding.allows_relocation(None))
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
        let statement = self.current.item;
        let open_tail = retained.is_none()
            && (following.is_some() && statement.has_open_type_tail(formatter.ast)
                || self.next.is_some() && statement.bare_identifier(formatter.ast) == Some("let"));
        if open_tail || self.grammar_requires_terminator(formatter, following)? {
            return Ok(StatementPunctuation::Canonical(Semicolons::Always));
        }
        Ok(punctuation)
    }

    fn grammar_requires_terminator(
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
                || ast.is_ts() && previous.bare_identifier(ast) == Some("global"));
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
        if !ast.is_ts() {
            return Ok(false);
        }
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
        let StatementPunctuation::Canonical(style) = punctuation else {
            return punctuation;
        };
        if self.has_suppression
            && let Some(source) = self.statement_source_terminator(span)
            && self.content_mode(
                self.comments
                    .in_range(Span::new(content_end, source.start())),
            ) == ContentMode::Verbatim
        {
            return StatementPunctuation::Source(source);
        }
        if style == Semicolons::AsNeeded
            && let Some((item, boundary)) = following
            && boundary.is_following()
        {
            return boundary.for_child(span).after_raw_type(
                punctuation,
                content_end,
                item.raw_type_terminal(self.ast),
            );
        }
        punctuation
    }

    pub(super) fn punctuation_for_expression_output(
        &self,
        punctuation: StatementPunctuation,
        boundary: ContinuationBoundary,
        statement: Span,
        output: &ExpressionOutput<'ast>,
    ) -> StatementPunctuation {
        if !matches!(
            punctuation,
            StatementPunctuation::Canonical(Semicolons::AsNeeded)
        ) || !boundary.is_following()
        {
            return punctuation;
        }
        let ExpressionRightEdge::SourceFenceCandidate(raw) = &output.right_edge else {
            return punctuation;
        };
        let span = raw.span(self.ast);
        if statement.start > span.start || span.end > statement.end {
            return punctuation;
        }
        boundary.after_raw_type(punctuation, span.end, raw.terminal(self.ast))
    }
}
