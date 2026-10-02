//! Member receivers and source-owned lookup boundaries.

use swc_next_ecma_ast::{
    Comment, CommentKind, Expr, ExprData, GetSpan, IdentifierName, MemberExpression,
    PrivateIdentifier, PropertyKeyData, Span,
};

use super::AccessKind;
use super::parentheses::ResolvedExpressionHead;
use super::{
    ConditionalClosing, ExpressionContext, ExpressionPosition, Formatter, MemberLayout,
    OperandIndent, TailOwner,
};
use crate::formatter::comments::{
    CommentBoundary, CommentGap, CommentRunLayout, ExpressionTail, NodeHead,
};
use crate::formatter::diagnostics::{FlatSeparator, LineConstraint};
use crate::formatter::suppression::{ContentMode, DelimitedFollowing, SuppressionPosition};
use crate::{FormatError, document::Writer};

#[derive(Clone, Copy)]
enum MemberKey {
    Static(StaticMemberKey),
    Computed(Expr),
}

#[derive(Clone, Copy)]
enum StaticMemberKey {
    Public(IdentifierName),
    Private(PrivateIdentifier),
}

enum MemberPunctuation {
    Static {
        key: StaticMemberKey,
        operator: Span,
        key_gap: Option<Span>,
    },
    Computed {
        key: Expr,
        optional: Option<Span>,
        brackets: Span,
        opening: LineConstraint,
    },
}

impl MemberPunctuation {
    fn operator(&self) -> Span {
        match *self {
            Self::Static { operator, .. } => operator,
            Self::Computed {
                optional: Some(operator),
                ..
            } => operator,
            Self::Computed {
                optional: None,
                brackets,
                ..
            } => Span::new(brackets.start, brackets.start + 1),
        }
    }
}

enum MemberContent {
    Verbatim,
    Formatted {
        receiver: ContentMode,
        key: ContentMode,
    },
}

pub(super) enum PreparedMember {
    Verbatim(MemberExpression),
    Formatted(SourceMember),
}

/// The actual member's key and receiver facts, before punctuation or modes.
pub(super) struct MemberHead {
    node: MemberExpression,
    pub(super) object: Expr,
    property_span: Span,
    gap: Span,
    key: MemberKey,
    access: AccessKind,
}

impl MemberHead {
    pub(super) fn is_computed(&self) -> bool {
        matches!(self.key, MemberKey::Computed(_))
    }

    pub(super) fn key(&self) -> Option<Expr> {
        match self.key {
            MemberKey::Computed(key) => Some(key),
            MemberKey::Static(_) => None,
        }
    }

    pub(super) fn name(&self) -> Option<IdentifierName> {
        match self.key {
            MemberKey::Static(StaticMemberKey::Public(name)) => Some(name),
            MemberKey::Static(StaticMemberKey::Private(_)) | MemberKey::Computed(_) => None,
        }
    }

    /// Used after the source-clear query has established formatted content.
    pub(super) fn receiver_context(&self, context: ExpressionContext) -> ExpressionContext {
        member_receiver_context(
            context,
            self.access,
            ContentMode::Formatted,
            self.is_computed(),
            self.object,
        )
    }
}

fn member_receiver_context(
    context: ExpressionContext,
    access: AccessKind,
    mode: ContentMode,
    computed: bool,
    object: Expr,
) -> ExpressionContext {
    let position = if matches!(
        context.position,
        ExpressionPosition::AssignmentTarget | ExpressionPosition::MemberAssignmentObject
    ) {
        ExpressionPosition::MemberAssignmentObject
    } else {
        ExpressionPosition::MemberObject(access)
    };
    let mut receiver = ExpressionContext::at(position)
        .with_parent_head(context)
        .with_tail_owner(TailOwner::ParentOperation)
        .with_content_mode(mode);
    receiver.member_layout = context.member_layout;
    receiver.member_chain_layout = context.member_chain_layout;
    receiver.conditional_envelope = context.conditional_envelope.chain_head();
    receiver.conditional_closing = if computed {
        ConditionalClosing::Inline
    } else {
        ConditionalClosing::MemberLine
    };
    if computed
        && access == AccessKind::Ordinary
        && context.expression_head == super::ExpressionHead::Statement
    {
        receiver.computed_receiver = Some(object);
    }
    receiver
}

