//! Shared source-ordered argument slots and comma/comment boundaries.
use super::AccessKind;

use super::ExpressionContext;
use super::Formatter;
use super::calls::CallCallee;
use crate::formatter::comments::{
    ArrayCommentOwner, CommentBinding, CommentRunLayout, ExpressionTail, NodeHead,
};
use crate::formatter::diagnostics::{FlatSeparator, LineConstraint};
use crate::formatter::suppression::{
    ContentMode, DelimitedFollowing, DelimitedSuppression, SuppressionPosition,
};
use crate::{FormatError, TrailingCommas, document::Writer};
use swc_next_ecma_ast::{
    Argument, ArgumentData, CallExpression, Comment, CommentKind, Expr, ExprData, GetSpan, Span,
    SubRange,
};

#[derive(Clone, Copy)]
pub(super) enum FollowingArgument {
    End,
    Argument(Argument),
    Elision {
        argument: Option<Argument>,
        holes: usize,
    },
}

pub(super) struct ArrayArgumentSlot {
    pub(super) argument: Argument,
    pub(super) following: FollowingArgument,
    pub(super) mode: ContentMode,
    pub(super) layout: ArgumentLayout,
}

#[derive(Clone, Copy)]
pub(in crate::formatter) enum FollowingItem {
    End,
    ResolvedTerminal(DelimitedSuppression),
    Node(Span),
    ModuleSpecifier(Span),
    Elision { node: Option<Span>, holes: usize },
}

impl FollowingItem {
    fn node(self) -> Option<Span> {
        match self {
            Self::End | Self::ResolvedTerminal(_) => None,
            Self::Node(span) | Self::ModuleSpecifier(span) => Some(span),
            Self::Elision { node, .. } => node,
        }
    }
}

/// The real terminal item owns its comma before the following comments.
#[derive(Clone, Copy)]
pub(in crate::formatter) enum TrailingCommaPolicy {
    Forbidden,
    IfBreak,
}

struct ItemComments<'item, 'ast> {
    head: CommentRunLayout,
    array: Option<&'item mut ArrayCommentOwner<'ast>>,
}

pub(in crate::formatter) struct PreparedItem {
    item_span: Span,
    following: FollowingItem,
    span: Span,
    pub(in crate::formatter) mode: ContentMode,
    has_leading_comments: bool,
    trailing_comma: TrailingCommaPolicy,
    suppression: Option<DelimitedSuppression>,
}

impl PreparedItem {
    pub(in crate::formatter) fn set_trailing_comma(&mut self, policy: TrailingCommaPolicy) {
        self.trailing_comma = policy;
    }

    pub(in crate::formatter) fn set_leading_mode(&mut self, mode: ContentMode) {
        self.mode = if mode == ContentMode::Verbatim
            || self
                .suppression
                .as_ref()
                .and_then(|source| source.preceding)
                .is_some_and(|target| target == self.item_span)
        {
            ContentMode::Verbatim
        } else {
            ContentMode::Formatted
        };
    }
}

#[derive(Clone, Copy)]
pub(in crate::formatter) enum ArgumentLayout {
    Lines,
    Fill,
}

#[derive(Clone, Copy)]
pub(in crate::formatter) enum Separator {
    Line,
    EmptyLine,
    OwnLineComment,
    TrailingLine,
    Source(LineConstraint),
}

#[derive(Clone, Copy)]
pub(in crate::formatter) enum ElisionLayout {
    Formatted,
    Source(usize),
}

#[derive(Clone, Copy)]
pub(in crate::formatter) struct ItemBoundary {
    pub(in crate::formatter) end: u32,
    pub(in crate::formatter) separator: Separator,
    pub(in crate::formatter) following_mode: ContentMode,
    pub(in crate::formatter) elisions: ElisionLayout,
}

impl ArgumentLayout {
    pub(super) fn entry<'a, T>(
        self,
        writer: &mut Writer<'a>,
        write: impl FnOnce(&mut Writer<'a>) -> T,
    ) -> T {
        match self {
            Self::Lines => write(writer),
            Self::Fill => writer.fill_entry(write),
        }
    }

    pub(in crate::formatter) fn separator(
        self,
        separator: Separator,
        constraint: LineConstraint,
        writer: &mut Writer<'_>,
    ) {
        self.entry(writer, |writer| {
            if let Separator::Source(constraint) = separator {
                constraint.write(writer, FlatSeparator::Space, Writer::line);
                return;
            }
            if matches!(separator, Separator::TrailingLine) {
                // Fill may choose a flat separator even in an expanded array.
                // A pending owned suffix must flush before the following item.
                writer.hard_line();
                return;
            }
            constraint.write(writer, FlatSeparator::Space, |writer| {
                match (self, separator) {
                    (Self::Fill, Separator::EmptyLine) => {
                        writer.hard_line();
                        writer.hard_line();
                    }
                    (Self::Fill, Separator::OwnLineComment) => writer.hard_line(),
                    (_, Separator::EmptyLine) => {
                        writer.line();
                        writer.soft_line();
                    }
                    _ => writer.line(),
                }
            })
        });
    }
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_argument_item_with_array_comments(
        &mut self,
        slot: ArrayArgumentSlot,
        span: Span,
        trailing_comma: bool,
        mut array_comments: Option<&mut ArrayCommentOwner<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<ItemBoundary, FormatError> {
        let ArrayArgumentSlot {
            argument,
            following,
            mode,
            layout,
        } = slot;
        let following = match following {
            FollowingArgument::Argument(argument) => FollowingItem::Node(argument.span(self.ast)),
            FollowingArgument::Elision { argument, holes } => FollowingItem::Elision {
                node: argument.map(|node| node.span(self.ast)),
                holes,
            },
            FollowingArgument::End => FollowingItem::End,
        };
        let mut item = self.prepare_delimited_item_with_comments(
            argument.span(self.ast),
            following,
            span,
            trailing_comma,
            ItemComments {
                head: CommentRunLayout::Source,
                array: array_comments.as_deref_mut(),
            },
            writer,
        )?;
        item.set_leading_mode(mode);
        let mut write = |writer: &mut Writer<'ast>| {
            self.format_argument_with_array_comments(
                argument,
                item.mode,
                array_comments.as_deref_mut(),
                writer,
            )
        };
        // Ordinary elements own their complete expression scope; their comma
        // and separator still observe the array. Concise fill owns its entries.
        let tail = match layout {
            ArgumentLayout::Lines => writer.group(false, write),
            ArgumentLayout::Fill => write(writer),
        }?;
        self.finish_delimited_item_with_array_comments(item, tail, array_comments, writer)
    }

