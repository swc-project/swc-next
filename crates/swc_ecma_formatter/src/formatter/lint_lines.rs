//! Immutable source-line coverage for line-scoped lint directives.

use swc_next_ecma_ast::{Ast, Span};

use super::Formatter;
use super::suppression::{LintLineDirective, lint_line_directive};

#[derive(Default)]
pub(super) struct LintLineCoverage {
    lines: Vec<Span>,
}

impl LintLineCoverage {
    pub(super) fn new(ast: &Ast<'_>) -> Self {
        let source = ast.source();
        let mut directives = ast
            .comments()
            .iter()
            .filter_map(|&comment| {
                let kind = lint_line_directive(ast, comment)?;
                let text = &source[comment.span.start as usize..comment.span.end as usize];
                // Line-scoped lint markers in multiline comments are invalid.
                if text.contains(['\r', '\n', '\u{2028}', '\u{2029}']) {
                    return None;
                }
                Some((comment.span.start as usize, kind))
            })
            .peekable();
        if directives.peek().is_none() {
            return Self { lines: Vec::new() };
        }

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
        Self { lines }
    }

    pub(super) fn intersects(&self, span: Span) -> bool {
        let first = self.lines.partition_point(|line| line.end <= span.start);
        span.start < span.end
            && self
                .lines
                .get(first)
                .is_some_and(|line| line.start < span.end)
    }
}

impl Formatter<'_, '_> {
    fn source_directive_lines(&self) -> &LintLineCoverage {
        self.lint_lines
            .get_or_init(|| LintLineCoverage::new(self.ast))
    }

    pub(super) fn lint_line_intersects(&self, span: Span) -> bool {
        !self.ast.comments().is_empty() && self.source_directive_lines().intersects(span)
    }
}

#[cfg(test)]
mod tests {
    use swc_next_allocator::Allocator;
    use swc_next_ecma_ast::{Lang, Span};
    use swc_next_ecma_parser::{CommentMode, NoTokenParserConfig, Options, Parser};

    use super::LintLineCoverage;

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
    fn line_coverage() {
        let (source, lines) = coverage("one; /* eslint-disable-line */ two;\nthree;");
        assert!(lines.intersects(text_span(source, "one")));
        assert!(lines.intersects(text_span(source, "two")));
        assert!(!lines.intersects(text_span(source, "three")));

        let (source, lines) = coverage("// eslint-disable-next-line\r\none;\r\ntwo;");
        assert!(lines.intersects(text_span(source, "one")));
        assert!(!lines.intersects(text_span(source, "two")));

        let (source, lines) = coverage("/* eslint-disable-next-line */\u{2028}one;\u{2029}two;");
        assert!(lines.intersects(text_span(source, "one")));
        assert!(!lines.intersects(text_span(source, "two")));

        let (source, lines) = coverage("one; // eslint-disable-line\ntwo;");
        assert!(lines.intersects(text_span(source, "one")));
        assert!(!lines.intersects(text_span(source, "two")));

        let (source, lines) = coverage("/* eslint-disable-next-line\n*/\none;");
        assert!(!lines.intersects(text_span(source, "one")));
    }
}