pub(super) struct SourceMember {
    pub(super) object: Expr,
    property_span: Span,
    punctuation: MemberPunctuation,
    pub(super) access: AccessKind,
    pub(super) receiver_mode: ContentMode,
    key_mode: ContentMode,
}

pub(super) struct MemberBoundary<'ast> {
    before: CommentBoundary<'ast>,
    after: &'ast [Comment],
    pub(super) owns_lookup_comments: bool,
    pub(super) empty_line: bool,
    pub(super) has_comments: bool,
}

impl MemberBoundary<'_> {
    pub(super) fn has_before_comments(&self) -> bool {
        !self.before.is_empty()
    }

    pub(super) fn gap(&self) -> CommentGap {
        self.before.gap()
    }
}

#[derive(Clone, Copy)]
pub(super) enum MemberLookupOwner {
    InlineContext,
    Receiver,
    Rhs,
    Expression,
}
impl MemberLookupOwner {
    pub(super) fn from_context(context: ExpressionContext) -> Self {
        if matches!(context.position, ExpressionPosition::MemberAssignmentObject)
            || context.member_layout == MemberLayout::Tight
        {
            Self::InlineContext
        } else if matches!(context.position, ExpressionPosition::MemberObject(_)) {
            Self::Receiver
        } else if context.member_layout == MemberLayout::Rhs {
            Self::Rhs
        } else {
            Self::Expression
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum MemberLookupLayout {
    Inline,
    Grouped,
}

impl MemberLookupLayout {
    pub(super) fn write<'ast>(
        self,
        gap: CommentGap,
        writer: &mut Writer<'ast>,
        access: impl FnOnce(&mut Writer<'ast>) -> Result<(), FormatError>,
    ) -> Result<(), FormatError> {
        // The caller printed a receiver-owned suffix outside this lookup.
        // Its mandatory break has the receiver's indentation; only later
        // lookup continuation breaks belong to the lookup's indent scope.
        if matches!(gap, CommentGap::LineComment) {
            writer.hard_line();
            return self.write_contents(writer, access);
        }
        self.write_contents(writer, |writer| {
            gap.write(writer, FlatSeparator::Empty, |writer| {
                if matches!(self, Self::Grouped) {
                    writer.soft_line();
                }
            });
            access(writer)
        })
    }

    pub(super) fn write_contents<'ast>(
        self,
        writer: &mut Writer<'ast>,
        write: impl FnOnce(&mut Writer<'ast>) -> Result<(), FormatError>,
    ) -> Result<(), FormatError> {
        match self {
            Self::Inline => write(writer),
            Self::Grouped => writer.group(false, |writer| writer.indent(write)),
        }
    }
}

impl SourceMember {
    pub(super) fn operator_span(&self) -> Span {
        self.punctuation.operator()
    }

    pub(super) fn is_computed(&self) -> bool {
        matches!(self.punctuation, MemberPunctuation::Computed { .. })
    }

    pub(super) fn key(&self) -> Option<Expr> {
        match self.punctuation {
            MemberPunctuation::Computed { key, .. } => Some(key),
            MemberPunctuation::Static { .. } => None,
        }
    }

    pub(super) fn name(&self) -> Option<IdentifierName> {
        match self.punctuation {
            MemberPunctuation::Static {
                key: StaticMemberKey::Public(key),
                ..
            } => Some(key),
            MemberPunctuation::Static {
                key: StaticMemberKey::Private(_),
                ..
            } => None,
            MemberPunctuation::Computed { .. } => None,
        }
    }

    pub(super) fn receiver_context(&self, context: ExpressionContext) -> ExpressionContext {
        member_receiver_context(
            context,
            self.access,
            self.receiver_mode,
            self.is_computed(),
            self.object,
        )
    }
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn source_member_receiver(
        &self,
        member: MemberExpression,
        context: ExpressionContext,
    ) -> Result<(Expr, ExpressionContext), FormatError> {
        // Only suppression can change the receiver or preserve the whole member.
        if self.has_suppression {
            return Ok(match self.source_member(member)? {
                PreparedMember::Verbatim(member) => (
                    Expr::MemberExpression(member),
                    context.with_content_mode(ContentMode::Verbatim),
                ),
                PreparedMember::Formatted(source) => {
                    (source.object, source.receiver_context(context))
                }
            });
        }
        let receiver = member.object(self.ast);
        let context = member_receiver_context(
            context,
            if member.optional(self.ast) {
                AccessKind::Optional
            } else {
                AccessKind::Ordinary
            },
            ContentMode::Formatted,
            member.computed(self.ast),
            receiver,
        );
        Ok((receiver, context))
    }

    pub(super) fn format_member(
        &mut self,
        member: MemberExpression,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let PreparedMember::Formatted(member) = self.source_member(member)? else {
            return self.format_expression(
                Expr::MemberExpression(member),
                context.with_content_mode(ContentMode::Verbatim),
                writer,
            );
        };
        let tail =
            self.format_expression(member.object, member.receiver_context(context), writer)?;
        let boundary = self.prepare_member_boundary(&member, tail)?;
        let layout =
            self.member_lookup_layout(&member, &boundary, MemberLookupOwner::from_context(context));
        if boundary.owns_lookup_comments {
            layout.write_contents(writer, |writer| {
                self.format_member_trailing(&boundary, writer)?;
                boundary
                    .gap()
                    .write(writer, FlatSeparator::Empty, Writer::soft_line);
                self.format_member_access(&member, boundary, writer)
            })?;
        } else {
            self.format_member_trailing(&boundary, writer)?;
            layout.write(boundary.gap(), writer, |writer| {
                self.format_member_access(&member, boundary, writer)
            })?;
        }
        Ok(None)
    }

    pub(super) fn member_lookup_layout(
        &self,
        member: &SourceMember,
        boundary: &MemberBoundary<'ast>,
        owner: MemberLookupOwner,
    ) -> MemberLookupLayout {
        if boundary.owns_lookup_comments {
            return MemberLookupLayout::Grouped;
        }
        if matches!(owner, MemberLookupOwner::InlineContext) || member.is_computed() {
            return MemberLookupLayout::Inline;
        }
        if matches!(owner, MemberLookupOwner::Receiver) {
            return MemberLookupLayout::Grouped;
        }
        let receiver = self.expression_content(member.object).data;
        if matches!(receiver, ExprData::IdentifierReference(_)) {
            return MemberLookupLayout::Inline;
        }
        // The enclosing RHS fits the complete call and its terminal lookup.
        // Keep the call's arguments as the available internal break points.
        if matches!(owner, MemberLookupOwner::Rhs)
            && member.receiver_mode == ContentMode::Formatted
            && matches!(self.chain_element_data(receiver), ExprData::CallExpression(call)
                if !call.arguments(self.ast).is_empty())
        {
            return MemberLookupLayout::Inline;
        }
        MemberLookupLayout::Grouped
    }

    pub(super) fn prepare_member_head(
        &self,
        member: MemberExpression,
    ) -> Result<MemberHead, FormatError> {
        let access = if member.optional(self.ast) {
            AccessKind::Optional
        } else {
            AccessKind::Ordinary
        };
        let property = member.property(self.ast);
        let key = match (
            member.computed(self.ast),
            self.ast.property_key_data(property),
        ) {
            (false, PropertyKeyData::IdentifierName(identifier)) => {
                MemberKey::Static(StaticMemberKey::Public(identifier))
            }
            (false, PropertyKeyData::Expr(expression)) => match self.ast.expr_data(expression) {
                ExprData::PrivateIdentifier(identifier) => {
                    MemberKey::Static(StaticMemberKey::Private(identifier))
                }
                _ => return Err(self.unsupported("member property", property)),
            },
            (true, PropertyKeyData::Expr(expression)) => MemberKey::Computed(expression),
            _ => return Err(self.unsupported("member property", property)),
        };
        let object = member.object(self.ast);
        let object_span = object.span(self.ast);
        let property_span = property.span(self.ast);
        let gap = Span::new(object_span.end, property_span.start);
        Ok(MemberHead {
            node: member,
            object,
            property_span,
            gap,
            key,
            access,
        })
    }

    /// Callers have proved the enclosing immutable comment and diagnostic span.
    /// Actual source lines or suppression retain the complete punctuation owner.
    pub(super) fn source_clear_member_has_chain_barrier(
        &self,
        head: &MemberHead,
    ) -> Result<bool, FormatError> {
        if !self.has_suppression && self.line_breaks(head.gap.start, head.gap.end) == 0 {
            return Ok(false);
        }
        let PreparedMember::Formatted(source) = self.prepare_source_member(head)? else {
            return Ok(true);
        };
        Ok(source.receiver_mode != ContentMode::Formatted
            || self.next_line_is_empty(head.gap.start, source.operator_span().start))
    }

    pub(super) fn source_member(
        &self,
        member: MemberExpression,
    ) -> Result<PreparedMember, FormatError> {
        let head = self.prepare_member_head(member)?;
        self.prepare_source_member(&head)
    }

    fn prepare_source_member(&self, head: &MemberHead) -> Result<PreparedMember, FormatError> {
        let member = head.node;
        let object = head.object;
        let property_span = head.property_span;
        let gap = head.gap;
        let key = head.key;
        let access = head.access;
        let comments = self.comments.in_range(gap);
        let mut punctuation = match key {
            MemberKey::Computed(expression) => {
                let optional = if access == AccessKind::Optional {
                    let start = self
                        .find_token(gap, comments, "?.")
                        .ok_or_else(|| self.unsupported("member optional boundary", member))?;
                    Some(Span::new(start, start + 2))
                } else {
                    None
                };
                let opening_gap = Span::new(optional.map_or(gap.start, |span| span.end), gap.end);
                let opening = self
                    .find_token(opening_gap, comments, "[")
                    .ok_or_else(|| self.unsupported("member opening boundary", member))?;
                MemberPunctuation::Computed {
                    key: expression,
                    optional,
                    brackets: Span::new(opening, member.span(self.ast).end),
                    opening: LineConstraint::Unconstrained,
                }
            }
            MemberKey::Static(identifier) => {
                let token = if access == AccessKind::Optional {
                    "?."
                } else {
                    "."
                };
                let start = self
                    .find_token(gap, comments, token)
                    .ok_or_else(|| self.unsupported("member operator boundary", member))?;
                MemberPunctuation::Static {
                    key: identifier,
                    operator: Span::new(start, start + token.len() as u32),
                    key_gap: (self.has_suppression
                        && self.content_mode(
                            &comments[comments.partition_point(|comment| {
                                comment.span.end <= start + token.len() as u32
                            })..],
                        ) == ContentMode::Verbatim)
                        .then_some(Span::new(start + token.len() as u32, property_span.start)),
                }
            }
        };
        let key_boundary = match punctuation {
            MemberPunctuation::Computed { brackets, .. } => brackets.start,
            MemberPunctuation::Static { operator, .. } => operator.start,
        };
        let content_comments = if self.has_suppression {
            self.member_receiver_comments(object, property_span.start, comments)
        } else {
            comments
        };
        let content = self.member_content(key, key_boundary, property_span.start, content_comments);
        let MemberContent::Formatted { receiver, mut key } = content else {
            return Ok(PreparedMember::Verbatim(member));
        };
        if self.has_suppression
            && let MemberPunctuation::Computed {
                brackets, opening, ..
            } = &mut punctuation
        {
            let after_opening = &comments
                [comments.partition_point(|comment| comment.span.end <= brackets.start + 1)..];
            if self.content_mode(after_opening) == ContentMode::Verbatim {
                let first = after_opening
                    .first()
                    .map_or(property_span.start, |comment| comment.span.start);
                *opening = self.source_line_constraint(brackets.start + 1, first);
            }
            if key != ContentMode::Verbatim {
                key = self
                    .trailing_content_mode(Span::new(property_span.end, brackets.end - 1), true);
            }
        }
        Ok(PreparedMember::Formatted(SourceMember {
            object,
            property_span,
            punctuation,
            access,
            receiver_mode: receiver,
            key_mode: key,
        }))
    }

    /// The same source packet decides lookup ownership and later relocation.
    pub(in crate::formatter) fn member_lookup_comments(
        &self,
        comments: &'ast [Comment],
        gap: Span,
        operator: u32,
        computed: bool,
    ) -> &'ast [Comment] {
        let split = comments.partition_point(|comment| comment.span.end <= operator);
        let move_lines = !computed
            && comments[split..]
                .iter()
                .any(|comment| comment.kind == CommentKind::Line)
            && self
                .comment_binding(comments, gap, None)
                .allows_relocation(None);
        &comments[..if move_lines { comments.len() } else { split }]
    }

    pub(super) fn prepare_member_boundary(
        &self,
        member: &SourceMember,
        tail: Option<ExpressionTail<'ast>>,
    ) -> Result<MemberBoundary<'ast>, FormatError> {
        let operator = member.operator_span();
        let computed = member.is_computed();
        let object_end = member.object.span(self.ast).end;
        let source_end = tail.as_ref().map_or(object_end, |tail| tail.source_end);
        let comments = self
            .comments
            .between(Span::new(source_end, member.property_span.start))?;
        let before = self.member_lookup_comments(
            comments,
            Span::new(source_end, member.property_span.start),
            operator.start,
            computed,
        );
        let owns_lookup_comments = !computed && {
            // A receiver tail extends the packet before the member's native gap.
            // Lookup ownership still comes from that original gap.
            let lookup_comments = if source_end == object_end {
                before
            } else {
                let first = comments.partition_point(|comment| comment.span.start < object_end);
                self.member_lookup_comments(
                    &comments[first..],
                    Span::new(object_end, member.property_span.start),
                    operator.start,
                    false,
                )
            };
            lookup_comments
                .iter()
                .any(|comment| self.source_position_starts_line(comment.span.start))
        };
        let after = &comments[before.len()..];
        let move_lines = before
            .iter()
            .any(|comment| comment.span.start >= operator.end);
        let next_start = if move_lines {
            member.property_span.start
        } else {
            operator.start
        };
        let before = self.prepare_comment_boundary(before, source_end, next_start);
        let empty_line = self.next_line_is_empty(before.end(), next_start);
        Ok(MemberBoundary {
            before,
            after,
            owns_lookup_comments,
            empty_line,
            has_comments: !comments.is_empty(),
        })
    }

    pub(super) fn format_member_trailing(
        &mut self,
        boundary: &MemberBoundary<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        self.format_comment_boundary(&boundary.before, writer)
    }

    pub(super) fn format_member_access(
        &mut self,
        member: &SourceMember,
        boundary: MemberBoundary<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let after = boundary.after;
        match &member.punctuation {
            MemberPunctuation::Computed {
                key: expression,
                optional,
                brackets,
                opening,
            } => {
                if let Some(optional) = optional {
                    writer.text("?.");
                    let before_bracket_count =
                        after.partition_point(|comment| comment.span.end <= brackets.start);
                    let before_bracket = &after[..before_bracket_count];
                    let boundary =
                        self.prepare_comment_boundary(before_bracket, optional.end, brackets.start);
                    let separator = if boundary.is_empty() {
                        FlatSeparator::Empty
                    } else {
                        FlatSeparator::Space
                    };
                    self.format_comment_boundary(&boundary, writer)?;
                    boundary.gap().write(writer, separator, |writer| {
                        if !boundary.is_empty() {
                            writer.space();
                        }
                    });
                }
                self.format_member_key(*expression, *brackets, member.key_mode, *opening, writer)
            }
            MemberPunctuation::Static {
                key: identifier,
                operator,
                key_gap,
            } => {
                writer.text(if member.access == AccessKind::Optional {
                    "?."
                } else {
                    "."
                });
                if let Some(gap) = key_gap {
                    let source =
                        self.format_delimited_source_gap(*gap, DelimitedFollowing::Node, writer)?;
                    source.following.write(writer, FlatSeparator::Empty, |_| {});
                } else {
                    self.comments.claim(after)?;
                    let next = after
                        .first()
                        .map_or(member.property_span.start, |comment| comment.span.start);
                    self.line_constraint(operator.end, next).write(
                        writer,
                        FlatSeparator::Empty,
                        |_| {},
                    );
                    // The dot and key share one tight run; source-only breaks must
                    // not change a block comment's layout on the next pass.
                    self.format_comment_run(
                        after,
                        CommentRunLayout::FlatLeading(member.property_span.start),
                        writer,
                    )?;
                }
                if member.key_mode == ContentMode::Verbatim {
                    self.format_verbatim(member.property_span, writer)?;
                } else {
                    match identifier {
                        StaticMemberKey::Public(identifier) => {
                            writer.text(self.ast.get_utf8(identifier.name(self.ast)))
                        }
                        StaticMemberKey::Private(identifier) => {
                            writer.text("#");
                            writer.text(self.ast.get_utf8(identifier.name(self.ast)));
                        }
                    }
                }
                Ok(())
            }
        }
    }

    /// Parentheses do not make a new member comment owner. Include only trivia
    /// after the actual receiver operand, keeping its internal owners separate.
    fn member_receiver_comments(
        &self,
        object: Expr,
        end: u32,
        outside: &'ast [Comment],
    ) -> &'ast [Comment] {
        let ExprData::ParenthesizedExpression(parenthesized) = self.ast.expr_data(object) else {
            return outside;
        };
        let mut operand = parenthesized.expression(self.ast);
        while let ExprData::ParenthesizedExpression(parenthesized) = self.ast.expr_data(operand) {
            operand = parenthesized.expression(self.ast);
        }
        self.comments
            .in_range(Span::new(operand.span(self.ast).end, end))
    }

