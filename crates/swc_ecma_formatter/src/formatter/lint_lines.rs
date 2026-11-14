//! Immutable source-line coverage for line-scoped lint directives.

use swc_next_ecma_ast::{Ast, Comment, Span};

use super::Formatter;
use super::diagnostics::LineConstraint;
use super::suppression::{
    CommentDirective, LintLineDirective, comment_directive_value, is_suppression_value,
};

const LINE_TERMINATORS: [char; 4] = ['\n', '\r', '\u{2028}', '\u{2029}'];

#[derive(Clone, Copy)]
struct SourceLine {
    start: usize,
    end: usize,
    next: usize,
}

impl SourceLine {
    fn containing(source: &str, offset: usize, lower_bound: usize) -> Self {
        let start =
            source[lower_bound..offset]
                .rfind(LINE_TERMINATORS)
                .map_or(lower_bound, |index| {
                    let terminator = lower_bound + index;
                    let width = if source.as_bytes()[terminator].is_ascii() {
                        1
                    } else {
                        '\u{2028}'.len_utf8()
                    };
                    terminator + width
                });
        Self::from_start(source, start, offset)
    }

    fn from_start(source: &str, start: usize, search_start: usize) -> Self {
        let end = source[search_start..]
            .find(LINE_TERMINATORS)
            .map_or(source.len(), |index| search_start + index);
        let next = match source.as_bytes().get(end) {
            Some(b'\r') if source.as_bytes().get(end + 1) == Some(&b'\n') => end + 2,
            Some(b'\r' | b'\n') => end + 1,
            Some(_) => end + '\u{2028}'.len_utf8(),
            None => end,
        };
        Self { start, end, next }
    }
}

fn push_line(lines: &mut Vec<Span>, line: SourceLine) {
    if line.start == line.end {
        return;
    }
    let span = Span::new(line.start as u32, line.end as u32);
    if lines.last() != Some(&span) {
        lines.push(span);
    }
}

fn push_covered_lines(
    source: &str,
    lines: &mut Vec<Span>,
    line: SourceLine,
    same_line: bool,
    next_line: bool,
) {
    if same_line {
        push_line(lines, line);
    }
    if next_line && line.next < source.len() {
        push_line(lines, SourceLine::from_start(source, line.next, line.next));
    }
}

#[derive(Clone, Copy)]
struct DirectiveMarker {
    span: Span,
    kind: CommentDirective,
}

#[derive(Default)]
pub(super) struct LintLineCoverage {
    lines: Vec<Span>,
    suppression_markers: Vec<Span>,
    directives: Vec<DirectiveMarker>,
    // Raw directive facts include invalid multiline same-line markers;
    // only effective line markers enter this source-ordered index.
    line_markers: Vec<u32>,
    has_line_directives: bool,
}

impl LintLineCoverage {
    pub(super) fn new(ast: &Ast<'_>) -> Self {
        let source = ast.source();
        let mut suppression_markers = Vec::new();
        let mut directives = Vec::new();
        let mut line_markers = Vec::new();
        let mut lines = Vec::new();
        let mut line: Option<SourceLine> = None;
        let mut same_line = false;
        let mut next_line = false;
        for &comment in ast.comments() {
            let value = ast.get_utf8(comment.value(source.as_bytes()));
            if is_suppression_value(value) {
                suppression_markers.push(comment.span);
            }
            let Some(directive) = comment_directive_value(value, comment.kind) else {
                continue;
            };
            let index = directives.len() as u32;
            directives.push(DirectiveMarker {
                span: comment.span,
                kind: directive,
            });
            let CommentDirective::LintLine(kind) = directive else {
                continue;
            };
            // A same-line directive is invalid in a multiline block. A
            // next-line directive instead uses the comment's ending line.
            if matches!(kind, LintLineDirective::SameLine)
                && source[comment.span.start as usize..comment.span.end as usize]
                    .contains(['\r', '\n', '\u{2028}', '\u{2029}'])
            {
                continue;
            }
            line_markers.push(index);
            let offset = comment.span.end as usize;
            // Reuse each line for all of its markers; later searches cannot
            // revisit the preceding line even when a comment spans lines.
            if line.is_none_or(|line| offset > line.end) {
                let lower_bound = if let Some(previous) = line {
                    push_covered_lines(source, &mut lines, previous, same_line, next_line);
                    previous.next
                } else {
                    0
                };
                line = Some(SourceLine::containing(source, offset, lower_bound));
                same_line = false;
                next_line = false;
            }
            match kind {
                LintLineDirective::SameLine => same_line = true,
                LintLineDirective::NextLine => next_line = true,
            }
        }
        if let Some(line) = line {
            push_covered_lines(source, &mut lines, line, same_line, next_line);
        }
        Self {
            lines,
            suppression_markers,
            directives,
            has_line_directives: !line_markers.is_empty(),
            line_markers,
        }
    }

