//! Shared fluent layout over source-owned member, call, and postfix operations.

use std::iter::{Peekable, Take};
use std::vec::IntoIter;

use swc_next_ecma_ast::{
    CallExpression, Expr, ExprData, GetSpan, IdentifierName, ParenthesizedExpression, Span,
};

use super::arguments::{ArgumentGroup, SourceArguments};
use super::members::{
    MemberBoundary, MemberHead, MemberLookupLayout, MemberLookupOwner, PreparedMember, SourceMember,
};
use super::parentheses::{
    ExpressionContent, ParenthesisInner, RemovedExpressionParentheses, ResolvedExpressionHead,
};
use super::typescript::SourceNonNull;
use super::{
    AccessKind, ExpressionContext, ExpressionLayout, ExpressionPosition, ExpressionStart,
    Formatter, MemberChainLayout, MemberLayout,
};
use crate::formatter::comments::{
    CommentBinding, CommentGap, CommentRunLayout, ExpressionTail, NodeHead,
};
use crate::formatter::diagnostics::FlatSeparator;
use crate::formatter::suppression::ContentMode;
use crate::{
    FormatError,
    document::{Fragment, NeutralFragment, Writer},
};

pub(super) enum FluentChainEligibility {
    Fluent,
    MemberOnly,
    Unqualified,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum RhsCallChainOwnership {
    Compact,
    Fluent,
}

/// Reverse summaries of the head printer's two incoming `leading` states.
/// Only source-clear chains can use topology in place of recorded boundary facts.
#[derive(Clone, Copy, Default)]
struct ChainHeadPrefix {
    parts: usize,
    last_name: Option<IdentifierName>,
    pending_computed: bool,
}

impl ChainHeadPrefix {
    fn prepend(self, name: Option<IdentifierName>) -> Self {
        Self {
            parts: self.parts + 1,
            last_name: if self.parts == 0 {
                name
            } else {
                self.last_name
            },
            pending_computed: self.pending_computed,
        }
    }

    fn pending(computed: bool) -> Self {
        Self {
            pending_computed: computed,
            ..Self::default()
        }
    }
}

#[derive(Default)]
struct RhsChainHead {
    leading: ChainHeadPrefix,
    following: ChainHeadPrefix,
    next_is_member: bool,
}

impl RhsChainHead {
    fn call(&mut self) {
        self.leading = self.leading.prepend(None);
        self.following = self.following.prepend(None);
        self.next_is_member = false;
    }

    fn member(&mut self, member: &MemberHead, numeric: bool) {
        let following = if self.next_is_member {
            self.following.prepend(member.name())
        } else {
            ChainHeadPrefix::pending(member.is_computed())
        };
        self.leading = if numeric {
            self.leading.prepend(member.name())
        } else {
            following
        };
        self.following = following;
        self.next_is_member = true;
    }

    fn non_null(&mut self) {
        self.next_is_member = false;
    }
}

pub(super) fn atomic_chain_head(data: ExprData) -> bool {
    matches!(
        data,
        ExprData::IdentifierReference(_)
            | ExprData::ThisExpression(_)
            | ExprData::MetaProperty(_)
            | ExprData::StringLiteral(_)
            | ExprData::NumericLiteral(_)
            | ExprData::BigIntLiteral(_)
            | ExprData::BooleanLiteral(_)
            | ExprData::NullLiteral(_)
            | ExprData::RegExpLiteral(_)
    )
}

struct ChainMember {
    source: SourceMember,
    lookup_owner: MemberLookupOwner,
    numeric: bool,
    next_part_is_member: bool,
}

enum ChainPart<'ast> {
    Member(ChainMember),
    Call {
        call: CallExpression,
        arguments: SourceArguments<'ast>,
    },
    NonNull(SourceNonNull<'ast>),
    Parentheses(RemovedExpressionParentheses),
}

pub(in crate::formatter) struct PreparedChain<'ast> {
    expression: Expr,
    context: ExpressionContext,
    head: Expr,
    head_context: ExpressionContext,
    cast_head: Option<ParenthesizedExpression>,
    prepared_head: Option<PreparedChainHead<'ast>>,
    operation_head: Option<(CallExpression, NodeHead<'ast>)>,
    parts: Vec<ChainPart<'ast>>,
    // The prepared operation boundary between the callee and terminal lookups.
    callee_parts: usize,
    calls: usize,
    // Source boundaries can add groups and independently require expansion.
    groups: usize,
}

struct PreparedChainHead<'ast> {
    content: ExpressionContent,
    resolved: ResolvedExpressionHead<'ast>,
    leading_owner: ChainLeadingOwner,
    has_comments: bool,
}

enum ChainLeadingOwner {
    Chain,
    Head,
}

impl PreparedChain<'_> {
    pub(in crate::formatter) fn has_call_operation_head(&self) -> bool {
        self.operation_head.is_some()
    }

    /// Call/member descendants already own their grammar contexts. A resolved
    /// operation head changes only the complete chain's enclosing frame.
    pub(in crate::formatter) fn set_enclosing_context(&mut self, context: ExpressionContext) {
        self.context = context;
    }

    pub(in crate::formatter) fn set_leading_comment_layout(&mut self, layout: CommentRunLayout) {
        if let Some(head) = self.prepared_head.as_mut()
            && matches!(head.leading_owner, ChainLeadingOwner::Chain)
        {
            head.resolved.head.layout = layout;
        }
    }

    pub(super) fn set_expression_start(&mut self, start: ExpressionStart) {
        self.context.expression_start = start;
        self.head_context.expression_start = start;
    }

    pub(super) fn emitted_head(&self) -> (Expr, ExpressionContext) {
        let expression = self
            .prepared_head
            .as_ref()
            .map_or(self.head, |head| head.content.expression);
        (expression, self.head_context)
    }
}

enum ChainSuffix {
    Collecting { parts: usize },
    Complete { parts: usize },
}

enum ChainLookupLayout {
    Local,
    InlineContinuation,
    ExpressionContinuation,
}

struct ChainGrouping {
    groups: usize,
    pending_call: bool,
}

impl ChainGrouping {
    fn new() -> Self {
        Self {
            groups: 1,
            pending_call: false,
        }
    }

    fn call(&mut self) {
        self.pending_call = true;
    }

    fn member(&mut self, numeric: bool) {
        // The reverse walk closes the same call-bearing groups as the forward
        // printer. Numeric accesses stay with a call; terminal lookup groups
        // are owned separately. The remaining prefix is head.
        if !numeric {
            if self.pending_call {
                self.groups += 1;
            }
            self.pending_call = false;
        }
    }
}