    fn member_content(
        &self,
        key: MemberKey,
        key_boundary: u32,
        key_start: u32,
        comments: &[Comment],
    ) -> MemberContent {
        let mut receiver = ContentMode::Formatted;
        let mut key_mode = ContentMode::Formatted;
        if !self.has_suppression {
            return MemberContent::Formatted {
                receiver,
                key: key_mode,
            };
        }
        if let MemberKey::Static(StaticMemberKey::Private(private)) = key {
            let mut receiver = ContentMode::Formatted;
            let mut key = ContentMode::Formatted;
            let split = comments.partition_point(|comment| comment.span.end <= key_boundary);
            for position in self.suppression_positions(&comments[..split], key_boundary) {
                match position {
                    crate::formatter::suppression::SuppressionPosition::OwnLine => {
                        key = ContentMode::Verbatim
                    }
                    _ => receiver = ContentMode::Verbatim,
                }
            }
            for position in
                self.suppression_positions(&comments[split..], private.span(self.ast).start)
            {
                match position {
                    crate::formatter::suppression::SuppressionPosition::EndOfLine => {
                        receiver = ContentMode::Verbatim
                    }
                    _ => key = ContentMode::Verbatim,
                }
            }
            return MemberContent::Formatted { receiver, key };
        }
        for (run, position) in self.suppressed_comment_runs(comments, key_start) {
            match position {
                SuppressionPosition::OwnLine => {
                    let identifier = match key {
                        MemberKey::Static(_) => true,
                        MemberKey::Computed(expression) => {
                            self.member_key_is_identifier(expression)
                        }
                    };
                    if identifier {
                        return MemberContent::Verbatim;
                    }
                    key_mode = ContentMode::Verbatim;
                }
                SuppressionPosition::EndOfLine => receiver = ContentMode::Verbatim,
                SuppressionPosition::Inline => {
                    if run
                        .last()
                        .is_some_and(|comment| comment.span.end <= key_boundary)
                    {
                        receiver = ContentMode::Verbatim;
                    } else {
                        key_mode = ContentMode::Verbatim;
                    }
                }
            }
        }
        MemberContent::Formatted {
            receiver,
            key: key_mode,
        }
    }

