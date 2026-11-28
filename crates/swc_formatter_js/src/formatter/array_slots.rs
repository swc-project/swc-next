//! Array expressions and patterns share slot order and separator ownership.

use swc_next_ecma_ast::{CommentKind, Span};

use super::Formatter;
use super::comments::{CommentRunLayout, SourceSeparatorTransition};
use super::diagnostics::{FlatSeparator, LineConstraint};
use super::expressions::arguments::{
    ArgumentLayout, ElisionLayout, FollowingItem, ItemBoundary, Separator,
};
use super::suppression::ContentMode;
use crate::{FormatError, document::Writer};

/// Holes preceding the next value, or the remaining holes at the closing bracket.
/// The AST supplies the slot count even when a source comma has attached comments.
pub(super) struct ArraySlot<T> {
    pub(super) holes: usize,
    pub(super) value: Option<T>,
}

pub(super) fn array_slots<T>(
    mut elements: impl Iterator<Item = Option<T>>,
) -> impl Iterator<Item = ArraySlot<T>> {
    std::iter::from_fn(move || {
        let mut holes = 0;
        for value in elements.by_ref() {
            if value.is_some() {
                return Some(ArraySlot { holes, value });
            }
            holes += 1;
        }
        (holes != 0).then_some(ArraySlot { holes, value: None })
    })
}

