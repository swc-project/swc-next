//! Ignore targets resolved at their owning syntax boundaries.

use swc_next_ecma_ast::{Ast, Comment, CommentKind, Span};

use super::{Formatter, source::is_source_whitespace};
use crate::{FormatError, document::Writer};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ContentMode {
    Formatted,
    Verbatim,
}

/// Source-run placement is intrinsic; the consumer decides whether it supplies
/// the terminal boundary needed by an own-line trailing suppression.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum TrailingSuppression {
    Inline,
    Terminal,
}

#[derive(Clone, Copy)]
pub(super) enum SuppressionPosition {
    OwnLine,
    EndOfLine,
    Inline,
}

pub(super) fn is_suppression_comment(ast: &Ast<'_>, comment: Comment) -> bool {
    let value = ast.get_utf8(comment.value(ast.source().as_bytes()));
    value.trim_matches(|character| is_source_whitespace(character) && character != '\u{85}')
        == "prettier-ignore"
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    #[inline]
    pub(super) fn suppression_position(
        &self,
        comments: &[Comment],
        following_start: u32,
    ) -> Option<SuppressionPosition> {
        if !self.has_suppression {
            return None;
        }
        self.suppressed_comment_runs(comments, following_start)
            .next()
            .map(|(_, position)| position)
    }

    /// Physical runs stop at punctuation and line breaks and share source placement.
    /// Syntax owners resolve each marker's actual target, including Block/Line differences.
    pub(super) fn suppressed_comment_runs<'a>(
        &'a self,
        comments: &'a [Comment],
        following_start: u32,
    ) -> impl Iterator<Item = (&'a [Comment], SuppressionPosition)> + 'a {
        let comments = if self.has_suppression { comments } else { &[] };
        let mut runs = comments
            .chunk_by(|left, right| {
                self.source(Span::new(left.span.end, right.span.start))
                    .chars()
                    .all(|character| {
                        is_source_whitespace(character)
                            && !matches!(character, '\n' | '\r' | '\u{2028}' | '\u{2029}')
                    })
            })
            .peekable();
        std::iter::from_fn(move || {
            while let Some(run) = runs.next() {
                if self.content_mode(run) != ContentMode::Verbatim {
                    continue;
                }
                let first = run.first()?;
                let last = run.last()?;
                let following = runs
                    .peek()
                    .and_then(|next| next.first())
                    .map_or(following_start, |comment| comment.span.start);
                let position = if self.source_position_starts_line(first.span.start) {
                    SuppressionPosition::OwnLine
                } else if self.comment_run_ends_line(*last, following) {
                    SuppressionPosition::EndOfLine
                } else {
                    SuppressionPosition::Inline
                };
                return Some((run, position));
            }
            None
        })
    }

    pub(super) fn comment_run_ends_line(&self, last: Comment, following: u32) -> bool {
        last.kind == CommentKind::Line
            || self
                .source(Span::new(last.span.end, following))
                .chars()
                .find(|&character| {
                    !is_source_whitespace(character)
                        || matches!(character, '\n' | '\r' | '\u{2028}' | '\u{2029}')
                })
                .is_some_and(|character| matches!(character, '\n' | '\r' | '\u{2028}' | '\u{2029}'))
    }

    pub(super) fn content_mode(&self, comments: &[Comment]) -> ContentMode {
        if self.has_suppression
            && comments
                .iter()
                .any(|&comment| is_suppression_comment(self.ast, comment))
        {
            ContentMode::Verbatim
        } else {
            ContentMode::Formatted
        }
    }

    pub(super) fn trailing_suppression_from_comments(
        &self,
        span: Span,
        comments: &[Comment],
        terminal: bool,
    ) -> Option<TrailingSuppression> {
        if !self.has_suppression {
            return None;
        }
        let mut previous = span.start;
        let mut position = TrailingSuppression::Inline;
        for comment in comments {
            let gap = self.source(Span::new(previous, comment.span.start));
            if !gap.chars().all(is_source_whitespace) {
                break;
            }
            if position == TrailingSuppression::Inline
                && self.line_breaks(previous, comment.span.start) != 0
            {
                if !terminal {
                    break;
                }
                position = TrailingSuppression::Terminal;
            }
            if is_suppression_comment(self.ast, *comment) {
                return Some(position);
            }
            previous = comment.span.end;
        }
        None
    }

    pub(super) fn format_verbatim(
        &mut self,
        span: Span,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let comments = self.comments.between(span)?;
        self.comments.claim(comments)?;
        writer.literal_text(self.source(span));
        Ok(())
    }
}
