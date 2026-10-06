//! Heritage keywords share a prepared continuation owner.

use swc_next_ecma_ast::{Comment, Expr, ExprData, Span, TsTypeParameterInstantiation};

use super::Formatter;
use super::comments::{CommentBinding, CommentBoundary, CommentRunLayout};
use super::diagnostics::FlatSeparator;
use super::expressions::arguments::{ArgumentLayout, FollowingItem, ItemBoundary};
use super::expressions::{ExpressionContext, ExpressionPosition, TailOwner};
use super::suppression::{ContentMode, SuppressionPosition};
use crate::{FormatError, document::Writer};

#[derive(Clone, Copy)]
pub(super) enum HeritageLayout {
    Inline,
    Single,
    Multiple,
}

impl HeritageLayout {
    pub(super) fn grouped(self) -> bool {
        !matches!(self, Self::Inline)
    }
}

pub(super) struct HeritageItem {
    pub(super) span: Span,
    pub(super) expression: Expr,
    pub(super) arguments: Option<TsTypeParameterInstantiation>,
}

pub(super) struct HeritageOwners {
    pub(super) inline_before: Span,
    pub(super) line_preceding: Span,
}

pub(super) struct PreparedHeritagePrefix<'ast> {
    pub(super) prefix: HeritagePrefix<'ast>,
    pub(super) preceding: CommentBinding,
    pub(super) first_mode: ContentMode,
}

pub(super) struct HeritagePrefix<'ast> {
    layout: HeritagePrefixLayout<'ast>,
    pub(super) has_comments: bool,
}

enum HeritagePrefixLayout<'ast> {
    Source(Span),
    Formatted {
        before: CommentBoundary<'ast>,
        after: CommentBoundary<'ast>,
        leading: HeritageLeading,
        inline_target: bool,
        first_start: u32,
    },
}

#[derive(Clone, Copy)]
enum HeritageLeading {
    None,
    Keyword(CommentBinding),
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    /// Source parentheses and chain wrappers do not change the header's operand shape.
    pub(super) fn heritage_expression_is_member(&self, mut expression: Expr) -> bool {
        loop {
            expression = match self.ast.expr_data(expression) {
                ExprData::MemberExpression(_) => return true,
                ExprData::ParenthesizedExpression(node) => node.expression(self.ast),
                ExprData::ChainExpression(node) => node.expression(self.ast),
                ExprData::TsNonNullExpression(node) => node.expression(self.ast),
                _ => return false,
            };
        }
    }

