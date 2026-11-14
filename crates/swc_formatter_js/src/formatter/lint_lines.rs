//! Immutable source-line coverage for line-scoped lint directives.

use swc_next_ecma_ast::{Ast, Span};

use super::Formatter;
use super::diagnostics::LineConstraint;
use super::suppression::{LintLineDirective, is_suppression_comment, lint_line_directive};

#[derive(Default)]
pub(super) struct LintLineCoverage {
    lines: Vec<Span>,
    suppression_markers: Vec<Span>,
    line_markers: Vec<Span>,
    has_directives: bool,
}

impl LintLineCoverage {
    pub(super) fn new(ast: &Ast<'_>, has_suppression: bool) -> Self {
        let source = ast.source();
        let suppression_markers = if has_suppression {
            ast.comments()
                .iter()
                .filter(|&&comment| is_suppression_comment(ast, comment))
                .map(|comment| comment.span)
                .collect()
        } else {
            Vec::new()
        };
        let mut directives = Vec::new();
        let mut line_markers = Vec::new();
        for &comment in ast.comments() {
            let Some(kind) = lint_line_directive(ast, comment) else {
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
            line_markers.push(comment.span);
            directives.push(((comment.span.end - 1) as usize, kind));
        }
        if directives.is_empty() {
            return Self {
                lines: Vec::new(),
                suppression_markers,
                line_markers,
                has_directives: false,
            };
        }
        let mut directives = directives.into_iter().peekable();
        let mut lines = Vec::new();
        let mut boundaries = source
            .match_indices(['\n', '\r', '\u{2028}', '\u{2029}'])
            .peekable();
        let mut start = 0;
        let mut pending = false;
        loop {
            let (end, next) = match boundaries.next() {
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
            let mut covered = pending;
            pending = false;
            while let Some((_, kind)) = directives.next_if(|(offset, _)| *offset < end) {
                match kind {
                    LintLineDirective::SameLine => covered = true,
                    LintLineDirective::NextLine => pending = true,
                }
            }
            if covered && start < end {
                lines.push(Span::new(start as u32, end as u32));
            }
            if next == source.len() && end == source.len()
                || directives.peek().is_none() && !pending
            {
                break;
            }
            start = next;
        }
        Self {
            lines,
            suppression_markers,
            line_markers,
            has_directives: true,
        }
    }

    pub(super) fn intersects(&self, span: Span) -> bool {
        let first = self.lines.partition_point(|line| line.end <= span.start);
        span.start < span.end
            && self
                .lines
                .get(first)
                .is_some_and(|line| line.start < span.end)
    }

    fn contains(&self, span: Span) -> bool {
        let first = self.lines.partition_point(|line| line.end <= span.start);
        span.start < span.end
            && self
                .lines
                .get(first)
                .is_some_and(|line| line.start <= span.start && span.end <= line.end)
    }

    fn has_line_directive_in(&self, span: Span) -> bool {
        let first = self
            .line_markers
            .partition_point(|marker| marker.start < span.start);
        self.line_markers
            .get(first)
            .is_some_and(|marker| marker.end <= span.end)
    }

    fn has_suppression(&self, span: Span) -> bool {
        let first = self
            .suppression_markers
            .partition_point(|marker| marker.start < span.start);
        self.suppression_markers
            .get(first)
            .is_some_and(|marker| marker.end <= span.end)
    }
}

impl Formatter<'_, '_> {
    fn source_directive_lines(&self) -> &LintLineCoverage {
        self.lint_lines
            .get_or_init(|| LintLineCoverage::new(self.ast, self.has_suppression))
    }

    pub(super) fn lint_line_intersects(&self, span: Span) -> bool {
        !self.ast.comments().is_empty() && self.source_directive_lines().intersects(span)
    }

    pub(super) fn has_lint_line_directives(&self) -> bool {
        !self.ast.comments().is_empty() && self.source_directive_lines().has_directives
    }

    pub(super) fn has_line_directive_in(&self, span: Span) -> bool {
        self.has_lint_line_directives() && self.source_directive_lines().has_line_directive_in(span)
    }

    pub(super) fn lint_line_contains(&self, span: Span) -> bool {
        !self.ast.comments().is_empty() && self.source_directive_lines().contains(span)
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
        (
            source,
            LintLineCoverage::new(unsafe { parsed.ast.ast() }, true),
        )
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
        let coverage = LintLineCoverage::new(ast, false);

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

        let (source, lines) = coverage("α; /* \u{feff}eslint-disable-line\u{85}rule */\nβ;");
        assert!(lines.intersects(text_span(source, "α")));
        assert!(!lines.intersects(text_span(source, "β")));

        let (_, lines) = coverage("α; /* eslint-disable-lineSuffix */\nβ;");
        assert!(!lines.has_directives);

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