struct ChainHead<'ast> {
    expression: Expr,
    context: ExpressionContext,
    has_comments: bool,
    attach: bool,
    prepared: Option<PreparedChainHead<'ast>>,
}

struct PendingMember<'ast> {
    member: ChainMember,
    boundary: MemberBoundary<'ast>,
}

impl PendingMember<'_> {
    fn separator(&self) -> ChainSeparator {
        ChainSeparator {
            gap: self.boundary.gap(),
            empty_line: self.boundary.empty_line,
        }
    }

    fn source_break(&self) -> bool {
        self.boundary.has_comments
            || self.boundary.empty_line
            || self.boundary.gap().requires_line()
    }
}

enum ChainOperation<'ast> {
    Member(PendingMember<'ast>),
    Call {
        call: CallExpression,
        arguments: SourceArguments<'ast>,
    },
    NonNull(SourceNonNull<'ast>),
}

enum ChainArgumentLayout {
    ByWidth,
    Inspect,
    Complex,
}

struct ChainSource<'ast, 'parts> {
    parts: Peekable<Take<&'parts mut IntoIter<ChainPart<'ast>>>>,
    pending: Option<PendingMember<'ast>>,
    tail: Option<ExpressionTail<'ast>>,
    operation_head: Option<(CallExpression, NodeHead<'ast>)>,
    argument_layout: ChainArgumentLayout,
}

#[derive(Clone, Copy)]
struct ChainSeparator {
    gap: CommentGap,
    empty_line: bool,
}

impl ChainSeparator {
    fn write(self, expanded: bool, writer: &mut Writer<'_>) {
        self.write_with(writer, |writer| {
            if expanded {
                writer.hard_line();
            }
        });
    }

    fn write_with(self, writer: &mut Writer<'_>, default: impl FnOnce(&mut Writer<'_>)) {
        match self.gap {
            CommentGap::LineComment => {
                writer.hard_line();
                if self.empty_line {
                    writer.hard_line();
                }
            }
            CommentGap::Source(constraint) => {
                constraint.write(writer, FlatSeparator::Empty, |writer| {
                    if self.empty_line {
                        writer.hard_line();
                        writer.hard_line();
                    } else {
                        default(writer);
                    }
                });
            }
        }
    }
}

#[derive(Clone, Copy)]
struct RecordedBoundary<'ast> {
    trailing: Option<Fragment<'ast>>,
    separator: ChainSeparator,
}

#[derive(Default)]
struct GroupFacts {
    has_function: bool,
    has_boundary_comments: bool,
    source_break: bool,
    preferred_breaks: bool,
    has_parent_call_group: bool,
}

struct ChainGroup<'ast> {
    doc: Fragment<'ast>,
    before: Option<RecordedBoundary<'ast>>,
    can_flat: bool,
    facts: GroupFacts,
}

impl<'ast> ChainGroup<'ast> {
    fn new(doc: Fragment<'ast>, before: Option<RecordedBoundary<'ast>>, facts: GroupFacts) -> Self {
        Self {
            doc,
            before,
            can_flat: doc.try_flat().is_some(),
            facts,
        }
    }
}

struct ChainGroups<'ast> {
    head: ChainGroup<'ast>,
    tail: Vec<ChainGroup<'ast>>,
}

enum ChainLayout<'ast> {
    Expanded,
    Shared(NeutralFragment<'ast>),
    Alternatives,
}

