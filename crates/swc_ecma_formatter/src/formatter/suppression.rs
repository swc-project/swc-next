//! Ignore targets resolved at their owning syntax boundaries.

use swc_next_ecma_ast::{Ast, Comment, CommentKind, Span};

use super::{Formatter, diagnostics::LineConstraint, source::is_source_whitespace};
use crate::{FormatError, document::Writer};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ContentMode {
    Formatted,
    Verbatim,
}

#[derive(Clone, Copy)]
pub(super) enum DelimitedFollowing {
    Node,
    End,
}

#[derive(Clone, Copy)]
pub(super) struct DelimitedSourceGap {
    pub(super) end: u32,
    pub(super) following: LineConstraint,
}

#[derive(Clone, Copy)]
pub(super) struct DelimitedSuppression {
    pub(super) preceding: Option<Span>,
    pub(super) following: ContentMode,
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

    pub(super) fn suppression_positions<'a>(
        &'a self,
        comments: &'a [Comment],
        following_start: u32,
    ) -> impl Iterator<Item = SuppressionPosition> + 'a {
        self.suppressed_comment_runs(comments, following_start)
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

    pub(super) fn delimited_suppression(
        &self,
        target: Span,
        end: u32,
        following: DelimitedFollowing,
    ) -> Option<DelimitedSuppression> {
        if !self.has_suppression {
            return None;
        }
        self.resolve_delimited_suppression(target, end, following)
    }

    fn resolve_delimited_suppression(
        &self,
        target: Span,
        end: u32,
        following: DelimitedFollowing,
    ) -> Option<DelimitedSuppression> {
        let span = Span::new(target.end, end);
        let comments = self.comments.in_range(span);
        if self.content_mode(comments) != ContentMode::Verbatim {
            return None;
        }
        let mut preceding_target = None;
        let mut following_mode = ContentMode::Formatted;
        let mut first = 0;
        while first < comments.len() {
            let mut last = first + 1;
            while last < comments.len() {
                let gap = Span::new(comments[last - 1].span.end, comments[last].span.start);
                if self.line_breaks(gap.start, gap.end) != 0
                    || !self.source(gap).chars().all(is_source_whitespace)
                {
                    break;
                }
                last += 1;
            }
            let run = &comments[first..last];
            if self.content_mode(run) == ContentMode::Verbatim {
                let after = run[run.len() - 1];
                let next = comments.get(last).map_or(end, |comment| comment.span.start);
                let end_of_line = self.comment_run_ends_line(after, next);
                let preceding = matches!(following, DelimitedFollowing::End)
                    || !self.source_position_starts_line(run[0].span.start)
                        && (end_of_line
                            || self
                                .find_token(Span::new(after.span.end, end), &comments[last..], ",")
                                .is_some());
                if preceding {
                    preceding_target = Some(target);
                } else {
                    following_mode = ContentMode::Verbatim;
                }
            }
            first = last;
        }
        Some(DelimitedSuppression {
            preceding: preceding_target,
            following: following_mode,
        })
    }

    /// Preserve an effective trailing ignore's complete delimiter relationship.
    /// Node and slot ownership are resolved before this claims the borrowed run.
    pub(super) fn format_delimited_source_gap(
        &mut self,
        gap: Span,
        following: DelimitedFollowing,
        writer: &mut Writer<'ast>,
    ) -> Result<DelimitedSourceGap, FormatError> {
        let comments = self.comments.in_range(gap);
        let source_end = gap.start
            + self
                .source(gap)
                .trim_end_matches(is_source_whitespace)
                .len() as u32;
        let last = comments
            .last()
            .filter(|comment| comment.span.end == source_end);
        let following = match following {
            DelimitedFollowing::Node if last.is_some() => {
                self.source_line_constraint(source_end, gap.end)
            }
            DelimitedFollowing::Node => self.line_constraint(source_end, gap.end),
            DelimitedFollowing::End => {
                // The closing delimiter owns its indentation and newline. Literal
                // source text has no pending suffix to force that closing break.
                if last.is_some_and(|comment| comment.kind == CommentKind::Line) {
                    writer.break_parent();
                }
                LineConstraint::Unconstrained
            }
        };
        self.format_verbatim(Span::new(gap.start, source_end), writer)?;
        Ok(DelimitedSourceGap {
            end: source_end,
            following,
        })
    }

    pub(super) fn trailing_content_mode(&self, span: Span, terminal: bool) -> ContentMode {
        if self
            .trailing_suppression_from_comments(span, self.comments.in_range(span), terminal)
            .is_some()
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