    fn member_key_is_identifier(&self, mut expression: Expr) -> bool {
        loop {
            let content = self.expression_content(expression);
            let ExprData::ParenthesizedExpression(parenthesized) = content.data else {
                return matches!(content.data, ExprData::IdentifierReference(_));
            };
            if self.parenthesis_has_type_cast(parenthesized.span(self.ast).start) {
                return false;
            }
            let Some(child) = self.ignored_parentheses_child(parenthesized, ContentMode::Formatted)
            else {
                return false;
            };
            expression = child;
        }
    }

    fn format_member_key(
        &mut self,
        expression: Expr,
        span: Span,
        mode: ContentMode,
        opening: LineConstraint,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let expression_span = expression.span(self.ast);
        let leading = self
            .comments
            .between(Span::new(span.start + 1, expression_span.start))?;
        let compact = matches!(self.ast.expr_data(expression), ExprData::NumericLiteral(_));
        writer.group(false, |writer| {
            writer.text("[");
            let (end, line_comment) = writer.indent(|writer| {
                let first = leading
                    .first()
                    .map_or(expression_span.start, |comment| comment.span.start);
                let opening = if opening == LineConstraint::Unconstrained {
                    self.line_constraint(span.start + 1, first)
                } else {
                    opening
                };
                opening.write(writer, FlatSeparator::Empty, |writer| {
                    if !compact {
                        writer.soft_line();
                    }
                });
                let content = self.expression_content(expression);
                let binding = self.expression_head_binding(leading, expression);
                let binding = content
                    .parentheses
                    .map_or(binding, |removed| binding.combine(removed.head.binding));
                let operand_indent = if mode == ContentMode::Formatted
                    && matches!(content.data, ExprData::ConditionalExpression(_))
                {
                    OperandIndent::Parent
                } else {
                    OperandIndent::Expression
                };
                self.comments.claim(leading)?;
                let tail = self.format_prepared_expression_with_resolved_head(
                    expression,
                    content,
                    ResolvedExpressionHead {
                        head: NodeHead {
                            comments: leading,
                            content_start: expression_span.start,
                            layout: CommentRunLayout::GroupedLeading(expression_span.start),
                            gap_end: None,
                        },
                        binding,
                    },
                    ExpressionContext::at(ExpressionPosition::Nested)
                        .with_operand_indent(operand_indent)
                        .with_tail_owner(TailOwner::ParentOperation)
                        .with_content_mode(mode),
                    writer,
                )?;
                let end = tail
                    .as_ref()
                    .map_or(expression_span.end, |tail| tail.source_end);
                let trailing = self.comments.between(Span::new(end, span.end - 1))?;
                self.comments.claim(trailing)?;
                self.format_comment_run(trailing, CommentRunLayout::SourceAfter(end), writer)?;
                Ok::<_, FormatError>((
                    trailing.last().map_or(end, |comment| comment.span.end),
                    trailing
                        .last()
                        .is_some_and(|comment| comment.kind == CommentKind::Line),
                ))
            })?;
            if line_comment {
                writer.hard_line();
            } else {
                self.line_constraint(end, span.end - 1).write(
                    writer,
                    FlatSeparator::Empty,
                    |writer| {
                        if !compact {
                            writer.soft_line();
                        }
                    },
                );
            }
            writer.text("]");
            Ok(())
        })
    }
}