impl<'ast> ChainGroups<'ast> {
    fn layout(&self) -> ChainLayout<'ast> {
        if self.head.facts.source_break {
            return ChainLayout::Expanded;
        }
        let Some((last, earlier)) = self.tail.split_last() else {
            return ChainLayout::Alternatives;
        };
        if self.head.facts.preferred_breaks || !self.head.can_flat {
            return ChainLayout::Expanded;
        }
        let mut has_function = self.head.facts.has_function;
        let mut local_scopes = true;
        for group in earlier {
            if group.facts.source_break || group.facts.preferred_breaks || !group.can_flat {
                return ChainLayout::Expanded;
            }
            has_function |= group.facts.has_function;
            local_scopes &= !group.facts.has_parent_call_group;
        }
        let last_breaks = last.facts.preferred_breaks || !last.can_flat;
        if last.facts.source_break || last_breaks && has_function {
            return ChainLayout::Expanded;
        }
        local_scopes &= !last.facts.has_parent_call_group;
        if local_scopes
            && !last_breaks
            && let Some(head) = self.head.doc.try_neutral()
        {
            return ChainLayout::Shared(head);
        }
        ChainLayout::Alternatives
    }
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    /// Qualification retains a no-call proof only for bare member receivers.
    /// Source wrappers and grammar or diagnostic barriers keep their own rules.
    pub(super) fn fluent_chain_eligibility(
        &self,
        data: ExprData,
        context: ExpressionContext,
        layout: ExpressionLayout,
    ) -> FluentChainEligibility {
        if context.member_chain_layout == MemberChainLayout::Local
            || context.content_mode != ContentMode::Formatted
            || context.member_layout == MemberLayout::Tight
            || matches!(
                context.position,
                ExpressionPosition::NewCallee | ExpressionPosition::MemberAssignmentObject
            )
            || layout == ExpressionLayout::Flat
        {
            return FluentChainEligibility::Unqualified;
        }
        let (mut current, required_calls, mut calls, mut member) = match data {
            ExprData::CallExpression(call) => (call.callee(self.ast), 2, 1, false),
            ExprData::MemberExpression(access) => (access.object(self.ast), 1, 0, true),
            _ => return FluentChainEligibility::Unqualified,
        };
        let mut bare_members = member;
        loop {
            let data = self.ast.expr_data(current);
            current = match data {
                ExprData::CallExpression(call) => {
                    calls += 1;
                    bare_members = false;
                    call.callee(self.ast)
                }
                ExprData::MemberExpression(access) => {
                    member = true;
                    access.object(self.ast)
                }
                ExprData::TsNonNullExpression(non_null) => {
                    bare_members = false;
                    non_null.expression(self.ast)
                }
                ExprData::ParenthesizedExpression(parenthesized) => {
                    bare_members = false;
                    parenthesized.expression(self.ast)
                }
                ExprData::ChainExpression(chain) => {
                    bare_members = false;
                    chain.expression(self.ast)
                }
                _ => {
                    return if bare_members && atomic_chain_head(data) {
                        FluentChainEligibility::MemberOnly
                    } else {
                        FluentChainEligibility::Unqualified
                    };
                }
            };
            if member && calls >= required_calls {
                return FluentChainEligibility::Fluent;
            }
        }
    }

    /// Inspect the immutable source window while walking short calls and member
    /// barriers. Empty argument frames consume their own packets; the remaining
    /// spine stays clear, including terminal members outside the outermost call.
    pub(super) fn rhs_call_chain_ownership(
        &self,
        call: CallExpression,
        covered_span: Span,
    ) -> Option<RhsCallChainOwnership> {
        if self.diagnostic_range_is_sensitive(covered_span) {
            return None;
        }
        let comments = self.ast.comments();
        let first = comments.partition_point(|comment| comment.span.end <= covered_span.start);
        let comments = &comments[first..];
        let count = comments.partition_point(|comment| comment.span.start < covered_span.end);
        let mut comments = &comments[..count];
        let mut current = Expr::CallExpression(call);
        let mut context = ExpressionContext::at(ExpressionPosition::Nested);
        let mut grouping = ChainGrouping::new();
        let mut prefix = RhsChainHead::default();
        loop {
            current = match self.ast.expr_data(current) {
                ExprData::MemberExpression(member) => {
                    let head = self.prepare_member_head(member).ok()?;
                    if comments
                        .last()
                        .is_some_and(|comment| comment.span.end > head.object.span(self.ast).end)
                    {
                        return None;
                    }
                    let numeric = head.key().is_some_and(|key| {
                        matches!(self.ast.expr_data(key), ExprData::NumericLiteral(_))
                    });
                    grouping.member(numeric);
                    // Prepending never removes groups, and the head can merge
                    // at most one. Once that budget fails, compact is impossible.
                    if !short_chain(grouping.groups, true) {
                        return None;
                    }
                    if self.source_clear_member_has_chain_barrier(&head).ok()? {
                        return None;
                    }
                    prefix.member(&head, numeric);
                    context = head.receiver_context(context);
                    head.object
                }
                ExprData::CallExpression(call) => {
                    if !self.assignment_call_is_short(call) {
                        return None;
                    }
                    let callee_end = call.callee(self.ast).span(self.ast).end;
                    let mut count = comments.len();
                    while count != 0 && comments[count - 1].span.end > callee_end {
                        count -= 1;
                    }
                    let (callee_comments, frame_comments) = comments.split_at(count);
                    let callee = if let Some(first) = frame_comments.first() {
                        let arguments = self.source_call(call, context).ok()?;
                        let content = arguments.empty_argument_content()?;
                        if arguments.callee_mode != ContentMode::Formatted
                            || arguments.has_outside_prefix()
                            || first.span.start < content.start
                            || frame_comments
                                .last()
                                .is_some_and(|comment| comment.span.end > content.end)
                        {
                            return None;
                        }
                        arguments.callee_head()
                    } else {
                        self.rhs_call_callee(call, context)?
                    };
                    comments = callee_comments;
                    grouping.call();
                    prefix.call();
                    context = callee.context(context);
                    callee.expression
                }
                ExprData::TsNonNullExpression(node) => {
                    if comments.last().is_some_and(|comment| {
                        comment.span.end > node.expression(self.ast).span(self.ast).end
                    }) {
                        return None;
                    }
                    let source = self.source_non_null(node).ok()?;
                    if source.source_break() {
                        return None;
                    }
                    prefix.non_null();
                    context = source.operand_context(context);
                    if self.expression_needs_parentheses(
                        self.ast.expr_data(source.expression),
                        context.position,
                        context.content_mode,
                    ) == Some(true)
                    {
                        return None;
                    }
                    source.expression
                }
                ExprData::ParenthesizedExpression(node) => {
                    let content = self.prepare_chain_parentheses_head(node, context)?;
                    let removed = content.parentheses?;
                    if !matches!(removed.head.binding, CommentBinding::Unbound)
                        || removed.tail.is_some()
                        || comments
                            .first()
                            .is_some_and(|comment| comment.span.start < removed.source.outer.start)
                        || self.expression_needs_parentheses(
                            content.data,
                            context.position,
                            context.content_mode,
                        ) != Some(false)
                    {
                        return None;
                    }
                    // Released leading trivia is outside the emitted spine.
                    // Keep comments inside its real nodes for their frame checks.
                    let start = content.expression.span(self.ast).start;
                    let count = comments.partition_point(|comment| comment.span.end <= start);
                    comments = &comments[count..];
                    content.expression
                }
                ExprData::ChainExpression(chain)
                    if matches!(
                        context.position,
                        ExpressionPosition::MemberObject(AccessKind::Optional)
                            | ExpressionPosition::CallCallee {
                                access: AccessKind::Optional,
                                ..
                            }
                    ) =>
                {
                    chain.expression(self.ast)
                }
                data => {
                    if !comments.is_empty()
                        || !matches!(
                            data,
                            ExprData::IdentifierReference(_) | ExprData::ThisExpression(_)
                        )
                    {
                        return None;
                    }
                    let head = prefix.leading;
                    let attach = self.chain_head_attaches(data, ExpressionPosition::Nested);
                    let factory = head
                        .last_name
                        .is_some_and(|name| factory_name(self.ast.get_utf8(name.name(self.ast))));
                    let merge =
                        merge_chain_head(head.parts, factory, head.pending_computed, attach);
                    return Some(if short_chain(grouping.groups, merge) {
                        RhsCallChainOwnership::Compact
                    } else {
                        RhsCallChainOwnership::Fluent
                    });
                }
            };
        }
    }

    fn chain_head_attaches(&self, data: ExprData, position: ExpressionPosition) -> bool {
        match data {
            ExprData::ThisExpression(_) => true,
            ExprData::IdentifierReference(identifier) => {
                let name = self.ast.get_utf8(identifier.name(self.ast));
                factory_name(name)
                    || matches!(position, ExpressionPosition::Statement)
                        && name.encode_utf16().count()
                            <= usize::from(self.options.indent_width.value())
            }
            _ => false,
        }
    }

    fn prepare_fluent_chain(
        &self,
        expression: Expr,
        context: ExpressionContext,
    ) -> Result<PreparedChain<'ast>, FormatError> {
        self.prepare_chain(expression, context, Vec::new(), None)
    }

    fn prepare_chain(
        &self,
        expression: Expr,
        context: ExpressionContext,
        mut parts: Vec<ChainPart<'ast>>,
        leading_binding: Option<CommentBinding>,
    ) -> Result<PreparedChain<'ast>, FormatError> {
        let mut current = expression;
        let mut current_context = context;
        let mut head_content = None;
        let mut head_binding = leading_binding.unwrap_or(CommentBinding::Unbound);
        let mut leading_start = None;
        let mut prepared_head = leading_binding.is_some();
        if leading_binding.is_none() {
            for part in &parts {
                if let ChainPart::Parentheses(removed) = part {
                    head_binding = head_binding.combine(removed.head.binding);
                    leading_start.get_or_insert(removed.source.outer.start);
                    prepared_head = true;
                }
            }
        }
        let mut calls = 0;
        let mut next_part_is_member = false;
        let terminal_member = matches!(
            self.ast.expr_data(expression),
            ExprData::MemberExpression(_)
        );
        let mut suffix = if terminal_member {
            ChainSuffix::Collecting { parts: 0 }
        } else {
            ChainSuffix::Complete { parts: 0 }
        };
        let mut grouping = ChainGrouping::new();
        while current_context.content_mode == ContentMode::Formatted {
            match self.ast.expr_data(current) {
                ExprData::MemberExpression(member) => match self.source_member(member)? {
                    PreparedMember::Formatted(source) => {
                        let lookup_owner = MemberLookupOwner::from_context(current_context);
                        let numeric = source.key().is_some_and(|key| {
                            matches!(self.ast.expr_data(key), ExprData::NumericLiteral(_))
                        });
                        match &mut suffix {
                            ChainSuffix::Collecting {
                                parts: continuation_parts,
                            } => {
                                // The last non-numeric lookup encountered in reverse
                                // marks the suffix's beginning. Earlier numeric
                                // accesses stay with the final call's receiver.
                                if !numeric {
                                    *continuation_parts = parts.len() + 1;
                                }
                            }
                            ChainSuffix::Complete { .. } => grouping.member(numeric),
                        }
                        current = source.object;
                        head_content = None;
                        current_context = source.receiver_context(current_context);
                        parts.push(ChainPart::Member(ChainMember {
                            source,
                            lookup_owner,
                            numeric,
                            next_part_is_member,
                        }));
                        next_part_is_member = true;
                    }
                    PreparedMember::Verbatim(member) => {
                        current = Expr::MemberExpression(member);
                        current_context = current_context.with_content_mode(ContentMode::Verbatim);
                        break;
                    }
                },
                ExprData::CallExpression(call) => {
                    let arguments = self.source_call(call, current_context)?;
                    current = arguments.callee;
                    head_content = None;
                    current_context = arguments.callee_context(current_context);
                    if let ChainSuffix::Collecting { parts } = suffix {
                        suffix = ChainSuffix::Complete { parts };
                    }
                    parts.push(ChainPart::Call { call, arguments });
                    calls += 1;
                    next_part_is_member = false;
                    grouping.call();
                }
                ExprData::TsNonNullExpression(node) => {
                    let source = self.source_non_null(node)?;
                    current = source.expression;
                    head_content = None;
                    current_context = source.operand_context(current_context);
                    parts.push(ChainPart::NonNull(source));
                    next_part_is_member = false;
                    // A grammar-required pair still owns its complete operand.
                    if self.expression_needs_parentheses(
                        self.ast.expr_data(current),
                        current_context.position,
                        current_context.content_mode,
                    ) == Some(true)
                    {
                        break;
                    }
                }
                ExprData::ParenthesizedExpression(parenthesized) => {
                    let Some(mut content) =
                        self.prepare_chain_parentheses_head(parenthesized, current_context)
                    else {
                        break;
                    };
                    let Some(removed) = content.parentheses else {
                        break;
                    };
                    let binding = head_binding.combine(removed.head.binding);
                    if matches!(binding, CommentBinding::SourceBound) {
                        break;
                    }
                    head_binding = binding;
                    prepared_head = true;
                    leading_start.get_or_insert(removed.source.outer.start);
                    if self.expression_needs_parentheses(
                        content.data,
                        current_context.position,
                        current_context.content_mode,
                    ) != Some(false)
                    {
                        head_content = Some(content);
                        break;
                    }
                    current = content.expression;
                    content.parentheses = None;
                    head_content = Some(content);
                    parts.push(ChainPart::Parentheses(removed));
                }
                ExprData::ChainExpression(chain) => {
                    if matches!(
                        current_context.position,
                        ExpressionPosition::MemberObject(AccessKind::Optional)
                            | ExpressionPosition::CallCallee {
                                access: AccessKind::Optional,
                                ..
                            }
                    ) {
                        current = chain.expression(self.ast);
                        head_content = None;
                    } else if prepared_head
                        && !self.expression_is_parenthesized(
                            self.ast.expr_data(current),
                            current_context,
                        )
                    {
                        current = chain.expression(self.ast);
                        head_content = None;
                        current_context.position = ExpressionPosition::Nested;
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
        let continuation_parts = match suffix {
            ChainSuffix::Collecting { parts } | ChainSuffix::Complete { parts } => parts,
        };
        let callee_parts = parts.len() - continuation_parts;
        parts.reverse();
        Ok(PreparedChain {
            expression,
            context,
            head: current,
            head_context: current_context,
            cast_head: self.source_cast_head(current),
            prepared_head: prepared_head.then(|| {
                let content = head_content.unwrap_or_else(|| ExpressionContent {
                    expression: current,
                    data: self.ast.expr_data(current),
                    parentheses: None,
                });
                let start = content.expression.span(self.ast).start;
                let comments = if leading_binding.is_some() {
                    &[][..]
                } else {
                    self.comments
                        .in_range(Span::new(leading_start.unwrap_or(start), start))
                };
                let mut binding = head_binding.combine(
                    content
                        .parentheses
                        .map_or(CommentBinding::Unbound, |removed| removed.head.binding),
                );
                if matches!(
                    content.data,
                    ExprData::Function(_) | ExprData::ArrowFunctionExpression(_)
                ) {
                    // A call's ordinary packet can still target its actual
                    // function leaf. Resolve that target before choosing an owner.
                    binding =
                        binding.combine(self.expression_head_binding(comments, content.expression));
                }
                let leading_owner = if leading_binding.is_none()
                    && matches!(binding, CommentBinding::Unbound)
                    && !comments.is_empty()
                    && self.expression_needs_parentheses(
                        content.data,
                        current_context.position,
                        current_context.content_mode,
                    ) == Some(false)
                {
                    ChainLeadingOwner::Chain
                } else {
                    ChainLeadingOwner::Head
                };
                PreparedChainHead {
                    content,
                    leading_owner,
                    resolved: ResolvedExpressionHead {
                        head: NodeHead {
                            comments,
                            content_start: start,
                            layout: CommentRunLayout::Leading(start),
                            gap_end: None,
                        },
                        binding,
                    },
                    has_comments: !comments.is_empty(),
                }
            }),
            operation_head: None,
            parts,
            callee_parts,
            calls,
            groups: grouping.groups,
        })
    }

    /// Assertion alternatives retain the same operation owners and actual head.
    pub(super) fn prepare_assertion_chain(
        &self,
        content: ExpressionContent,
        context: ExpressionContext,
        leading_start: u32,
        binding: CommentBinding,
    ) -> Result<(PreparedChain<'ast>, ResolvedExpressionHead<'ast>), FormatError> {
        let mut parts = Vec::new();
        if let Some(removed) = content.parentheses {
            parts.push(ChainPart::Parentheses(removed));
        }
        // An optional wrapper has no token of its own. Match the real inner
        // printer's root/context while retaining the envelope tail separately.
        let mut context = context;
        let expression = match content.data {
            ExprData::ChainExpression(chain) => {
                context.position = ExpressionPosition::Nested;
                chain.expression(self.ast)
            }
            _ => content.expression,
        };
        let mut chain = self.prepare_chain(expression, context, parts, Some(binding))?;
        let (head_expression, _) = chain.emitted_head();
        let start = head_expression.span(self.ast).start;
        let comments = self.comments.in_range(Span::new(leading_start, start));
        if let Some(head) = chain.prepared_head.as_mut() {
            head.has_comments = !comments.is_empty();
        }
        let binding = chain
            .prepared_head
            .as_ref()
            .map_or(binding, |head| head.resolved.binding);
        Ok((
            chain,
            ResolvedExpressionHead {
                head: NodeHead {
                    comments,
                    content_start: start,
                    layout: CommentRunLayout::FlatLeading(start),
                    gap_end: None,
                },
                binding,
            },
        ))
    }

    pub(super) fn format_fluent_chain(
        &mut self,
        expression: Expr,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let chain = self.prepare_fluent_chain(expression, context)?;
        self.format_prepared_chain(chain, writer)
    }

    /// Prepare the restricted consumer's complete chain once, including its
    /// actual call-head owner. Other fluent contexts retain their own heads.
    pub(in crate::formatter) fn prepare_restricted_fluent_chain(
        &self,
        argument: Expr,
        content: &ExpressionContent,
        context: ExpressionContext,
        head: &ResolvedExpressionHead<'ast>,
    ) -> Result<Option<PreparedChain<'ast>>, FormatError> {
        if context.content_mode != ContentMode::Formatted
            || !matches!(head.binding, CommentBinding::Unbound)
            || head
                .head
                .comments
                .first()
                .is_some_and(|comment| comment.span.start < argument.span(self.ast).start)
            || self.diagnostic_range_is_sensitive(argument.span(self.ast))
            || self.expression_is_parenthesized(content.data, context)
            || !matches!(
                self.fluent_chain_eligibility(
                    content.data,
                    context,
                    self.expression_layout(argument),
                ),
                FluentChainEligibility::Fluent
            )
        {
            return Ok(None);
        }
        if let Some(parenthesized) = argument.as_parenthesized_expression(self.ast)
            && !self.source_parentheses_head_is_removable(parenthesized, content, context)
        {
            return Ok(None);
        }
        let mut parts = Vec::new();
        if let Some(removed) = content.parentheses {
            parts.push(ChainPart::Parentheses(removed));
        }
        let mut chain = self.prepare_chain(content.expression, context, parts, None)?;
        // These printers retain local source frames and do not split call suffixes.
        if chain.cast_head.is_none()
            && !(matches!(content.data, ExprData::MemberExpression(_)) && chain.calls <= 1)
        {
            self.prepare_chain_call_head(&mut chain);
        }
        Ok(Some(chain))
    }

    fn prepare_chain_call_head(&self, chain: &mut PreparedChain<'ast>) {
        let source_head = chain.prepared_head.as_ref().and_then(|head| {
            let first = head.resolved.head.comments.first()?;
            let last = head.resolved.head.comments.last()?;
            matches!(head.resolved.binding, CommentBinding::Unbound).then_some((
                first.span.start,
                last.span.end,
                head.resolved.head.content_start,
            ))
        });
        if let Some((first, last, start)) = source_head {
            let owner = chain.parts.iter().find_map(|part| {
                let ChainPart::Parentheses(removed) = part else {
                    return None;
                };
                let ParenthesisInner::Expression(inner) = removed.source.inner_target else {
                    return None;
                };
                let ExprData::CallExpression(call) = self.ast.expr_data(inner) else {
                    return None;
                };
                (removed.head.has_comments
                    && matches!(removed.head.binding, CommentBinding::Unbound)
                    && first > removed.source.outer.start
                    && last <= removed.source.inner.start
                    && start == call.span(self.ast).start
                    && Expr::CallExpression(call) != chain.expression)
                    .then_some(call)
            });
            let Some(owner) = owner else {
                return;
            };
            let Some(arguments) = chain.parts.iter_mut().find_map(|part| match part {
                ChainPart::Call { call, arguments } if *call == owner => Some(arguments),
                _ => None,
            }) else {
                return;
            };
            if !self.call_span_is_source_clear(arguments.callee.span(self.ast)) {
                return;
            }
            let Some(head) = arguments.empty_call_head() else {
                return;
            };
            let comments = self.comments.in_range(Span::new(first, head.content_start));
            if !self.empty_call_prefix_can_relocate(arguments.callee, head.content_start, comments)
            {
                return;
            }
            let Some(mut head) = arguments.take_empty_call_head() else {
                return;
            };
            // The clear callee separates only the two parts of this one packet.
            head.comments = comments;
            if let Some(prepared) = chain.prepared_head.as_mut() {
                prepared.resolved.head.comments = &[];
                prepared.has_comments = false;
            }
            chain.operation_head = Some((owner, head));
            return;
        }
        // A protected source head keeps precedence over an ordinary later slot.
        if chain
            .prepared_head
            .as_ref()
            .is_some_and(|head| !head.resolved.head.comments.is_empty())
        {
            return;
        }
        for part in &mut chain.parts {
            let ChainPart::Call { call, arguments } = part else {
                continue;
            };
            if Expr::CallExpression(*call) == chain.expression
                || arguments
                    .empty_call_head()
                    .is_none_or(|head| head.comments.is_empty())
                || !self.call_span_is_source_clear(arguments.callee.span(self.ast))
            {
                continue;
            }
            if let Some(head) = arguments.take_empty_call_head() {
                chain.operation_head = Some((*call, head));
            }
            break;
        }
    }

    pub(in crate::formatter) fn format_prepared_chain(
        &mut self,
        mut chain: PreparedChain<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let expression = chain.expression;
        if let Some(head) = chain.prepared_head.as_mut()
            && matches!(head.leading_owner, ChainLeadingOwner::Chain)
        {
            // The removed envelope's leading run precedes the complete chain.
            // Keep it outside the head group, as its next parsed shape does.
            let comments = std::mem::take(&mut head.resolved.head.comments);
            self.comments.claim(comments)?;
            let start = self.format_node_head(
                NodeHead {
                    comments,
                    ..head.resolved.head
                },
                chain.head_context.expression_start,
                None,
                writer,
            )?;
            head.has_comments = false;
            chain.set_expression_start(start);
        }
        let context = chain.context;
        let terminal_member = matches!(
            self.ast.expr_data(expression),
            ExprData::MemberExpression(_)
        );
        // The local path consumes the prepared sources without any choice Docs.
        // Cast-owned heads keep their actual protected source parentheses.
        if chain.cast_head.is_some() || terminal_member && chain.calls <= 1 {
            return self.format_local_chain(chain, writer);
        }
        let head_comments = chain
            .prepared_head
            .as_ref()
            .is_some_and(|head| head.has_comments)
            || !self.comments.in_range(chain.head.span(self.ast)).is_empty();
        let head_data = self.ast.expr_data(chain.head);
        let head_attach = self.chain_head_attaches(head_data, context.position);
        let operation_head = chain.operation_head.take();
        if let Some((_, head)) = &operation_head {
            self.comments.claim(head.comments)?;
        }
        let mut parts = chain.parts.into_iter();
        let tail = {
            let mut source = ChainSource {
                parts: parts.by_ref().take(chain.callee_parts).peekable(),
                pending: None,
                tail: None,
                operation_head,
                // Call count owns the complex-argument expansion rule. Group
                // count only skips inspection on the shared short-chain path.
                argument_layout: if chain.calls > 2 && chain.groups > 2 {
                    ChainArgumentLayout::Inspect
                } else {
                    ChainArgumentLayout::ByWidth
                },
            };
            let (head, merge_head) = self.record_chain_head(
                ChainHead {
                    expression: chain.head,
                    context: chain.head_context,
                    has_comments: head_comments,
                    attach: head_attach,
                    prepared: chain.prepared_head,
                },
                &mut source,
                writer,
            )?;
            // Merging the head leaves at most two groups in a short chain. Source
            // boundaries may split those groups, but already require expansion.
            if merge_head && chain.groups <= 3 {
                source.argument_layout = ChainArgumentLayout::ByWidth;
            }
            let mut tail = Vec::new();
            while source.pending.is_some() || source.parts.peek().is_some() {
                tail.push(self.record_chain_group(&mut source, writer)?);
            }
            let groups = ChainGroups { head, tail };
            let merge_head = merge_head
                && !groups.head.facts.has_boundary_comments
                && groups.tail.first().is_some_and(|group| {
                    !group.facts.has_boundary_comments && !group.facts.source_break
                });
            let short_chain = short_chain(groups.tail.len() + 1, merge_head);
            // A short chain has one shared layout even when an argument's own
            // collection group breaks. Its new source shape must not change the
            // chain's strategy on the next pass.
            if short_chain
                && !groups.head.facts.source_break
                && !groups.tail.iter().any(|group| group.facts.source_break)
            {
                writer.group(false, |writer| write_compact(&groups, writer));
            } else {
                let layout = if matches!(source.argument_layout, ChainArgumentLayout::Complex) {
                    ChainLayout::Expanded
                } else {
                    groups.layout()
                };
                match layout {
                    ChainLayout::Expanded => {
                        writer.group(false, |writer| write_expanded(&groups, merge_head, writer));
                    }
                    ChainLayout::Shared(head) => {
                        let (fragment, ()) = writer.record_with_value(|writer| {
                            write_shared(&groups, head, merge_head, writer);
                            Ok::<_, FormatError>(())
                        })?;
                        // Candidate measurement remains independent of an
                        // enclosing Flat group, as in the compact layout.
                        writer.write_canonical_alternatives(
                            fragment,
                            &[fragment.canonical_candidate()],
                            fragment,
                        );
                    }
                    ChainLayout::Alternatives => {
                        let (expanded, ()) = writer.record_with_value(|writer| {
                            write_expanded(&groups, merge_head, writer);
                            Ok::<_, FormatError>(())
                        })?;
                        let (compact, ()) = writer.record_with_value(|writer| {
                            write_compact(&groups, writer);
                            Ok::<_, FormatError>(())
                        })?;
                        writer.write_canonical_alternatives(
                            compact,
                            &[compact.canonical_candidate()],
                            expanded,
                        );
                    }
                }
            }
            source.tail
        };
        let continuation_layout = if matches!(
            context.position,
            ExpressionPosition::AssignmentRight(_) | ExpressionPosition::Initializer(_)
        ) {
            ChainLookupLayout::InlineContinuation
        } else {
            ChainLookupLayout::ExpressionContinuation
        };
        self.format_chain_operations(parts, tail, continuation_layout, writer)
    }

    fn format_local_chain(
        &mut self,
        chain: PreparedChain<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let tail =
            self.format_chain_head(chain.head, chain.prepared_head, chain.head_context, writer)?;
        self.format_chain_operations(
            chain.parts.into_iter(),
            tail,
            ChainLookupLayout::Local,
            writer,
        )
    }

    fn format_chain_head(
        &mut self,
        expression: Expr,
        prepared: Option<PreparedChainHead<'ast>>,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let Some(PreparedChainHead {
            mut content,
            resolved,
            ..
        }) = prepared
        else {
            return self.format_expression(expression, context, writer);
        };
        let removed = content.parentheses.take();
        let inner = content.expression;
        self.comments.claim(resolved.head.comments)?;
        let tail = self.format_prepared_expression_with_resolved_head(
            inner, content, resolved, context, writer,
        )?;
        match removed {
            Some(removed) => self.capture_expression_tail(
                Span::new(removed.source.inner.end, removed.source.outer.end),
                tail,
            ),
            None => Ok(tail),
        }
    }

    fn format_chain_operations(
        &mut self,
        parts: IntoIter<ChainPart<'ast>>,
        mut tail: Option<ExpressionTail<'ast>>,
        layout: ChainLookupLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        for part in parts {
            match part {
                ChainPart::Parentheses(removed) => {
                    tail = self.capture_expression_tail(
                        Span::new(removed.source.inner.end, removed.source.outer.end),
                        tail,
                    )?;
                }
                ChainPart::Member(member) => {
                    let boundary = self.prepare_member_boundary(&member.source, tail.take())?;
                    let lookup = match layout {
                        ChainLookupLayout::InlineContinuation
                            if boundary.empty_line || boundary.gap().requires_line() =>
                        {
                            MemberLookupLayout::Grouped
                        }
                        ChainLookupLayout::InlineContinuation => MemberLookupLayout::Inline,
                        ChainLookupLayout::Local | ChainLookupLayout::ExpressionContinuation => {
                            self.member_lookup_layout(
                                &member.source,
                                &boundary,
                                member.lookup_owner,
                            )
                        }
                    };
                    let boundary_inside = boundary.owns_lookup_comments
                        || !matches!(layout, ChainLookupLayout::Local)
                            && (boundary.has_before_comments() || boundary.empty_line);
                    if boundary_inside {
                        lookup.write_contents(writer, |writer| {
                            self.format_member_trailing(&boundary, writer)?;
                            if boundary.empty_line {
                                ChainSeparator {
                                    gap: boundary.gap(),
                                    empty_line: true,
                                }
                                .write(true, writer);
                            } else {
                                boundary
                                    .gap()
                                    .write(writer, FlatSeparator::Empty, |writer| {
                                        if matches!(lookup, MemberLookupLayout::Grouped) {
                                            writer.soft_line();
                                        }
                                    });
                            }
                            self.format_member_access(&member.source, boundary, writer)
                        })?;
                    } else {
                        self.format_member_trailing(&boundary, writer)?;
                        lookup.write(boundary.gap(), writer, |writer| {
                            self.format_member_access(&member.source, boundary, writer)
                        })?;
                    }
                }
                ChainPart::Call { arguments, .. } => {
                    tail = self.format_source_arguments(arguments, tail.take(), writer)?;
                }
                ChainPart::NonNull(source) => {
                    self.format_source_non_null(source, tail.take(), writer)?;
                }
            }
        }
        Ok(tail)
    }

    fn next_chain_operation(
        &self,
        source: &mut ChainSource<'ast, '_>,
    ) -> Result<Option<ChainOperation<'ast>>, FormatError> {
        if let Some(member) = source.pending.take() {
            return Ok(Some(ChainOperation::Member(member)));
        }
        for part in source.parts.by_ref() {
            match part {
                ChainPart::Parentheses(removed) => {
                    source.tail = self.capture_expression_tail(
                        Span::new(removed.source.inner.end, removed.source.outer.end),
                        source.tail.take(),
                    )?;
                }
                ChainPart::Member(member) => {
                    let boundary =
                        self.prepare_member_boundary(&member.source, source.tail.take())?;
                    return Ok(Some(ChainOperation::Member(PendingMember {
                        member,
                        boundary,
                    })));
                }
                ChainPart::Call { call, arguments } => {
                    return Ok(Some(ChainOperation::Call { call, arguments }));
                }
                ChainPart::NonNull(non_null) => return Ok(Some(ChainOperation::NonNull(non_null))),
            }
        }
        Ok(None)
    }

    fn record_chain_head(
        &mut self,
        head: ChainHead<'ast>,
        source: &mut ChainSource<'ast, '_>,
        writer: &mut Writer<'ast>,
    ) -> Result<(ChainGroup<'ast>, bool), FormatError> {
        let mut facts = GroupFacts {
            has_boundary_comments: head.has_comments,
            ..GroupFacts::default()
        };
        let mut leading = true;
        let mut head_parts = 0;
        let mut last_name: Option<IdentifierName> = None;
        let write_head = |writer: &mut Writer<'ast>| {
            source.tail =
                self.format_chain_head(head.expression, head.prepared, head.context, writer)?;
            while let Some(operation) = self.next_chain_operation(source)? {
                match operation {
                    ChainOperation::Member(member) => {
                        if !member.member.numeric || !leading {
                            leading = false;
                            if !member.member.next_part_is_member || member.source_break() {
                                source.pending = Some(member);
                                break;
                            }
                        }
                        last_name = member.member.source.name();
                        self.write_chain_member(member, false, &mut facts, writer)?;
                        head_parts += 1;
                    }
                    ChainOperation::Call { call, arguments } => {
                        last_name = None;
                        self.write_chain_call(call, arguments, source, &mut facts, writer)?;
                        head_parts += 1;
                    }
                    ChainOperation::NonNull(non_null) => {
                        self.write_chain_non_null(non_null, source, &mut facts, writer)?;
                    }
                }
            }
            Ok::<_, FormatError>(())
        };
        let (doc, ()) = writer.record_with_value(write_head)?;
        facts.preferred_breaks = doc.preferred_breaks();
        let factory =
            last_name.is_some_and(|name| factory_name(self.ast.get_utf8(name.name(self.ast))));
        let pending_computed = source
            .pending
            .as_ref()
            .is_some_and(|member| member.member.source.is_computed());
        let merge_head = !head.has_comments
            && source.pending.is_some()
            && merge_chain_head(head_parts, factory, pending_computed, head.attach);
        Ok((ChainGroup::new(doc, None, facts), merge_head))
    }

    fn record_chain_group(
        &mut self,
        source: &mut ChainSource<'ast, '_>,
        writer: &mut Writer<'ast>,
    ) -> Result<ChainGroup<'ast>, FormatError> {
        let mut before = None;
        let mut facts = GroupFacts::default();
        let mut has_parts = false;
        let mut saw_call = false;
        let (doc, ()) = writer.record_with_value(|writer| {
            while let Some(operation) = self.next_chain_operation(source)? {
                match operation {
                    ChainOperation::Member(member) => {
                        if has_parts
                            && (saw_call && !member.member.numeric || member.source_break())
                        {
                            source.pending = Some(member);
                            break;
                        }
                        if !has_parts {
                            let trailing = if member.boundary.has_before_comments() {
                                let (trailing, ()) = writer.record_with_value(|writer| {
                                    self.format_member_trailing(&member.boundary, writer)
                                })?;
                                Some(trailing)
                            } else {
                                None
                            };
                            before = Some(RecordedBoundary {
                                trailing,
                                separator: member.separator(),
                            });
                        }
                        self.write_chain_member(member, !has_parts, &mut facts, writer)?;
                    }
                    ChainOperation::Call { call, arguments } => {
                        saw_call = true;
                        self.write_chain_call(call, arguments, source, &mut facts, writer)?;
                    }
                    ChainOperation::NonNull(non_null) => {
                        self.write_chain_non_null(non_null, source, &mut facts, writer)?;
                        continue;
                    }
                }
                has_parts = true;
            }
            Ok::<_, FormatError>(())
        })?;
        facts.preferred_breaks = doc.preferred_breaks();
        Ok(ChainGroup::new(doc, before, facts))
    }

    fn write_chain_member(
        &mut self,
        member: PendingMember<'ast>,
        boundary_outside: bool,
        facts: &mut GroupFacts,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        facts.has_boundary_comments |= member.boundary.has_comments;
        facts.source_break |= member.source_break();
        if !boundary_outside {
            self.format_member_trailing(&member.boundary, writer)?;
            member.separator().write(false, writer);
        }
        self.format_member_access(&member.member.source, member.boundary, writer)
    }

    fn write_chain_non_null(
        &mut self,
        non_null: SourceNonNull<'ast>,
        source: &mut ChainSource<'ast, '_>,
        facts: &mut GroupFacts,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        facts.has_boundary_comments |= non_null.has_comments();
        facts.source_break |= non_null.source_break();
        self.format_source_non_null(non_null, source.tail.take(), writer)
    }

    fn write_chain_call(
        &mut self,
        call: CallExpression,
        arguments: SourceArguments<'ast>,
        source: &mut ChainSource<'ast, '_>,
        facts: &mut GroupFacts,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        if let Some((_, head)) = source.operation_head.take_if(|(owner, _)| *owner == call) {
            self.format_node_head(head, ExpressionStart::None, None, writer)?;
            facts.has_boundary_comments = true;
            facts.source_break = true;
        }
        // Long chains with complex arguments keep an AST-owned expansion
        // decision when a nested collection changes its source layout.
        if matches!(source.argument_layout, ChainArgumentLayout::Inspect) {
            let complex = arguments
                .arguments()
                .iter()
                .any(|id| !self.simple_call_argument(self.ast.get_node_in_sub_range(id), 2));
            if complex {
                source.argument_layout = ChainArgumentLayout::Complex;
            }
        }
        // Comments owned by the actual arguments may move inside their opener.
        // Only an outside prefix is a boundary for both merging and expansion.
        facts.has_parent_call_group |= matches!(arguments.group, ArgumentGroup::ParentCall);
        let missing_function = if facts.has_function {
            None
        } else {
            Some(&mut facts.has_function)
        };
        let output = self.format_source_arguments_with_function_fact(
            arguments,
            source.tail.take(),
            missing_function,
            writer,
        )?;
        facts.has_boundary_comments |= output.has_outside_prefix;
        facts.source_break |= output.has_outside_prefix;
        source.tail = output.tail;
        Ok(())
    }
}

