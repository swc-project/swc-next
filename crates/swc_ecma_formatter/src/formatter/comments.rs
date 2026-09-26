//! Bounded, source-order comment consumption and basic comment layout.

use swc_next_ecma_ast::{Comment, CommentKind, Span};

use super::Formatter;
use super::diagnostics::FlatSeparator;
use super::source::{lines, split_line};
use crate::{FormatError, document::Writer};

/// A comment can carry more than one tool's annotation.
pub(super) struct CommentSemantics {
    pub(super) suppression: bool,
    pub(super) diagnostic: bool,
    pub(super) file_directive: bool,
    pub(super) optimizer: bool,
    pub(super) type_cast: bool,
}

impl CommentSemantics {
    pub(super) fn is_ordinary(&self) -> bool {
        !self.suppression
            && !self.diagnostic
            && !self.file_directive
            && !self.has_target_annotation()
    }

    pub(super) fn has_target_annotation(&self) -> bool {
        self.optimizer || self.type_cast
    }
}

/// A syntax owner resolves the source target before relocating comments.
/// Unknown annotation targets and diagnostic coverage retain the source boundary.
#[derive(Clone, Copy)]
pub(super) enum CommentBinding {
    Unbound,
    SourceBound,
}

impl CommentBinding {
    pub(super) fn combine(self, other: Self) -> Self {
        match (self, other) {
            (Self::Unbound, binding) | (binding, Self::Unbound) => binding,
            _ => Self::SourceBound,
        }
    }

    pub(super) fn combine_all(bindings: impl Iterator<Item = Self>) -> Self {
        let mut combined = Self::Unbound;
        for binding in bindings {
            combined = combined.combine(binding);
            if matches!(combined, Self::SourceBound) {
                break;
            }
        }
        combined
    }

    pub(super) fn allows_relocation(&self) -> bool {
        match self {
            Self::Unbound => true,
            Self::SourceBound => false,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum CommentRunLayout {
    Source,
    SourceAfter(u32),
    CanonicalDangling,
}

enum CommentText<'ast> {
    Inline(&'ast str),
    Literal {
        first: &'ast str,
        rest: &'ast str,
    },
    Aligned {
        first: &'ast str,
        rest: &'ast str,
        jsdoc: bool,
    },
}

struct PreparedComment<'ast> {
    span: Span,
    kind: CommentKind,
    text: CommentText<'ast>,
}

impl PreparedComment<'_> {
    fn is_alignable(&self) -> bool {
        matches!(self.text, CommentText::Aligned { .. })
    }
}

impl<'ast> CommentText<'ast> {
    fn write(&self, writer: &mut Writer<'ast>) {
        match *self {
            Self::Inline(text) => writer.text(text),
            Self::Literal { first, rest } => {
                writer.text(first);
                for line in lines(rest) {
                    writer.literal_line();
                    writer.text(line);
                }
            }
            Self::Aligned { first, rest, jsdoc } => {
                writer.text(first.trim_end());
                for line in lines(rest) {
                    writer.hard_line();
                    writer.space();
                    let trimmed = line.trim();
                    writer.text(trimmed);
                    if jsdoc && trimmed != "*" && line.ends_with("  ") {
                        // Markdown uses two trailing spaces as a hard line break.
                        writer.text("  ");
                    }
                }
            }
        }
    }
}

pub(super) struct CommentCursor<'ast> {
    remaining: &'ast [Comment],
}

impl<'ast> CommentCursor<'ast> {
    /// A prepared boundary becomes claimable after its preceding child is printed.
    pub(super) fn claim(&mut self, comments: &'ast [Comment]) -> Result<(), FormatError> {
        match (self.first(), comments.first()) {
            (_, None) => Ok(()),
            (Some(pending), Some(first)) if pending.span == first.span => {
                self.take(comments.len());
                Ok(())
            }
            (Some(pending), Some(_)) => Err(FormatError::unsupported(
                "comment placement",
                None,
                pending.span,
            )),
            (None, Some(_)) => {
                unreachable!("prepared comment boundary must remain unconsumed")
            }
        }
    }