    pub(super) fn intersects(&self, span: Span) -> bool {
        if span.start >= span.end
            || self.lines.first().is_none_or(|line| span.end <= line.start)
            || self.lines.last().is_some_and(|line| line.end <= span.start)
        {
            return false;
        }
        let first = self.lines.partition_point(|line| line.end <= span.start);
        self.lines
            .get(first)
            .is_some_and(|line| line.start < span.end)
    }

    fn line_at(&self, offset: u32) -> Option<Span> {
        let first = self.lines.partition_point(|line| line.end <= offset);
        self.lines
            .get(first)
            .filter(|line| line.start <= offset)
            .copied()
    }

    fn first_marker_at_or_after(&self, start: u32) -> Option<&DirectiveMarker> {
        let first = self
            .line_markers
            .partition_point(|&index| self.directives[index as usize].span.start < start);
        self.line_markers
            .get(first)
            .map(|&index| &self.directives[index as usize])
    }

    fn has_line_directive_in(&self, span: Span) -> bool {
        self.first_marker_at_or_after(span.start)
            .is_some_and(|marker| marker.span.end <= span.end)
    }

    fn is_next_line_directive(&self, span: Span) -> bool {
        self.first_marker_at_or_after(span.start)
            .is_some_and(|marker| {
                marker.span == span
                    && matches!(
                        marker.kind,
                        CommentDirective::LintLine(LintLineDirective::NextLine)
                    )
            })
    }

    fn first_directive_at_or_after(&self, start: u32) -> Option<&DirectiveMarker> {
        let first = self
            .directives
            .partition_point(|marker| marker.span.start < start);
        self.directives.get(first)
    }

    fn comment_directive(&self, span: Span) -> Option<CommentDirective> {
        self.first_directive_at_or_after(span.start)
            .filter(|marker| marker.span == span)
            .map(|marker| marker.kind)
    }

    fn has_comment_directive_in(&self, span: Span) -> bool {
        self.first_directive_at_or_after(span.start)
            .is_some_and(|marker| marker.span.end <= span.end)
    }

    fn has_suppression(&self, span: Span) -> bool {
        let first = self
            .suppression_markers
            .partition_point(|marker| marker.start < span.start);
        self.suppression_markers
            .get(first)
            .is_some_and(|marker| marker.end <= span.end)
    }

    pub(super) fn has_suppressions(&self) -> bool {
        !self.suppression_markers.is_empty()
    }
}

impl Formatter<'_, '_> {
    fn source_directive_lines(&self) -> &LintLineCoverage {
        &self.lint_lines
    }

    pub(super) fn lint_line_intersects(&self, span: Span) -> bool {
        self.source_directive_lines().intersects(span)
    }

    pub(super) fn has_lint_line_directives(&self) -> bool {
        self.source_directive_lines().has_line_directives
    }

    pub(super) fn comment_directive(&self, comment: Comment) -> Option<CommentDirective> {
        self.source_directive_lines()
            .comment_directive(comment.span)
    }

    pub(super) fn has_comment_directive_in(&self, span: Span) -> bool {
        self.source_directive_lines().has_comment_directive_in(span)
    }

    pub(super) fn has_line_directive_in(&self, span: Span) -> bool {
        self.has_lint_line_directives() && self.source_directive_lines().has_line_directive_in(span)
    }

    pub(super) fn is_next_line_directive(&self, span: Span) -> bool {
        let lines = self.source_directive_lines();
        lines.has_line_directives && lines.is_next_line_directive(span)
    }

    pub(super) fn lint_line_contains(&self, span: Span) -> bool {
        span.start < span.end
            && self
                .lint_line_at(span.start)
                .is_some_and(|line| span.end <= line.end)
    }

    pub(super) fn lint_line_at(&self, offset: u32) -> Option<Span> {
        self.source_directive_lines().line_at(offset)
    }

    pub(super) fn lint_opening_constraint(&self, start: u32, first: u32) -> Option<LineConstraint> {
        if start == 0 || start > first || !self.lint_line_intersects(Span::new(start - 1, start)) {
            return None;
        }
        let comments = self.ast.comments();
        let index = comments.partition_point(|comment| comment.span.start < start);
        let next = comments
            .get(index)
            .filter(|comment| comment.span.end <= first)
            .map_or(first, |comment| comment.span.start);
        Some(self.source_line_constraint(start, next))
    }

    pub(super) fn same_covered_lint_line(&self, left: u32, right: u32) -> bool {
        left > 0
            && left <= right
            && (right as usize) < self.ast.source().len()
            && self.has_lint_line_directives()
            && self.source_line_breaks(left, right) == 0
            && self.lint_line_intersects(Span::new(left - 1, right + 1))
    }

    pub(super) fn has_suppression_in(&self, span: Span) -> bool {
        self.has_suppression && self.source_directive_lines().has_suppression(span)
    }
}

