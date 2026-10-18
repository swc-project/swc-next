//! Immutable source-line coverage for diagnostic directives.

use swc_next_ecma_ast::{Comment, CommentKind, Span};

use super::Formatter;
use super::source::is_source_whitespace;
use crate::document::Writer;

pub(super) struct DiagnosticLines {
    lines: Vec<DiagnosticLine>,
    directive_ends: Vec<u32>,
}

struct DiagnosticLine {
    span: Span,
    sensitive_start: u32,
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
            };
        }
        let mut directives = directive_ends.iter().copied().peekable();
        let mut boundaries = source
            .match_indices(['\n', '\r', '\u{2028}', '\u{2029}'])
            .peekable();
        let mut start = 0;
        let mut covered = false;
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
                lines.push(DiagnosticLine {
                    span: Span::new(start as u32, end as u32),
                    sensitive_start,
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
        }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    fn sensitive_at(&self, position: u32) -> bool {
        let index = self.lines.partition_point(|line| line.span.end < position);
        self.lines
            .get(index)
            .is_some_and(|line| line.span.start <= position && line.sensitive_start <= position)
    }
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