fn merge_chain_head(parts: usize, factory: bool, computed: bool, attach: bool) -> bool {
    if parts == 0 {
        attach || computed
    } else {
        factory
    }
}

fn short_chain(groups: usize, merge: bool) -> bool {
    groups <= if merge { 3 } else { 2 }
}

fn factory_name(name: &str) -> bool {
    name.as_bytes().first().is_some_and(u8::is_ascii_uppercase)
        || !name.is_empty() && name.bytes().all(|byte| byte == b'_' || byte == b'$')
}

fn write_group_boundary<'ast>(group: &ChainGroup<'ast>, expanded: bool, writer: &mut Writer<'ast>) {
    if let Some(boundary) = group.before {
        // The next member owns the preceding receiver's trailing comments.
        // Entering the shared indent affects any source lines in that run.
        if let Some(trailing) = boundary.trailing {
            writer.write_fragment(trailing);
        }
        boundary.separator.write(expanded, writer);
    }
}

fn write_group_contents<'ast>(group: &ChainGroup<'ast>, writer: &mut Writer<'ast>) {
    // Segments collect operations; only their real syntax owners introduce
    // groups. Inherited receiver frames observe the complete chain's mode.
    writer.write_fragment(group.doc);
}

fn write_group<'ast>(group: &ChainGroup<'ast>, expanded: bool, writer: &mut Writer<'ast>) {
    write_group_boundary(group, expanded, writer);
    write_group_contents(group, writer);
}