    pub(in crate::formatter) fn prepare_delimited_item(
        &mut self,
        item_span: Span,
        following: FollowingItem,
        span: Span,
        trailing_comma: bool,
        writer: &mut Writer<'ast>,
    ) -> Result<PreparedItem, FormatError> {
        self.prepare_delimited_item_with_head(
            item_span,
            following,
            span,
            trailing_comma,
            CommentRunLayout::Source,
            writer,
        )
    }
    pub(super) fn prepare_call_argument_item(
        &mut self,
        item_span: Span,
        following: FollowingItem,
        span: Span,
        trailing_comma: bool,
        writer: &mut Writer<'ast>,
    ) -> Result<PreparedItem, FormatError> {
        self.prepare_delimited_item_with_head(
            item_span,
            following,
            span,
            trailing_comma,
            CommentRunLayout::GroupedLeading(item_span.start),
            writer,
        )
    }
    pub(in crate::formatter) fn prepare_delimited_item_with_head(
        &mut self,
        item_span: Span,
        following: FollowingItem,
        span: Span,
        trailing_comma: bool,
        head_layout: CommentRunLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<PreparedItem, FormatError> {
        self.prepare_delimited_item_with_comments(
            item_span,
            following,
            span,
            trailing_comma,
            ItemComments {
                head: head_layout,
                array: None,
            },
            writer,
        )
    }
    fn prepare_delimited_item_with_comments(
        &mut self,
        item_span: Span,
        following: FollowingItem,
        span: Span,
        trailing_comma: bool,
        comments: ItemComments<'_, 'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<PreparedItem, FormatError> {
        let following_span = following.node();
        let suppression = if let FollowingItem::ResolvedTerminal(source) = following {
            Some(source)
        } else {
            self.delimited_suppression(
                item_span,
                following_span.map_or(span.end, |span| span.start),
                match following {
                    FollowingItem::End | FollowingItem::ResolvedTerminal(_) => {
                        DelimitedFollowing::End
                    }
                    FollowingItem::Node(_) => DelimitedFollowing::Node,
                    FollowingItem::ModuleSpecifier(_) => DelimitedFollowing::ModuleSpecifier,
                    FollowingItem::Elision { node: Some(_), .. } => DelimitedFollowing::Node,
                    FollowingItem::Elision { node: None, .. } => DelimitedFollowing::End,
                },
            )
        };
        let leading = self
            .comments
            .between(Span::new(span.start, item_span.start))?;
        let leading = self.comments.take(leading.len());
        let leading_mode = self.content_mode(leading);
        let mode = if leading_mode == ContentMode::Verbatim
            || suppression
                .as_ref()
                .and_then(|source| source.preceding)
                .is_some_and(|target| target == item_span)
        {
            ContentMode::Verbatim
        } else {
            ContentMode::Formatted
        };
        let source_head = matches!(comments.head, CommentRunLayout::Source)
            || !matches!(
                self.comment_binding(
                    leading,
                    Span::new(span.start, item_span.start),
                    Some(item_span)
                ),
                CommentBinding::Unbound
            );
        self.format_comment_run_with_array_comments(
            leading,
            if source_head {
                CommentRunLayout::Source
            } else {
                comments.head
            },
            comments.array,
            writer,
        )?;
        if source_head && let Some(last) = leading.last() {
            self.comment_gap(last.span.end, item_span.start, writer);
        }
        Ok(PreparedItem {
            item_span,
            following,
            span,
            mode,
            trailing_comma: if trailing_comma {
                TrailingCommaPolicy::IfBreak
            } else {
                TrailingCommaPolicy::Forbidden
            },
            suppression,
            has_leading_comments: !leading.is_empty(),
        })
    }

    pub(in crate::formatter) fn finish_delimited_item(
        &mut self,
        item: PreparedItem,
        tail: Option<ExpressionTail<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<ItemBoundary, FormatError> {
        self.finish_delimited_item_with_array_comments(item, tail, None, writer)
    }
    fn finish_delimited_item_with_array_comments(
        &mut self,
        item: PreparedItem,
        tail: Option<ExpressionTail<'ast>>,
        mut array_comments: Option<&mut ArrayCommentOwner<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<ItemBoundary, FormatError> {
        let PreparedItem {
            item_span,
            following,
            span,
            mode,
            trailing_comma,
            suppression,
            ..
        } = item;
        let following_span = following.node();
        let has_more = !matches!(
            following,
            FollowingItem::End | FollowingItem::ResolvedTerminal(_)
        );
        let content_end = tail.as_ref().map_or(item_span.end, |tail| tail.source_end);
        let following_start = following_span.map(|span| span.start);
        let gap = Span::new(content_end, following_start.unwrap_or(span.end));
        let following_mode = suppression
            .as_ref()
            .map_or(ContentMode::Formatted, |source| source.following);
        let preceding = suppression.as_ref().and_then(|source| source.preceding);
        if matches!(following, FollowingItem::ResolvedTerminal(_))
            || preceding.is_some()
                && (following_span.is_some() || matches!(following, FollowingItem::Elision { .. }))
        {
            let source = self.format_delimited_source_gap(
                gap,
                following_span.map_or(DelimitedFollowing::End, |_| DelimitedFollowing::Node),
                writer,
            )?;
            return Ok(ItemBoundary {
                end: source.end,
                separator: Separator::Source(source.following),
                following_mode,
                elisions: match following {
                    FollowingItem::Elision { holes, .. } => ElisionLayout::Source(holes),
                    _ => ElisionLayout::Formatted,
                },
            });
        }
        let comments = self.comments.between(gap)?;
        let comma = self.find_token(gap, comments, ",");
        let constrained = self.diagnostic_range_is_sensitive(span);
        let preserve_comma = constrained || matches!(following, FollowingItem::Elision { .. });
        // A directive-covered separator owns the comments on its source side.
        let protected_comma =
            comma.filter(|&position| constrained && self.diagnostic_line_is_sensitive(position));
        let trailing_limit = if matches!(following, FollowingItem::Elision { .. }) {
            // The next elision owns the comments after this item's separator.
            comma.unwrap_or(gap.end)
        } else {
            gap.end
        };
        let ordinary_attachment = mode == ContentMode::Formatted
            && !constrained
            && !matches!(following, FollowingItem::Elision { .. });
        let (trailing_count, physical_break, inline_line) = if ordinary_attachment {
            let before_separator = comma.map_or(comments.len(), |comma| {
                comments.partition_point(|comment| comment.span.end <= comma)
            });
            let (trailing, leading) = self.split_separator_comments(
                comments,
                before_separator,
                content_end,
                following_start,
            );
            let end = trailing
                .last()
                .map_or(content_end, |comment| comment.span.end);
            let physical_break = leading
                .first()
                .is_some_and(|comment| self.line_breaks(end, comment.span.start) != 0);
            let inline_line = trailing
                .last()
                .is_some_and(|comment| comment.kind == CommentKind::Line);
            (trailing.len(), physical_break, inline_line)
        } else {
            let mut inline_end = content_end;
            let mut trailing_count = 0;
            let mut physical_break = false;
            let mut inline_line = false;
            for (index, comment) in comments.iter().enumerate() {
                if comment.span.start >= trailing_limit {
                    break;
                }
                if self.line_breaks(inline_end, comment.span.start) != 0 {
                    physical_break = true;
                    break;
                }
                inline_end = comment.span.end;
                if comment.kind == CommentKind::Line {
                    trailing_count = index + 1;
                    inline_line = true;
                    break;
                }
                if following_span.is_none()
                    || comma.is_none_or(|position| comment.span.start < position)
                {
                    trailing_count = index + 1;
                }
            }
            (trailing_count, physical_break, inline_line)
        };
        let prepare_run = (physical_break || inline_line) && ordinary_attachment;
        let prepared_run = if prepare_run {
            self.prepare_delimited_trailing_run(
                comments,
                content_end,
                trailing_limit,
                following_span.is_some(),
                array_comments.as_deref_mut(),
            )
        } else {
            None
        };
        let canonical_run = prepared_run.is_some();
        let boundary_count = match prepared_run.as_ref() {
            Some(run) => {
                debug_assert!(
                    run.len() >= trailing_count,
                    "a canonical trailing run must include the complete legacy inline prefix",
                );
                run.len()
            }
            None => comma
                .filter(|_| preserve_comma)
                .map_or(trailing_count, |comma| {
                    comments
                        .partition_point(|comment| comment.span.end <= comma)
                        .max(trailing_count)
                }),
        };
        let boundary = self.comments.take(boundary_count);
        let (trailing, before_comma) = if canonical_run {
            (&boundary[..0], &boundary[..0])
        } else {
            boundary.split_at(trailing_count)
        };
        let trailing_end = trailing
            .last()
            .map_or(content_end, |comment| comment.span.end);
        self.format_inline_trailing_comments_with_array_comments(
            trailing,
            array_comments.as_deref_mut(),
            writer,
        )?;
        if !before_comma.is_empty() {
            self.format_comment_run_with_array_comments(
                before_comma,
                CommentRunLayout::SourceAfter(trailing_end),
                array_comments.as_deref_mut(),
                writer,
            )?;
        }
        let boundary_end = boundary
            .last()
            .map_or(content_end, |comment| comment.span.end);
        let mut separator = if comments[boundary_count..]
            .iter()
            .any(|comment| comment.kind == CommentKind::Line)
        {
            Separator::OwnLineComment
        } else {
            Separator::Line
        };
        let required_comma = has_more || protected_comma.is_some();
        if required_comma {
            if let Some(comma) = comma
                && let Some(last) = boundary.last()
                && last.span.end <= comma
            {
                self.line_constraint(last.span.end, comma).write(
                    writer,
                    FlatSeparator::Empty,
                    |writer| {
                        if before_comma
                            .last()
                            .is_some_and(|comment| comment.kind == CommentKind::Line)
                        {
                            writer.hard_line();
                        }
                    },
                );
            }
            writer.text(",");
            if let Some(comma) = comma {
                let after_separator = Span::new(comma + 1, gap.end);
                // The source gap after the comma is independent of comment ownership.
                let after_comma = comments.partition_point(|comment| comment.span.start <= comma);
                let next_start = comments
                    .get(boundary_count.max(after_comma))
                    .map_or(gap.end, |comment| comment.span.start);
                let start = after_separator.start.max(boundary_end);
                if start < next_start && self.next_line_is_empty(start, next_start) {
                    separator = Separator::EmptyLine;
                }
            }
        } else if matches!(trailing_comma, TrailingCommaPolicy::IfBreak) {
            writer.if_break(",");
        }
        if let Some(run) = prepared_run {
            // Keep comma and the entire ordinary Block/Line packet together.
            // The next real separator owns its mandatory suffix termination.
            self.format_prepared_trailing_run(run, array_comments.as_deref_mut(), writer)?;
            if !matches!(separator, Separator::EmptyLine) {
                separator = Separator::TrailingLine;
            }
        }
        let mut end = boundary_end;
        if required_comma
            && let Some(comma) = comma
            && preserve_comma
            && self
                .comments
                .first()
                .is_none_or(|comment| comment.span.start >= comma)
        {
            end = end.max(comma + 1);
        }
        if !has_more || !preserve_comma && following_span.is_none() {
            let own_line = self.comments.between(Span::new(end, span.end))?;
            let own_line = self.comments.take(own_line.len());
            if let Some(first) = own_line.first() {
                writer.hard_line();
                if self.previous_line_is_empty(end, first.span.start) {
                    writer.hard_line();
                }
                self.format_comment_run_with_array_comments(
                    own_line,
                    CommentRunLayout::Lines,
                    array_comments,
                    writer,
                )?;
            }
            if let Some(last) = own_line.last() {
                end = last.span.end;
            }
        }
        Ok(ItemBoundary {
            end,
            separator,
            following_mode,
            elisions: ElisionLayout::Formatted,
        })
    }
}

pub(super) struct ArgumentHead {
    pub(super) first_mode: ContentMode,
    pub(super) ordinary_group: ArgumentGroup,
}
struct ArgumentPunctuation {
    operator: Option<Span>,
    opening: u32,
}
fn argument_opening_gap(prefix_start: u32, first: Option<Span>, end: u32) -> Span {
    Span::new(prefix_start, first.map_or(end - 1, |span| span.start))
}
pub(super) struct SourceArguments<'ast> {
    arguments: SubRange<Argument>,
    first: Option<SourceFirstArgument>,
    delimiters: Span,
    pub(super) callee: Expr,
    callee_span: Span,
    prefix: SourceArgumentPrefix<'ast>,
    pub(super) callee_mode: ContentMode,
    pub(super) group: ArgumentGroup,
    pub(super) curried_callee: Option<CallExpression>,
}
#[derive(Clone, Copy)]
struct SourceFirstArgument {
    node: Argument,
    mode: ContentMode,
}
/// An argument owner also reports the boundary actually emitted to its chain.
pub(super) struct FormattedArguments<'ast> {
    pub(super) tail: Option<ExpressionTail<'ast>>,
    pub(super) has_outside_prefix: bool,
}
enum ArgumentPrefix<'ast> {
    Outside(&'ast [Comment]),
    Arguments(&'ast [Comment]),
}
impl<'ast> ArgumentPrefix<'ast> {
    fn comments(&self) -> &'ast [Comment] {
        match self {
            Self::Outside(c) | Self::Arguments(c) => c,
        }
    }
    fn belongs_to_arguments(&self) -> bool {
        matches!(self, Self::Arguments(_))
    }
}
enum SourceArgumentPrefix<'ast> {
    Direct(ArgumentPrefix<'ast>),
    Optional {
        operator: Span,
        before: &'ast [Comment],
        after: ArgumentPrefix<'ast>,
    },
    Empty {
        operator: Option<Span>,
        comments: &'ast [Comment],
    },
}
impl SourceArgumentPrefix<'_> {
    fn has_comments(&self) -> bool {
        match self {
            Self::Direct(prefix) => !prefix.comments().is_empty(),
            Self::Optional { before, after, .. } => {
                !before.is_empty() || !after.comments().is_empty()
            }
            Self::Empty { .. } => true,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum ArgumentGroup {
    Local,
    ParentCall,
}

impl ArgumentGroup {
    pub(super) fn write<'a, T>(
        self,
        expand: bool,
        writer: &mut Writer<'a>,
        write: impl FnOnce(&mut Writer<'a>) -> T,
    ) -> T {
        match (self, expand) {
            (Self::ParentCall, false) => write(writer),
            _ => writer.group(expand, write),
        }
    }
}

impl<'ast> SourceArguments<'ast> {
    pub(super) fn arguments(&self) -> SubRange<Argument> {
        self.arguments
    }
    /// An empty call has no first argument that could own this ordinary packet.
    pub(super) fn empty_call_head(&self) -> Option<NodeHead<'ast>> {
        if !self.arguments.is_empty() || self.callee_mode != ContentMode::Formatted {
            return None;
        }
        let comments = match &self.prefix {
            SourceArgumentPrefix::Empty { comments, .. } => *comments,
            SourceArgumentPrefix::Direct(prefix) if prefix.comments().is_empty() => {
                prefix.comments()
            }
            SourceArgumentPrefix::Optional { before, after, .. }
                if before.is_empty() && after.comments().is_empty() =>
            {
                after.comments()
            }
            _ => return None,
        };
        Some(NodeHead {
            comments,
            content_start: self.delimiters.start,
            layout: CommentRunLayout::GroupedLeading(self.delimiters.start),
            gap_end: None,
        })
    }

    /// Move the prepared packet without claiming it; the chain keeps its source order.
    pub(super) fn take_empty_call_head(&mut self) -> Option<NodeHead<'ast>> {
        let head = self.empty_call_head()?;
        let operator = match self.prefix {
            SourceArgumentPrefix::Empty { operator, .. } => operator,
            SourceArgumentPrefix::Optional { operator, .. } => Some(operator),
            SourceArgumentPrefix::Direct(_) => None,
        };
        self.prefix = match operator {
            Some(operator) => SourceArgumentPrefix::Optional {
                operator,
                before: &[],
                after: ArgumentPrefix::Outside(&[]),
            },
            None => SourceArgumentPrefix::Direct(ArgumentPrefix::Outside(&[])),
        };
        Some(head)
    }

    pub(super) fn empty_argument_content(&self) -> Option<Span> {
        self.arguments
            .is_empty()
            .then(|| Span::new(self.delimiters.start + 1, self.delimiters.end - 1))
    }

    pub(super) fn access(&self) -> AccessKind {
        match self.prefix {
            SourceArgumentPrefix::Direct(_) => AccessKind::Ordinary,
            SourceArgumentPrefix::Optional { .. } => AccessKind::Optional,
            SourceArgumentPrefix::Empty { operator, .. } => {
                if operator.is_some() {
                    AccessKind::Optional
                } else {
                    AccessKind::Ordinary
                }
            }
        }
    }

    pub(super) fn has_outside_prefix(&self) -> bool {
        match &self.prefix {
            SourceArgumentPrefix::Direct(prefix) => {
                !prefix.belongs_to_arguments() && !prefix.comments().is_empty()
            }
            SourceArgumentPrefix::Optional { before, after, .. } => {
                !before.is_empty() || !after.belongs_to_arguments() && !after.comments().is_empty()
            }
            SourceArgumentPrefix::Empty { .. } => true,
        }
    }
    pub(super) fn callee_head(&self) -> CallCallee {
        CallCallee {
            expression: self.callee,
            access: self.access(),
            mode: self.callee_mode,
            curried_call: self.curried_callee,
        }
    }
    pub(super) fn callee_context(&self, context: ExpressionContext) -> ExpressionContext {
        self.callee_head().context(context)
    }
}
impl<'ast, 'source> Formatter<'ast, 'source> {
    /// The enclosing chain has already proved that this argument boundary is
    /// comment- and diagnostic-clear. Preserve the actual source token geometry.
    pub(super) fn source_clear_argument_opening(
        &self,
        arguments: SubRange<Argument>,
        callee: Expr,
        end: u32,
        access: AccessKind,
    ) -> bool {
        let ast = self.ast;
        let first = arguments.first().map(|id| {
            let argument: Argument = ast.get_node_in_sub_range(id);
            argument.span(ast)
        });
        let gap = argument_opening_gap(callee.span(ast).end, first, end);
        self.source_argument_punctuation(gap, access, &[]).is_some()
    }

    fn source_argument_punctuation(
        &self,
        gap: Span,
        access: AccessKind,
        comments: &[Comment],
    ) -> Option<ArgumentPunctuation> {
        let operator = match access {
            AccessKind::Ordinary => None,
            AccessKind::Optional => {
                let start = self.find_token(gap, comments, "?.")?;
                Some(Span::new(start, start + 2))
            }
        };
        let opening_start = operator.map_or(gap.start, |operator| operator.end);
        let opening_gap = Span::new(opening_start, gap.end);
        let opening = self.find_token(opening_gap, comments, "(")?;
        Some(ArgumentPunctuation { operator, opening })
    }

    pub(super) fn source_arguments(
        &self,
        arguments: SubRange<Argument>,
        callee: Expr,
        end: u32,
        access: AccessKind,
    ) -> Option<SourceArguments<'ast>> {
        let ast = self.ast;
        let callee_span = callee.span(ast);
        let prefix_start = callee_span.end;
        let mut first = arguments.iter().next().map(|id| SourceFirstArgument {
            node: ast.get_node_in_sub_range(id),
            mode: ContentMode::Formatted,
        });
        let first_span = first.map(|argument| argument.node.span(ast));
        let gap = argument_opening_gap(prefix_start, first_span, end);
        let boundary_comments = self.comments.in_range(gap);
        let ArgumentPunctuation { operator, opening } =
            self.source_argument_punctuation(gap, access, boundary_comments)?;
        let prefix_count = boundary_comments.partition_point(|comment| comment.span.end <= opening);
        let prefix_comments = &boundary_comments[..prefix_count];
        let (prefix, callee_mode) = match operator {
            None => {
                let span = Span::new(prefix_start, opening);
                let prefix = self.argument_prefix(first.as_mut(), span, prefix_comments);
                let mode = if prefix.belongs_to_arguments() {
                    ContentMode::Formatted
                } else {
                    self.trailing_content_mode(span, false)
                };
                (SourceArgumentPrefix::Direct(prefix), mode)
            }
            Some(operator) => {
                let before_count =
                    prefix_comments.partition_point(|comment| comment.span.end <= operator.start);
                let (before, after) = prefix_comments.split_at(before_count);
                let before_span = Span::new(prefix_start, operator.start);
                self.prepare_first_argument_suppression(first.as_mut(), before, operator.start);
                let before_mode = self.trailing_content_mode(before_span, false);
                let after_span = Span::new(operator.end, opening);
                let after = self.argument_prefix(first.as_mut(), after_span, after);
                let after_mode = if after.belongs_to_arguments() {
                    ContentMode::Formatted
                } else {
                    self.trailing_content_mode(after_span, false)
                };
                if !after.belongs_to_arguments()
                    && self.content_mode(after.comments()) == ContentMode::Verbatim
                    && (first.is_some() || after_mode != ContentMode::Verbatim)
                {
                    return None;
                }
                let mode = if before_mode == ContentMode::Verbatim
                    || after_mode == ContentMode::Verbatim
                {
                    ContentMode::Verbatim
                } else {
                    ContentMode::Formatted
                };
                let prefix = SourceArgumentPrefix::Optional {
                    operator,
                    before,
                    after,
                };
                (prefix, mode)
            }
        };
        let prefix =
            if first.is_none() && prefix.has_comments() && callee_mode == ContentMode::Formatted {
                let comments = prefix_comments;
                if !comments.is_empty()
                    && self.empty_call_prefix_can_relocate(
                        callee,
                        opening,
                        self.comments
                            .in_range(Span::new(callee_span.start, opening)),
                    )
                {
                    SourceArgumentPrefix::Empty { operator, comments }
                } else {
                    prefix
                }
            } else {
                prefix
            };
        Some(SourceArguments {
            arguments,
            first,
            delimiters: Span::new(opening, end),
            callee,
            callee_span,
            prefix,
            callee_mode,
            group: ArgumentGroup::Local,
            curried_callee: None,
        })
    }

    /// A packet moves into an empty call only if that prefix can own it again.
    pub(super) fn empty_call_prefix_can_relocate(
        &self,
        callee: Expr,
        opening: u32,
        comments: &[Comment],
    ) -> bool {
        let callee_start = callee.span(self.ast).start;
        let start = comments
            .first()
            .map_or(callee_start, |comment| callee_start.min(comment.span.start));
        self.comment_binding(comments, Span::new(start, opening + 1), None)
            .allows_relocation(None)
            && self.source_cast_head(callee).is_none()
    }

    #[inline]
    fn argument_prefix(
        &self,
        first: Option<&mut SourceFirstArgument>,
        span: Span,
        comments: &'ast [Comment],
    ) -> ArgumentPrefix<'ast> {
        let Some(first) = first else {
            return ArgumentPrefix::Outside(comments);
        };
        let target = self.has_suppression.then(|| first.node.span(self.ast));
        let binding = self.comment_binding(comments, Span::new(span.start, span.end + 1), target);
        let outside_line = comments.iter().any(|comment| {
            comment.kind == CommentKind::Line
                && (!self.source_line_starts_line(comment.span.start)
                    || !matches!(binding, CommentBinding::Unbound))
        });
        let prefix = if comments.is_empty() || outside_line || !binding.allows_relocation(target) {
            ArgumentPrefix::Outside(comments)
        } else {
            ArgumentPrefix::Arguments(comments)
        };
        if !prefix.belongs_to_arguments() {
            self.prepare_first_argument_suppression(Some(first), comments, span.end);
        }
        prefix
    }