impl FollowingItem {
    fn array_slot<T>(next: Option<&ArraySlot<T>>, span: impl FnOnce(&T) -> Span) -> Self {
        let Some(slot) = next else {
            return Self::End;
        };
        let node = slot.value.as_ref().map(span);
        if slot.holes == 0 {
            node.map_or(Self::End, Self::Node)
        } else {
            Self::Elision {
                node,
                holes: slot.holes,
            }
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum ArrayListLayout {
    Expression(ArgumentLayout),
    Binding,
}

impl ArrayListLayout {
    fn expand_binding_gap(self, separator: Separator, writer: &mut Writer<'_>) {
        if matches!(self, Self::Binding) && matches!(separator, Separator::EmptyLine) {
            writer.break_parent();
        }
    }
}

pub(super) struct ArrayItem<T> {
    pub(super) value: T,
    pub(super) span: Span,
    pub(super) remaining: Span,
    pub(super) following: FollowingItem,
    pub(super) layout: ArgumentLayout,
    pub(super) mode: ContentMode,
    pub(super) transition: Option<SourceSeparatorTransition>,
}

pub(super) struct ArrayEnd {
    pub(super) end: u32,
    pub(super) has_slot: bool,
    pub(super) terminal_close: Option<LineConstraint>,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_array_slots<T>(
        &mut self,
        content: Span,
        list: ArrayListLayout,
        mut next_slot: impl FnMut(&Self, u32) -> Result<Option<ArraySlot<T>>, FormatError>,
        span: impl Fn(&T) -> Span,
        mut write_item: impl FnMut(
            &mut Self,
            ArrayItem<T>,
            &mut Writer<'ast>,
        ) -> Result<ItemBoundary, FormatError>,
        writer: &mut Writer<'ast>,
    ) -> Result<ArrayEnd, FormatError> {
        let layout = match list {
            ArrayListLayout::Expression(layout) => layout,
            ArrayListLayout::Binding => ArgumentLayout::Lines,
        };
        let mut previous = ItemBoundary {
            end: content.start,
            separator: Separator::Line,
            following_mode: ContentMode::Formatted,
            elisions: ElisionLayout::Formatted,
            transition: None,
            terminal_close: None,
        };
        let mut has_slot = false;
        let mut next = next_slot(self, content.start)?;
        while let Some(slot) = next.take() {
            let value_span = slot.value.as_ref().map(&span);
            let start = value_span.map_or(content.end, |span| span.start);
            // Resolve the first real item's source prefix before its elisions claim comments.
            // Later real items inherit the mode prepared by the preceding boundary.
            let mode = if !has_slot && self.has_suppression && value_span.is_some() {
                self.content_mode(self.comments.in_range(Span::new(content.start, start)))
            } else {
                previous.following_mode
            };
            if slot.holes != 0 {
                list.expand_binding_gap(previous.separator, writer);
                (previous.end, previous.separator) = self.format_array_elisions(
                    slot.holes,
                    Span::new(previous.end, start),
                    previous.elisions,
                    has_slot.then_some(previous.separator),
                    layout,
                    writer,
                )?;
                has_slot = true;
            }
            let (Some(value), Some(value_span)) = (slot.value, value_span) else {
                break;
            };
            next = next_slot(self, value_span.end)?;
            if has_slot {
                let head = self
                    .comments
                    .between(Span::new(previous.end, start))?
                    .first()
                    .map_or(start, |comment| comment.span.start);
                let constraint = self.source_separator_constraint(
                    previous.end,
                    head,
                    previous.transition.as_ref(),
                );
                list.expand_binding_gap(previous.separator, writer);
                layout.separator(previous.separator, constraint, writer);
            }
            let item = ArrayItem {
                value,
                span: value_span,
                remaining: Span::new(previous.end, content.end),
                following: FollowingItem::array_slot(next.as_ref(), &span),
                layout,
                mode,
                transition: previous.transition,
            };
            previous = layout.entry(writer, |writer| write_item(self, item, writer))?;
            has_slot = true;
        }
        Ok(ArrayEnd {
            end: previous.end,
            has_slot,
            terminal_close: previous.terminal_close,
        })
    }

    fn format_array_elisions(
        &mut self,
        holes: usize,
        gap: Span,
        elisions: ElisionLayout,
        previous: Option<Separator>,
        layout: ArgumentLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<(u32, Separator), FormatError> {
        if let ElisionLayout::Source(count) = elisions {
            debug_assert_eq!(
                count, holes,
                "a source elision boundary covers the prepared AST slots"
            );
            return Ok((gap.start, previous.unwrap_or(Separator::Line)));
        }
        let mut comments = self.comments.between(gap)?;
        let constrained = self.diagnostic_range_is_sensitive(gap);
        let mut end = gap.start;
        let mut separator = previous.unwrap_or(Separator::Line);
        let mut has_slot = previous.is_some();
        let mut located = match elisions {
            ElisionLayout::Located { comma, separator } => Some((comma, separator)),
            ElisionLayout::Formatted | ElisionLayout::Source(_) => None,
        };
        // Borrow the packet once and advance its comment and token frontiers
        // together. Dense arrays never enter this path.
        for _ in 0..holes {
            if constrained || !comments.is_empty() || located.is_some() {
                let relocated = located.take();
                let comma = match relocated
                    .map(|(comma, _)| comma)
                    .or_else(|| self.find_comma(Span::new(end, gap.end), comments))
                {
                    Some(comma) => comma,
                    None => {
                        return Err(FormatError::unsupported(
                            "array elision boundary",
                            None,
                            gap,
                        ));
                    }
                };
                let count = comments.partition_point(|comment| comment.span.end <= comma);
                let leading = self.comments.take(count);
                comments = &comments[count..];
                match (leading.first(), has_slot) {
                    (Some(first), _) => {
                        let ordinary = !constrained
                            && leading
                                .iter()
                                .all(|&comment| self.comment_semantics(comment).is_ordinary());
                        if has_slot {
                            let start = relocated
                                .map(|(_, separator)| separator)
                                .filter(|&separator| {
                                    end <= separator && separator < first.span.start
                                })
                                .map_or(end, |separator| separator + 1);
                            let entry_gap = self.comment_gap_constraint(start, first.span.start);
                            if ordinary && matches!(entry_gap, LineConstraint::SameLine) {
                                // A movable horizontal gap belongs to the hole's slot layout.
                                layout.separator(separator, LineConstraint::Unconstrained, writer);
                            } else {
                                entry_gap.write(writer, FlatSeparator::Space, |_| {});
                            }
                        }
                        if ordinary {
                            self.format_comment_run(leading, CommentRunLayout::Source, writer)?;
                            let last = leading.last().expect("a nonempty elision comment packet");
                            if let Some((_, source_separator)) = relocated
                                && last.span.end <= source_separator
                            {
                                // This packet preceded the old real separator.
                                // Its following blank belongs to the hole boundary,
                                // rather than to the removed comma inside the packet.
                                self.line_constraint(source_separator + 1, comma).write(
                                    writer,
                                    FlatSeparator::Empty,
                                    |writer| {
                                        if matches!(separator, Separator::EmptyLine) {
                                            writer.hard_line();
                                            writer.hard_line();
                                        } else if last.kind == CommentKind::Line {
                                            writer.hard_line();
                                        }
                                    },
                                );
                            } else {
                                self.prepare_leading_comment_gap(
                                    last.span, last.kind, comma, false,
                                )
                                .gap
                                .write(
                                    writer,
                                    FlatSeparator::Empty,
                                    |_| {},
                                );
                            }
                        } else {
                            self.format_comment_run(
                                leading,
                                CommentRunLayout::Leading(comma),
                                writer,
                            )?;
                        }
                    }
                    (None, true) => {
                        layout.separator(separator, self.line_constraint(end, comma), writer)
                    }
                    // The array opener already owns this physical gap.
                    (None, false) => {}
                }
                writer.text(",");
                end = comma + 1;
            } else {
                if has_slot {
                    layout.separator(separator, LineConstraint::Unconstrained, writer);
                }
                writer.text(",");
            }
            separator = Separator::Line;
            has_slot = true;
        }
        Ok((end, separator))
    }
}
