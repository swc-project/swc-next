//! Immutable source-line coverage for diagnostic directives.

use swc_next_ecma_ast::{Comment, CommentKind, Span};

use super::Formatter;
use super::source::is_source_whitespace;
use crate::{FormatError, document::Writer};

pub(super) struct DiagnosticLines {
    lines: Vec<DiagnosticLine>,
    directive_ends: Vec<u32>,
    document_closing: Option<u32>,
}

struct DiagnosticLine {
    span: Span,
    sensitive_start: u32,
    terminal_suffix_start: u32,
}

/// Final source punctuation either stays put or moves with its prepared statement envelope.
#[derive(Clone, Copy)]
pub(super) enum TerminalCloseOwner {
    Source,
    Statement { original: u32, actual: u32 },
}

/// Original list punctuation and its immutable covered-source frontier.
#[derive(Clone, Copy)]
pub(super) struct TerminalCloseBoundary {
    close: u32,
    last_end: Option<u32>,
    release: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum LineConstraint {
    Unconstrained,
    SameLine,
    NewLine(u8),
}

#[derive(Clone, Copy)]
pub(super) enum FlatSeparator {
    Empty,
    Space,
}

impl LineConstraint {
    pub(super) fn write<'a>(
        self,
        writer: &mut Writer<'a>,
        flat: FlatSeparator,
        default: impl FnOnce(&mut Writer<'a>),
    ) {
        match self {
            Self::Unconstrained => default(writer),
            Self::SameLine => {
                if matches!(flat, FlatSeparator::Space) {
                    writer.space();
                }
            }
            Self::NewLine(count) => {
                writer.hard_line();
                if count == 2 {
                    writer.hard_line();
                }
            }
        }
    }
}

impl DiagnosticLines {
    pub(super) fn new(source: &str, comments: &[Comment]) -> Self {
        // Keep the immutable raw identity. Only an explicitly removed complete
        // statement envelope may transfer it to its selected actual closer.
        let document_closing = source
            .trim_end_matches(|character| is_source_whitespace(character) || character == ';')
            .char_indices()
            .next_back()
            .filter(|&(_, character)| matches!(character, ')' | ']' | '}'))
            .map(|(position, _)| position as u32);
        let directive_ends = comments
            .iter()
            .filter(|&&comment| is_diagnostic_directive(source, comment))
            .map(|comment| comment.span.end)
            .collect::<Vec<_>>();
        let mut lines = Vec::new();
        if directive_ends.is_empty() {
            return Self {
                lines,
                directive_ends,
                document_closing,
            };
        }
        let mut directives = directive_ends.iter().copied().peekable();
        let mut boundaries = source
            .match_indices(['\n', '\r', '\u{2028}', '\u{2029}'])
            .peekable();
        let mut start = 0;
        let mut covered = false;
        let mut comment_index = 0;
        loop {
            let (end, next_start) = match boundaries.next() {
                Some((end, terminator)) => {
                    let mut next = end + terminator.len();
                    if terminator == "\r" && boundaries.peek() == Some(&(next, "\n")) {
                        boundaries.next();
                        next += 1;
                    }
                    (end, next)
                }
                None => (source.len(), source.len()),
            };
            let mut directive_end = None;
            while let Some(comment_end) =
                directives.next_if(|&comment_end| comment_end as usize <= end)
            {
                directive_end.get_or_insert(comment_end);
            }
            let sensitive_start = match (covered, directive_end) {
                (true, _) => Some(start as u32),
                (false, end) => end,
            };
            if let Some(sensitive_start) = sensitive_start {
                let terminal_suffix_start =
                    terminal_suffix_start(source, comments, &mut comment_index, start, end);
                lines.push(DiagnosticLine {
                    span: Span::new(start as u32, end as u32),
                    sensitive_start,
                    terminal_suffix_start,
                });
            }
            let text = source[start..end].trim_matches(is_directive_whitespace);
            covered =
                directive_end.is_some() || covered && (text.is_empty() || text.starts_with("//"));
            if end == source.len() || !covered && directives.peek().is_none() {
                break;
            }
            start = next_start;
        }
        Self {
            lines,
            directive_ends,
            document_closing,
        }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    fn covered_terminal_suffix_contains(&self, close: u32) -> bool {
        let index = self.lines.partition_point(|line| line.span.end < close);
        self.lines.get(index).is_some_and(|line| {
            line.span.start <= close
                && line.sensitive_start <= close
                && line.terminal_suffix_start <= close
        })
    }

    fn sensitive_at(&self, position: u32) -> bool {
        let index = self.lines.partition_point(|line| line.span.end < position);
        self.lines
            .get(index)
            .is_some_and(|line| line.span.start <= position && line.sensitive_start <= position)
    }
}

// Each covered line and parser comment is visited monotonically. Unknown
// source characters (including quotes, regex slashes and template backticks)
// end the terminal suffix; only actual trivia and closing punctuation pass.
fn terminal_suffix_start(
    source: &str,
    comments: &[Comment],
    comment_index: &mut usize,
    start: usize,
    end: usize,
) -> u32 {
    while comments
        .get(*comment_index)
        .is_some_and(|comment| comment.span.end as usize <= start)
    {
        *comment_index += 1;
    }
    let mut position = start;
    let mut suffix = start;
    while position < end {
        let comment = comments.get(*comment_index);
        if let Some(comment) = comment
            && comment.span.start as usize <= position
        {
            position = (comment.span.end as usize).min(end);
            if comment.span.end as usize <= end {
                *comment_index += 1;
            }
            continue;
        }
        let gap_end = comment.map_or(end, |comment| (comment.span.start as usize).min(end));
        for (offset, character) in source[position..gap_end].char_indices() {
            if !is_source_whitespace(character) && !matches!(character, ')' | ']' | '}' | ',' | ';')
            {
                suffix = position + offset + character.len_utf8();
            }
        }
        position = gap_end;
    }
    suffix as u32
}

fn is_diagnostic_directive(source: &str, comment: Comment) -> bool {
    let source = &source[comment.span.start as usize..comment.span.end as usize];
    let marker = match comment.kind {
        CommentKind::Line => source
            .strip_prefix("///")
            .or_else(|| source.strip_prefix("//"))
            .unwrap_or(source),
        CommentKind::Block => {
            if !source.contains('@') {
                return false;
            }
            source
                .rsplit_once(['\n', '\r', '\u{2028}', '\u{2029}'])
                .map_or(source, |(_, last)| last)
                .trim_start_matches(is_directive_whitespace)
                .trim_start_matches(['/', '*'])
        }
    };
    let marker = marker.trim_start_matches(is_directive_whitespace);
    marker.starts_with("@ts-ignore") || marker.starts_with("@ts-expect-error")
}

fn is_directive_whitespace(character: char) -> bool {
    character == '\u{feff}' || character.is_whitespace() && character != '\u{85}'
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn statement_terminal_close_owner(
        &self,
        original: Span,
        actual: Span,
    ) -> Option<TerminalCloseOwner> {
        let source_close = original.end.checked_sub(1);
        let actual_close = actual.end.checked_sub(1);
        match (source_close, actual_close) {
            (Some(original), Some(actual))
                if self.diagnostic_lines.document_closing == Some(original)
                    && self
                        .ast
                        .source()
                        .as_bytes()
                        .get(actual as usize)
                        .is_some_and(|byte| matches!(byte, b')' | b']' | b'}')) =>
            {
                Some(TerminalCloseOwner::Statement { original, actual })
            }
            _ => None,
        }
    }

    /// A prepared removed envelope owns this synchronous content walk, including
    /// nested operand printers. Exact source identities prevent unrelated lists
    /// from claiming its final token. Restore the outer owner on every Result.
    pub(super) fn with_terminal_close_owner<T>(
        &mut self,
        owner: Option<TerminalCloseOwner>,
        write: impl FnOnce(&mut Self) -> Result<T, FormatError>,
    ) -> Result<T, FormatError> {
        let previous = self.terminal_close_owner;
        if let Some(owner) = owner {
            self.terminal_close_owner = owner;
        }
        let result = write(self);
        self.terminal_close_owner = previous;
        result
    }

    pub(super) fn terminal_close_boundary(
        &self,
        content: Span,
        prepare_last: impl FnOnce() -> Option<(Span, u32)>,
    ) -> TerminalCloseBoundary {
        // Only a covered source closer can release its terminal item. Keep
        // ordinary lists out of the item and comment ownership preparation.
        let last = self
            .diagnostic_lines
            .covered_terminal_suffix_contains(content.end)
            .then(prepare_last)
            .flatten();
        let release = last.is_some_and(|(last, head_start)| {
            self.comments.in_range(last).is_empty()
                && self
                    .comments
                    .in_range(Span::new(last.end, content.end))
                    .is_empty()
                && !self
                    .comments
                    .in_range(Span::new(head_start, last.start))
                    .iter()
                    .any(|&comment| self.comment_semantics(comment).suppression)
        });
        TerminalCloseBoundary {
            close: content.end,
            last_end: last.map(|(last, _)| last.end),
            release,
        }
    }

    pub(super) fn terminal_close_constraint(
        &self,
        boundary: TerminalCloseBoundary,
        end: u32,
    ) -> LineConstraint {
        let constraint = self.line_constraint(end, boundary.close);
        if !boundary.release || constraint != LineConstraint::SameLine {
            return constraint;
        }
        let document_close = match self.terminal_close_owner {
            TerminalCloseOwner::Source => self.diagnostic_lines.document_closing,
            TerminalCloseOwner::Statement { original, actual }
                if self.diagnostic_lines.document_closing == Some(original) =>
            {
                Some(actual)
            }
            TerminalCloseOwner::Statement { .. } => None,
        };
        if document_close == Some(boundary.close)
            && boundary.last_end.is_some_and(|last| end >= last)
        {
            LineConstraint::Unconstrained
        } else {
            constraint
        }
    }

    pub(super) fn diagnostic_line_is_sensitive(&self, position: u32) -> bool {
        self.diagnostic_lines.sensitive_at(position)
    }

    pub(super) fn line_constraint(&self, start: u32, end: u32) -> LineConstraint {
        if !self.diagnostic_line_is_sensitive(start) {
            LineConstraint::Unconstrained
        } else {
            self.source_line_constraint(start, end)
        }
    }

    pub(super) fn source_line_constraint(&self, start: u32, end: u32) -> LineConstraint {
        match self.line_breaks(start, end) {
            0 => LineConstraint::SameLine,
            count => LineConstraint::NewLine(count),
        }
    }

    pub(super) fn line_constraint_after_comments(
        &self,
        comments: &[Comment],
        start: u32,
        end: u32,
    ) -> LineConstraint {
        let previous = comments.last().map_or(start, |comment| comment.span.end);
        if comments
            .last()
            .is_some_and(|comment| comment.kind == CommentKind::Line)
        {
            self.source_line_constraint(previous, end)
        } else {
            self.line_constraint(previous, end)
        }
    }

    pub(super) fn diagnostic_range_is_sensitive(&self, span: Span) -> bool {
        let lines = &self.diagnostic_lines.lines;
        let first = lines.partition_point(|line| line.span.end < span.start);
        lines
            .get(first)
            .is_some_and(|line| line.sensitive_start <= span.end)
    }

    pub(super) fn opening_line_constraint(&self, span: Span) -> LineConstraint {
        if !self.diagnostic_line_is_sensitive(span.start) {
            return LineConstraint::Unconstrained;
        }
        let content = self.source(span).trim_start_matches(is_source_whitespace);
        self.line_constraint(span.start, span.end - content.len() as u32)
    }

    pub(super) fn comment_is_diagnostic_directive(&self, comment: Comment) -> bool {
        self.diagnostic_lines
            .directive_ends
            .binary_search(&comment.span.end)
            .is_ok()
    }
}