    fn prepare_first_argument_suppression(
        &self,
        first: Option<&mut SourceFirstArgument>,
        comments: &[Comment],
        following_start: u32,
    ) {
        if !self.has_suppression {
            return;
        }
        let Some(first) = first else {
            return;
        };
        // Relocation and consumption are separate: an own-line marker before
        // the opener still targets this actual argument when it stays outside.
        if self
            .suppression_positions(comments, following_start)
            .any(|position| matches!(position, SuppressionPosition::OwnLine))
        {
            first.mode = ContentMode::Verbatim;
        }
    }

    fn format_empty_argument_prefix(
        &self,
        comments: &'ast [Comment],
        operator: Option<Span>,
        source_end: u32,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let mut previous_end = source_end;
        let mut pending_line = false;
        for comment in comments {
            let own_line = self.source_position_starts_line(comment.span.start);
            let gap_start = operator
                .filter(|operator| {
                    previous_end <= operator.start && operator.end <= comment.span.start
                })
                .map_or(previous_end, |operator| operator.end);
            if pending_line || own_line {
                writer.hard_line();
                if self.next_line_is_empty(gap_start, comment.span.start) {
                    writer.hard_line();
                }
            } else {
                writer.space();
            }
            self.format_comment_run(
                std::slice::from_ref(comment),
                CommentRunLayout::Source,
                writer,
            )?;
            pending_line = own_line || comment.kind == CommentKind::Line;
            previous_end = comment.span.end;
        }
        if pending_line {
            writer.hard_line();
        }
        Ok(())
    }