    pub(super) fn prepare_heritage_prefix(
        &self,
        gap: Span,
        keyword: Span,
        comments: &'ast [Comment],
        owners: impl FnOnce() -> HeritageOwners,
        first: Span,
    ) -> PreparedHeritagePrefix<'ast> {
        let split = comments.partition_point(|comment| comment.span.end <= keyword.start);
        let (before, after) = comments.split_at(split);
        let mut preceding = CommentBinding::Unbound;
        let mut first_mode = ContentMode::Formatted;
        let mut before_suppression = CommentBinding::Unbound;
        let mut after_suppression = CommentBinding::Unbound;
        if self.has_suppression && !comments.is_empty() {
            let owners = owners();
            for (_, position) in self.suppressed_comment_runs(before, keyword.start) {
                let target = if matches!(position, SuppressionPosition::Inline) {
                    owners.inline_before
                } else {
                    owners.line_preceding
                };
                before_suppression = before_suppression.combine(CommentBinding::BoundTo(target));
                if target == first {
                    first_mode = ContentMode::Verbatim;
                } else {
                    preceding = preceding.combine(CommentBinding::BoundTo(target));
                }
            }
            for (_, position) in self.suppressed_comment_runs(after, gap.end) {
                let target = if matches!(position, SuppressionPosition::Inline) {
                    first
                } else {
                    owners.line_preceding
                };
                after_suppression = after_suppression.combine(CommentBinding::BoundTo(target));
                if target == first {
                    first_mode = ContentMode::Verbatim;
                } else {
                    preceding = preceding.combine(CommentBinding::BoundTo(target));
                }
            }
        }
        let before_binding = self.heritage_comment_binding(
            before,
            Span::new(gap.start, keyword.start),
            before_suppression,
        );
        // An inline marker occupies the sole physical run in this trivia-only
        // prefix; separated runs select the preceding owner instead.
        let after_binding = self.heritage_comment_binding(
            after,
            Span::new(keyword.end, gap.end),
            after_suppression,
        );
        let source = matches!(before_binding, CommentBinding::SourceBound)
            || matches!(after_binding, CommentBinding::SourceBound);
        let leading = !source
            && first_mode == ContentMode::Formatted
            && (matches!(after_binding, CommentBinding::BoundTo(_))
                || after
                    .first()
                    .is_some_and(|comment| self.source_position_starts_line(comment.span.start)));
        PreparedHeritagePrefix {
            prefix: HeritagePrefix {
                has_comments: !comments.is_empty(),
                layout: if source {
                    HeritagePrefixLayout::Source(gap)
                } else {
                    let before_selects_first = matches!(before_binding, CommentBinding::BoundTo(target) if target == first);
                    let (before, after) = if before_selects_first {
                        (
                            self.prepare_comment_boundary(&[], gap.start, keyword.start),
                            self.prepare_comment_boundary(comments, gap.start, gap.end),
                        )
                    } else {
                        (
                            self.prepare_comment_boundary(before, gap.start, keyword.start),
                            self.prepare_comment_boundary(after, keyword.end, gap.end),
                        )
                    };
                    HeritagePrefixLayout::Formatted {
                        before,
                        after,
                        leading: if leading {
                            HeritageLeading::Keyword(after_binding)
                        } else {
                            HeritageLeading::None
                        },
                        inline_target: first_mode == ContentMode::Verbatim,
                        first_start: first.start,
                    }
                },
            },
            preceding,
            first_mode,
        }
    }

    fn heritage_comment_binding(
        &self,
        comments: &[Comment],
        boundary: Span,
        suppression: CommentBinding,
    ) -> CommentBinding {
        let binding = self.comment_run_binding(comments.iter().copied(), |comment| {
            let semantics = self.comment_semantics(comment);
            if semantics.is_ordinary() {
                CommentBinding::Unbound
            } else if semantics.suppression {
                match suppression {
                    CommentBinding::Unbound => CommentBinding::SourceBound,
                    binding => binding,
                }
            } else {
                CommentBinding::SourceBound
            }
        });
        if self.diagnostic_range_is_sensitive(boundary) {
            CommentBinding::SourceBound
        } else {
            binding
        }
    }

    pub(super) fn format_heritage_clause<T>(
        &mut self,
        prefix: HeritagePrefix<'ast>,
        keyword: &'static str,
        layout: HeritageLayout,
        writer: &mut Writer<'ast>,
        body: impl FnOnce(&mut Self, &mut Writer<'ast>) -> Result<T, FormatError>,
    ) -> Result<T, FormatError> {
        let (before, after, leading, inline_target, first_start) = match prefix.layout {
            HeritagePrefixLayout::Source(span) => {
                self.format_verbatim(span, writer)?;
                return body(self, writer);
            }
            HeritagePrefixLayout::Formatted {
                before,
                after,
                leading,
                inline_target,
                first_start,
            } => (before, after, leading, inline_target, first_start),
        };
        self.format_comment_boundary(&before, writer)?;
        before.gap().write(writer, FlatSeparator::Space, |writer| {
            if layout.grouped() {
                writer.line();
            } else {
                writer.space();
            }
        });
        if matches!(leading, HeritageLeading::Keyword(_)) {
            if matches!(layout, HeritageLayout::Multiple)
                && matches!(leading, HeritageLeading::Keyword(CommentBinding::Unbound))
            {
                self.format_comment_boundary_with_layout(
                    &after,
                    CommentRunLayout::CanonicalDangling,
                    writer,
                )?;
                writer.hard_line();
            } else {
                self.format_comment_boundary_with_layout(
                    &after,
                    CommentRunLayout::GroupedLeading(first_start),
                    writer,
                )?;
            }
        }
        let write = |formatter: &mut Self, writer: &mut Writer<'ast>| {
            writer.text(keyword);
            let write_items = |formatter: &mut Self, writer: &mut Writer<'ast>| {
                match leading {
                    HeritageLeading::Keyword(_) => {
                        if matches!(layout, HeritageLayout::Multiple) {
                            writer.line();
                        } else {
                            writer.space();
                        }
                    }
                    HeritageLeading::None => {
                        formatter.format_comment_boundary(&after, writer)?;
                        after.gap().write(writer, FlatSeparator::Space, |writer| {
                            if matches!(layout, HeritageLayout::Multiple) && !inline_target {
                                writer.line();
                            } else {
                                writer.space();
                            }
                        });
                    }
                }
                body(formatter, writer)
            };
            if matches!(layout, HeritageLayout::Multiple) {
                writer.group(false, |writer| {
                    writer.indent(|writer| write_items(formatter, writer))
                })
            } else {
                write_items(formatter, writer)
            }
        };
        if matches!(layout, HeritageLayout::Single) {
            writer.group(false, |writer| write(self, writer))
        } else {
            write(self, writer)
        }
    }

    pub(super) fn format_heritage_items(
        &mut self,
        items: impl Iterator<Item = HeritageItem>,
        first_mode: ContentMode,
        list_end: u32,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let mut items = items.peekable();
        let mut previous: Option<ItemBoundary> = None;
        while let Some(node) = items.next() {
            let span = node.span;
            let end = previous.map_or(span.start, |previous| previous.end);
            if let Some(previous) = previous {
                let next = self
                    .comments
                    .between(Span::new(end, span.start))?
                    .first()
                    .map_or(span.start, |comment| comment.span.start);
                ArgumentLayout::Lines.separator(
                    previous.separator,
                    self.line_constraint(end, next),
                    writer,
                );
            }
            let following = items
                .peek()
                .map_or(FollowingItem::End, |next| FollowingItem::Node(next.span));
            let mut item = self.prepare_delimited_item(
                span,
                following,
                Span::new(end, list_end),
                false,
                writer,
            )?;
            item.set_leading_mode(previous.map_or(first_mode, |previous| previous.following_mode));
            let tail = if item.mode == ContentMode::Verbatim {
                self.format_verbatim(span, writer)?;
                None
            } else if let Some(arguments) = node.arguments {
                let arguments = self.prepare_expression_type_arguments(node.expression, arguments);
                let context = ExpressionContext::at(ExpressionPosition::Nested)
                    .with_content_mode(arguments.callee_mode)
                    .with_tail_owner(TailOwner::ParentOperation);
                let tail = self.format_expression(node.expression, context, writer)?;
                self.format_expression_type_arguments(arguments, tail, writer)?;
                None
            } else {
                self.format_expression(
                    node.expression,
                    ExpressionContext::at(ExpressionPosition::Nested),
                    writer,
                )?
            };
            previous = Some(self.finish_delimited_item(item, tail, writer)?);
        }
        Ok(())
    }
}