fn write_compact<'ast>(groups: &ChainGroups<'ast>, writer: &mut Writer<'ast>) {
    write_group(&groups.head, false, writer);
    for group in &groups.tail {
        write_group(group, false, writer);
    }
}

fn write_expanded<'ast>(groups: &ChainGroups<'ast>, merge_head: bool, writer: &mut Writer<'ast>) {
    write_group(&groups.head, true, writer);
    let tail = if merge_head && let Some((first, remaining)) = groups.tail.split_first() {
        write_group_boundary(first, false, writer);
        write_group_contents(first, writer);
        remaining
    } else {
        &groups.tail
    };
    writer.indent(|writer| {
        for group in tail {
            write_group(group, true, writer);
        }
    });
}

fn write_shared<'ast>(
    groups: &ChainGroups<'ast>,
    head: NeutralFragment<'ast>,
    merge_head: bool,
    writer: &mut Writer<'ast>,
) {
    // A neutral head and local call scopes share their flat render view. Each
    // segment keeps its own scope when the chain's separators expand.
    writer.write_neutral_fragment(head);
    let tail = if merge_head && let Some((first, remaining)) = groups.tail.split_first() {
        write_group_boundary(first, false, writer);
        write_group_contents(first, writer);
        remaining
    } else {
        &groups.tail
    };
    writer.indent(|writer| {
        for group in tail {
            if let Some(boundary) = group.before {
                if let Some(trailing) = boundary.trailing {
                    writer.write_fragment(trailing);
                }
                boundary
                    .separator
                    .write_with(writer, |writer| writer.soft_line());
            }
            write_group_contents(group, writer);
        }
    });
}