    pub(super) fn format_argument_with_function_fact(
        &mut self,
        argument: Argument,
        mode: ContentMode,
        missing_function: &mut Option<&mut bool>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        if missing_function.is_some()
            && let ArgumentData::Expr(expression) = self.ast.argument_data(argument)
            && matches!(
                self.expression_content(expression).data,
                ExprData::Function(_) | ExprData::ArrowFunctionExpression(_)
            )
            && let Some(function) = missing_function.take()
        {
            *function = true;
        }
        self.format_argument(argument, mode, writer)
    }

    pub(super) fn format_source_arguments(
        &mut self,
        list: SourceArguments<'ast>,
        tail: Option<ExpressionTail<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let output = self.format_source_arguments_with_function_fact(list, tail, None, writer)?;
        Ok(output.tail)
    }

    pub(super) fn format_source_arguments_with_function_fact(
        &mut self,
        list: SourceArguments<'ast>,
        tail: Option<ExpressionTail<'ast>>,
        mut missing_function: Option<&mut bool>,
        writer: &mut Writer<'ast>,
    ) -> Result<FormattedArguments<'ast>, FormatError> {
        let ast = self.ast;
        let SourceArguments {
            arguments,
            first,
            delimiters,
            callee_span,
            prefix,
            group,
            ..
        } = list;
        let first_mode = first.map_or(ContentMode::Formatted, |argument| argument.mode);
        let opening = delimiters.start;
        let prefix_start = callee_span.end;
        let source_end = tail.as_ref().map_or(prefix_start, |tail| tail.source_end);
        let mut has_outside_prefix = false;
        let content_start = match prefix {
            SourceArgumentPrefix::Empty { operator, comments } => {
                let comments = if tail.is_some() {
                    self.comments.between(Span::new(source_end, opening))?
                } else {
                    comments
                };
                has_outside_prefix |= !comments.is_empty();
                self.comments.claim(comments)?;
                self.format_empty_argument_prefix(comments, operator, source_end, writer)?;
                if operator.is_some() {
                    writer.text("?.");
                }
                opening + 1
            }
            SourceArgumentPrefix::Direct(prefix) => {
                let before = self.comments.between(Span::new(source_end, opening))?;
                let prefix_belongs_to_arguments = prefix.belongs_to_arguments();
                let move_tail = first.is_some()
                    && (prefix.comments().is_empty() || prefix_belongs_to_arguments)
                    && tail.as_ref().is_some_and(|tail| {
                        tail.comments
                            .iter()
                            .all(|comment| comment.kind == CommentKind::Block)
                            && self
                                .comment_binding(
                                    tail.comments,
                                    Span::new(source_end, opening + 1),
                                    None,
                                )
                                .allows_relocation(None)
                    });
                let outside_count = if move_tail {
                    0
                } else if prefix_belongs_to_arguments {
                    tail.as_ref().map_or(0, |tail| tail.comments.len())
                } else {
                    before.len()
                };
                let (outside, inside) = before.split_at(outside_count);
                has_outside_prefix |= !outside.is_empty();
                let boundary = self.prepare_comment_boundary(outside, source_end, opening);
                self.format_comment_boundary(&boundary, writer)?;
                boundary.gap().write(writer, FlatSeparator::Empty, |_| {});
                if inside.is_empty() {
                    opening + 1
                } else if move_tail {
                    source_end
                } else {
                    prefix_start
                }
            }
            SourceArgumentPrefix::Optional {
                operator,
                before,
                after,
            } => {
                let before = if tail.is_some() {
                    self.comments
                        .between(Span::new(source_end, operator.start))?
                } else {
                    before
                };

                has_outside_prefix |= !before.is_empty();
                let boundary = self.prepare_comment_boundary(before, source_end, operator.start);
                self.format_comment_boundary(&boundary, writer)?;
                boundary.gap().write(writer, FlatSeparator::Empty, |_| {});
                writer.text("?.");
                if after.belongs_to_arguments() {
                    operator.end
                } else {
                    let after = after.comments();
                    has_outside_prefix |= !after.is_empty();

                    let boundary = self.prepare_comment_boundary(after, operator.end, opening);
                    self.format_comment_boundary(&boundary, writer)?;
                    boundary.gap().write(writer, FlatSeparator::Empty, |_| {});
                    opening + 1
                }
            }
        };
        let content = Span::new(content_start, delimiters.end - 1);
        if first.is_none() {
            let comments = self.comments.in_range(content);
            if self.content_mode(comments) == ContentMode::Verbatim {
                self.format_verbatim(Span::new(opening, delimiters.end), writer)?;
                return Ok(FormattedArguments {
                    tail: None,
                    has_outside_prefix,
                });
            }
            if comments.is_empty() {
                writer.text("(");
                self.format_argument_dangling(content, writer)?;
                writer.text(")");
                return Ok(FormattedArguments {
                    tail: None,
                    has_outside_prefix,
                });
            }
        }
        let hugged = arguments.len() == 1
            && self.comments.in_range(content).is_empty()
            && !self.diagnostic_range_is_sensitive(content)
            && first.is_some_and(|argument| match ast.argument_data(argument.node) {
                ArgumentData::Expr(expression) => matches!(
                    self.expression_content(expression).data,
                    ExprData::ArrayExpression(_) | ExprData::ObjectExpression(_)
                ),
                ArgumentData::SpreadElement(_) => false,
            });
        let expand = arguments
            .iter()
            .map(|id| ast.get_node_in_sub_range::<Argument>(id))
            .scan(content.start, |previous, argument| {
                let span = argument.span(ast);
                let empty = self.next_line_is_empty(*previous, span.start);
                *previous = span.end;
                Some(empty)
            })
            .any(|empty| empty)
            || self.arguments_are_function_composition(arguments);
        if !expand && let Some(selected) = self.select_argument_layout(arguments, !hugged) {
            let tail = self.format_selected_arguments(
                arguments,
                content,
                ArgumentHead {
                    first_mode,
                    ordinary_group: group,
                },
                selected,
                missing_function,
                writer,
            )?;
            return Ok(FormattedArguments {
                tail,
                has_outside_prefix,
            });
        }
        let tail = group.write(expand, writer, |writer| {
            writer.text("(");
            let mut write_items = |writer: &mut Writer<'ast>| {
                if !hugged && (first.is_some() || !self.comments.in_range(content).is_empty()) {
                    self.opening_line_constraint(content).write(
                        writer,
                        FlatSeparator::Empty,
                        Writer::soft_line,
                    );
                }
                let mut arguments = arguments
                    .iter()
                    .map(|id| ast.get_node_in_sub_range(id))
                    .peekable();
                let mut previous = None;
                while let Some(argument) = arguments.next() {
                    let mut item = self.prepare_argument_slot(
                        argument,
                        arguments.peek().copied(),
                        previous,
                        content,
                        !hugged && self.options.trailing_commas == TrailingCommas::All,
                        writer,
                    )?;
                    if previous.is_none() && first_mode == ContentMode::Verbatim {
                        item.set_leading_mode(first_mode);
                    }
                    let tail = self.format_argument_with_function_fact(
                        argument,
                        item.mode,
                        &mut missing_function,
                        writer,
                    )?;
                    previous = Some(self.finish_delimited_item(item, tail, writer)?);
                }
                let end = previous.map_or(content.start, |boundary| boundary.end);
                self.format_argument_dangling(Span::new(end, content.end), writer)
            };
            let end = if hugged {
                write_items(writer)?
            } else {
                writer.indent(write_items)?
            };
            if !hugged && (first.is_some() || end != content.start) {
                self.line_constraint(end, content.end).write(
                    writer,
                    FlatSeparator::Empty,
                    Writer::soft_line,
                );
            }
            writer.text(")");
            Ok(None)
        })?;
        Ok(FormattedArguments {
            tail,
            has_outside_prefix,
        })
    }
    pub(super) fn prepare_argument_slot(
        &mut self,
        node: Argument,
        following: Option<Argument>,
        previous: Option<ItemBoundary>,
        content: Span,
        trailing_comma: bool,
        writer: &mut Writer<'ast>,
    ) -> Result<PreparedItem, FormatError> {
        let span = node.span(self.ast);
        let end = previous.map_or(content.start, |boundary| boundary.end);
        if let Some(previous) = previous {
            let leading = self.comments.between(Span::new(end, span.start))?;
            let next = leading
                .first()
                .map_or(span.start, |comment| comment.span.start);
            ArgumentLayout::Lines.separator(
                previous.separator,
                self.line_constraint(end, next),
                writer,
            );
        }
        let following = following.map_or(FollowingItem::End, |argument| {
            FollowingItem::Node(argument.span(self.ast))
        });
        let mut item = self.prepare_call_argument_item(
            span,
            following,
            Span::new(end, content.end),
            trailing_comma,
            writer,
        )?;
        if let Some(previous) = previous {
            item.set_leading_mode(previous.following_mode);
        }
        Ok(item)
    }
    /// Reads the unconsumed outside boundary of this actual argument slot.
    /// Object-internal comments remain owned by the value formatter.
    pub(super) fn argument_has_boundary_comments(&self, item: &PreparedItem) -> bool {
        if item.has_leading_comments {
            return true;
        }
        let end = item
            .following
            .node()
            .map_or(item.span.end, |span| span.start);
        !self
            .comments
            .in_range(Span::new(item.item_span.end, end))
            .is_empty()
    }

    pub(super) fn first_callback_has_boundary_comments(&self, item: &PreparedItem) -> bool {
        if item.has_leading_comments {
            return true;
        }
        let end = item
            .following
            .node()
            .map_or(item.span.end, |span| span.start);
        let gap = Span::new(item.item_span.end, end);
        let comments = self.comments.in_range(gap);
        let Some(comment) = comments.first() else {
            return false;
        };
        let comma = self.find_token(gap, comments, ",");
        comma.is_none_or(|comma| comment.span.start < comma)
    }

    pub(super) fn format_argument_dangling(
        &mut self,
        span: Span,
        writer: &mut Writer<'ast>,
    ) -> Result<u32, FormatError> {
        let dangling = self.comments.between(span)?;
        self.comments.claim(dangling)?;
        if dangling
            .iter()
            .any(|comment| comment.kind == CommentKind::Line)
        {
            writer.break_parent();
        }
        self.format_comment_run(dangling, CommentRunLayout::Lines, writer)?;
        Ok(dangling
            .last()
            .map_or(span.start, |comment| comment.span.end))
    }
}