#[cfg(test)]
mod tests {
    use swc_next_allocator::Allocator;
    use swc_next_ecma_ast::{Lang, Span};
    use swc_next_ecma_parser::{CommentMode, NoTokenParserConfig, Options, Parser};

    use super::LintLineCoverage;
    use crate::formatter::comments::CommentCursor;

    fn coverage(source: &str) -> (&str, LintLineCoverage) {
        let allocator = Allocator::new();
        let options = Options {
            lang: Lang::Ts,
            comments: CommentMode::Flat,
            ..Options::default()
        };
        let parsed = Parser::init(&allocator, source, options, NoTokenParserConfig).parse();
        assert!(parsed.diagnostics.is_empty());
        // SAFETY: All inspected comments come from this unmodified parsed AST.
        (source, LintLineCoverage::new(unsafe { parsed.ast.ast() }))
    }

    fn text_span(source: &str, text: &str) -> Span {
        let start = source.find(text).unwrap() as u32;
        Span::new(start, start + text.len() as u32)
    }

    #[test]
    fn source_directive_comments() {
        let source = "a, /* eslint-disable-next-line */\nb";
        let allocator = Allocator::new();
        let options = Options {
            comments: CommentMode::Flat,
            ..Options::default()
        };
        let parsed = Parser::init(&allocator, source, options, NoTokenParserConfig).parse();
        assert!(parsed.diagnostics.is_empty());
        // SAFETY: The parsed AST is not modified while its comments are queried.
        let ast = unsafe { parsed.ast.ast() };
        let span = ast.comments()[0].span;
        let mut cursor = CommentCursor::new(ast.comments());
        let coverage = LintLineCoverage::new(ast);

        assert!(coverage.has_line_directive_in(span));
        cursor.take(1);
        assert!(cursor.in_range(span).is_empty());
        assert!(coverage.has_line_directive_in(span));
        assert!(!coverage.has_line_directive_in(Span::new(0, span.start)));
    }

    #[test]
    fn line_coverage() {
        let (source, lines) = coverage("one; /* eslint-disable-line */ two;\nthree;");
        assert!(lines.intersects(text_span(source, "one")));
        assert!(lines.intersects(text_span(source, "two")));
        assert!(!lines.intersects(text_span(source, "three")));

        let (source, lines) = coverage(
            "α; /* eslint-disable-next-line */ /* eslint-disable-line */\nβ; /* eslint-disable-line */\nγ;",
        );
        let first = text_span(source, "α");
        let second = text_span(source, "β");
        let first_end = source.find('\n').unwrap() as u32;
        let second_end = source.rfind('\n').unwrap() as u32;
        assert_eq!(lines.line_at(first.start), Some(Span::new(0, first_end)));
        assert_eq!(
            lines.line_at(second.start),
            Some(Span::new(first_end + 1, second_end))
        );
        assert_eq!(lines.line_at(first_end), None);
        assert!(!lines.intersects(Span::new(first_end, first_end)));
        assert!(!lines.intersects(text_span(source, "γ")));

        let (source, lines) = coverage("α; /* \u{feff}eslint-disable-line\u{85}rule */\nβ;");
        assert!(lines.intersects(text_span(source, "α")));
        assert!(!lines.intersects(text_span(source, "β")));

        let (_, lines) = coverage("α; /* eslint-disable-lineSuffix */\nβ;");
        assert!(!lines.has_line_directives);

        let (source, lines) = coverage("// eslint-disable-next-line\r\none;\r\ntwo;");
        assert!(lines.intersects(text_span(source, "one")));
        assert!(!lines.intersects(text_span(source, "two")));

        let (source, lines) = coverage("/* eslint-disable-next-line */\u{2028}one;\u{2029}two;");
        assert!(lines.intersects(text_span(source, "one")));
        assert!(!lines.intersects(text_span(source, "two")));

        let (source, lines) = coverage("one; // eslint-disable-line\ntwo;");
        assert!(lines.intersects(text_span(source, "one")));
        assert!(!lines.intersects(text_span(source, "two")));

        for newline in ["\n", "\r", "\r\n", "\u{2028}", "\u{2029}"] {
            let source = format!(
                "/* eslint-disable-next-line{newline} no-undef{newline}*/ same;{newline}one;{newline}two;"
            );
            let (_, lines) = coverage(&source);
            assert!(!lines.intersects(text_span(&source, "same")));
            assert!(lines.intersects(text_span(&source, "one")));
            assert!(!lines.intersects(text_span(&source, "two")));
        }

        let (source, lines) = coverage("/* eslint-disable-line\n*/ one;");
        assert!(!lines.intersects(text_span(source, "one")));

        let (source, lines) = coverage("/* eslint-disable-next-line\n*/\n\none;");
        assert!(!lines.intersects(text_span(source, "one")));

        let (source, lines) = coverage("/* prettier-ignore */ one; two;");
        assert!(lines.has_suppression(text_span(source, "/* prettier-ignore */")));
        assert!(!lines.has_suppression(text_span(source, "two")));
    }
}