    /// Prepares a later boundary without consuming comments inside its child.
    pub(super) fn in_range(&self, span: Span) -> &'ast [Comment] {
        let first = self
            .remaining
            .partition_point(|comment| comment.span.start < span.start);
        let comments = &self.remaining[first..];
        let count = comments.partition_point(|comment| comment.span.end <= span.end);
        &comments[..count]
    }

    pub(super) fn new(comments: &'ast [Comment]) -> Self {
        debug_assert!(
            comments
                .windows(2)
                .all(|pair| pair[0].span.end <= pair[1].span.start),
            "AST comments must remain in source order",
        );
        Self {
            remaining: comments,
        }
    }

    pub(super) fn first(&self) -> Option<Comment> {
        self.remaining.first().copied()
    }

    pub(super) fn between(&self, span: Span) -> Result<&'ast [Comment], FormatError> {
        if let Some(comment) = self.first()
            && comment.span.start < span.start
        {
            return Err(FormatError::unsupported(
                "comment placement",
                None,
                comment.span,
            ));
        }
        let count = self
            .remaining
            .iter()
            .take_while(|comment| comment.span.end <= span.end)
            .count();
        Ok(&self.remaining[..count])
    }

    pub(super) fn take(&mut self, count: usize) -> &'ast [Comment] {
        let (claimed, remaining) = self.remaining.split_at(count);
        self.remaining = remaining;
        claimed
    }
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn comment_semantics(&self, comment: Comment) -> CommentSemantics {
        CommentSemantics {
            suppression: self.has_suppression
                && super::suppression::is_suppression_comment(self.ast, comment),
            diagnostic: self.comment_is_diagnostic_directive(comment),
            file_directive: self.has_file_directives && self.comment_is_file_directive(comment),
            optimizer: self.has_target_annotations && self.comment_is_optimizer_annotation(comment),
            type_cast: self.has_type_casts && self.comment_is_type_cast(comment),
        }
    }

    pub(super) fn comment_is_file_directive(&self, comment: Comment) -> bool {
        let raw = self.source(comment.span);
        if !raw.contains('@') && !raw.starts_with("///") {
            return false;
        }
        match comment.kind {
            CommentKind::Line => {
                let triple = raw.strip_prefix("///");
                let marker = triple
                    .or_else(|| raw.strip_prefix("//"))
                    .unwrap_or(raw)
                    .trim_start_matches(super::source::is_source_whitespace);
                let tag = marker
                    .split(|character| {
                        character == ':' || super::source::is_source_whitespace(character)
                    })
                    .next()
                    .unwrap_or(marker);
                if tag.eq_ignore_ascii_case("@ts-check") || tag.eq_ignore_ascii_case("@ts-nocheck")
                {
                    return true;
                }
                triple.is_some()
                    && ["<reference", "<amd-module", "<amd-dependency"]
                        .iter()
                        .any(|tag| {
                            marker
                                .strip_prefix(tag)
                                .is_some_and(|rest| rest.starts_with([' ', '\t', '/', '>']))
                        })
            }
            CommentKind::Block => {
                raw.contains('@')
                    && raw.char_indices().any(|(offset, character)| {
                        if character != '@' {
                            return false;
                        }
                        let tag = raw[offset + 1..]
                            .split(super::source::is_source_whitespace)
                            .next()
                            .unwrap_or("");
                        ["jsx", "jsxfrag", "jsximportsource", "jsxruntime"]
                            .iter()
                            .any(|name| tag.eq_ignore_ascii_case(name))
                    })
            }
        }
    }

    pub(super) fn comment_is_optimizer_annotation(&self, comment: Comment) -> bool {
        let raw = self.source(comment.span);
        raw.match_indices('@')
            .chain(raw.match_indices('#'))
            .any(|(offset, _)| {
                let tag = &raw[offset + 1..];
                tag.starts_with("__PURE__") || tag.starts_with("__NO_SIDE_EFFECTS__")
            })
    }

    pub(super) fn comment_has_target_annotation(&self, comment: Comment) -> bool {
        self.comment_is_optimizer_annotation(comment) || self.comment_is_type_cast(comment)
    }

    pub(super) fn comment_binding(&self, comments: &[Comment], boundary: Span) -> CommentBinding {
        let binding = self.comment_run_binding(comments.iter().copied(), |comment| {
            let semantics = self.comment_semantics(comment);
            if semantics.is_ordinary() {
                CommentBinding::Unbound
            } else {
                CommentBinding::SourceBound
            }
        });
        if matches!(binding, CommentBinding::SourceBound)
            || self.diagnostic_range_is_sensitive(boundary)
        {
            CommentBinding::SourceBound
        } else {
            binding
        }
    }

    /// Owners distinguish absent and unknown bindings; a run moves only with one shared target.
    pub(super) fn comment_run_binding(
        &self,
        comments: impl Iterator<Item = Comment>,
        resolve: impl FnMut(Comment) -> CommentBinding,
    ) -> CommentBinding {
        if !self.has_target_annotations
            && !self.has_file_directives
            && !self.has_suppression
            && self.diagnostic_lines.is_empty()
        {
            return CommentBinding::Unbound;
        }
        CommentBinding::combine_all(comments.map(resolve))
    }

    pub(super) fn comment_is_type_cast(&self, comment: Comment) -> bool {
        let raw = self.source(comment.span);
        if !raw.starts_with("/**") {
            return false;
        }
        for (offset, _) in raw.match_indices('@') {
            let tag = &raw[offset..];
            let after = tag
                .strip_prefix("@type")
                .or_else(|| tag.strip_prefix("@satisfies"));
            if after.is_some_and(|after| {
                after
                    .as_bytes()
                    .first()
                    .is_none_or(|byte| !byte.is_ascii_alphanumeric() && *byte != b'_')
            }) {
                return true;
            }
        }
        false
    }

    fn prepare_comment(&self, comment: Comment) -> Result<PreparedComment<'ast>, FormatError> {
        let source = self.source(comment.span);
        let (first, rest) = split_line(source);
        let text = match rest {
            None => CommentText::Inline(source.trim_end()),
            Some(rest) if lines(rest).all(|line| line.trim_start().starts_with('*')) => {
                CommentText::Aligned {
                    first,
                    rest,
                    jsdoc: source.starts_with("/**") && !source.starts_with("/***"),
                }
            }
            Some(rest) => CommentText::Literal { first, rest },
        };
        Ok(PreparedComment {
            span: comment.span,
            kind: comment.kind,
            text,
        })
    }

    pub(super) fn comment_gap(&self, start: u32, end: u32, writer: &mut Writer<'ast>) {
        self.line_constraint(start, end)
            .write(writer, FlatSeparator::Space, |writer| {
                match self.line_breaks(start, end) {
                    0 => writer.space(),
                    1 => writer.hard_line(),
                    _ => {
                        writer.hard_line();
                        if self.next_line_is_empty(start, end) {
                            writer.hard_line();
                        }
                    }
                }
            });
    }

    pub(super) fn format_comment_run(
        &self,
        comments: &[Comment],
        layout: CommentRunLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let mut previous: Option<PreparedComment<'ast>> = None;
        for &comment in comments {
            let prepared = self.prepare_comment(comment)?;
            if let Some(previous) = previous.as_ref() {
                let nestled = previous.span.end == prepared.span.start
                    && previous.is_alignable()
                    && prepared.is_alignable();
                if !nestled {
                    match layout {
                        CommentRunLayout::Source | CommentRunLayout::SourceAfter(_) => {
                            self.comment_gap(previous.span.end, prepared.span.start, writer)
                        }
                        CommentRunLayout::CanonicalDangling => self
                            .line_constraint(previous.span.end, prepared.span.start)
                            .write(writer, FlatSeparator::Space, Writer::hard_line),
                    }
                }
            }
            if previous.is_none()
                && let CommentRunLayout::SourceAfter(start) = layout
            {
                self.comment_gap(start, prepared.span.start, writer);
            }
            prepared.text.write(writer);
            previous = Some(prepared);
        }
        Ok(())
    }

    pub(super) fn format_trailing_comments(
        &mut self,
        span: Span,
        statement_end: u32,
        has_following: bool,
        writer: &mut Writer<'ast>,
    ) -> Result<u32, FormatError> {
        let comments = self.comments.between(span)?;
        let comments = self.trailing_comment_prefix(comments, span, statement_end, has_following);
        let comments = self.comments.take(comments.len());
        let end = comments
            .last()
            .map_or(span.start, |comment| comment.span.end);
        if self
            .comment_binding(comments, Span::new(span.start, end))
            .allows_relocation()
        {
            self.format_trailing_comment_run(comments, writer)?;
        } else {
            self.format_comment_run(comments, CommentRunLayout::SourceAfter(span.start), writer)?;
        }
        Ok(comments
            .last()
            .map_or(span.start, |comment| span.start.max(comment.span.end)))
    }

    pub(super) fn format_trailing_comment_run(
        &self,
        comments: &[Comment],
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let mut previous: Option<PreparedComment<'ast>> = None;
        for &comment in comments {
            let prepared = self.prepare_comment(comment)?;
            if previous
                .as_ref()
                .is_some_and(|previous| previous.kind == CommentKind::Line)
            {
                writer.hard_line();
                prepared.text.write(writer);
            } else if comment.kind == CommentKind::Line {
                writer.line_suffix(self.source(comment.span).trim_end());
            } else {
                let nestled = previous.as_ref().is_some_and(|previous| {
                    previous.span.end == prepared.span.start
                        && previous.is_alignable()
                        && prepared.is_alignable()
                });
                if !nestled {
                    writer.space();
                }
                prepared.text.write(writer);
            }
            previous = Some(prepared);
        }
        Ok(())
    }

    pub(super) fn trailing_comment_prefix(
        &self,
        comments: &'ast [Comment],
        span: Span,
        statement_end: u32,
        has_following: bool,
    ) -> &'ast [Comment] {
        let mut end = span.start;
        let mut count = 0;
        for (index, comment) in comments.iter().enumerate() {
            if has_following
                && self.has_target_annotations
                && self.comment_has_target_annotation(*comment)
            {
                break;
            }
            if self.line_breaks(end, comment.span.start) != 0 {
                break;
            }
            let following = comments
                .get(index + 1)
                .map_or(span.end, |next| next.span.start);
            if comment.kind == CommentKind::Line
                || self.line_breaks(comment.span.end, following) != 0
            {
                count = index + 1;
                break;
            }
            end = comment.span.end;
            if !has_following || comment.span.end < statement_end {
                count = index + 1;
            }
        }
        &comments[..count]
    }
}
