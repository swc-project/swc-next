use std::cell::OnceCell;

use swc_next_markdown_parser::{
    Document, Inline, InlineKind, delimiter_flanking, emphasis_runs_match,
};

use super::is_word_char;

#[derive(Clone, Copy, Default)]
enum Spelling {
    #[default]
    Check,
    Preferred,
    Source,
}

#[derive(Clone, Copy, Default)]
pub(super) struct Emphasis {
    italic: bool,
    strong_adjoins_word: bool,
    spelling: Spelling,
}

impl Emphasis {
    pub(super) fn inside_inline(self) -> Self {
        Self {
            strong_adjoins_word: false,
            ..self
        }
    }

    pub(super) fn inside_link(self) -> Self {
        // Delimiters in a link label are paired in their own scope.
        Self {
            spelling: Spelling::Check,
            ..self.inside_inline()
        }
    }
}

pub(super) struct EmphasisScope<'a> {
    document: &'a Document<'a>,
    nodes: &'a [Inline],
    literals: OnceCell<LiteralRuns>,
}

struct Pair<'a> {
    original: &'a str,
    preferred: &'a str,
    context: Emphasis,
}

impl<'a> EmphasisScope<'a> {
    pub(super) fn new(document: &'a Document<'a>, nodes: &'a [Inline]) -> Self {
        Self {
            document,
            nodes,
            literals: OnceCell::new(),
        }
    }

    fn neighbors(&self, index: usize) -> (Option<&Inline>, Option<&Inline>) {
        (
            index.checked_sub(1).map(|i| &self.nodes[i]),
            self.nodes.get(index + 1),
        )
    }

    fn preferred(&self, index: usize, children: &[Inline], context: Emphasis) -> Pair<'a> {
        let node = &self.nodes[index];
        let strong = matches!(node.kind, InlineKind::Strong(_));
        let original = &node.span.text(self.document.source)[..if strong { 2 } else { 1 }];
        let adjoins_word = || {
            let (before, after) = self.neighbors(index);
            before
                .and_then(|n| n.span.text(self.document.source).chars().next_back())
                .is_some_and(is_word_char)
                || after
                    .and_then(|n| n.span.text(self.document.source).chars().next())
                    .is_some_and(is_word_char)
        };
        let preferred = if strong {
            "**"
        } else if children
            .first()
            .is_some_and(|n| matches!(n.kind, InlineKind::Autolink))
        {
            original
        } else if context.italic || context.strong_adjoins_word || adjoins_word() {
            "*"
        } else {
            "_"
        };
        Pair {
            original,
            preferred,
            context: Emphasis {
                italic: !strong || context.italic,
                // Only directly nested emphasis consults this flag when choosing its marker.
                strong_adjoins_word: strong
                    && children
                        .iter()
                        .any(|n| matches!(n.kind, InlineKind::Emphasis(_)))
                    && adjoins_word(),
                ..context
            },
        }
    }

    pub(super) fn pair(
        &self,
        index: usize,
        children: &[Inline],
        context: Emphasis,
    ) -> (&'a str, Emphasis) {
        let pair = self.preferred(index, children, context);
        let spelling = match context.spelling {
            Spelling::Check if self.can_normalize(index, children, &pair) => Spelling::Preferred,
            Spelling::Check => Spelling::Source,
            decided => decided,
        };
        let marker = if matches!(spelling, Spelling::Source) {
            pair.original
        } else {
            pair.preferred
        };
        (
            marker,
            Emphasis {
                spelling,
                ..pair.context
            },
        )
    }

    // Text and escape nodes keep their source spelling. Check a maximal
    // emphasis subtree once, before choosing its delimiters as a unit.
    fn can_normalize(&self, index: usize, children: &[Inline], pair: &Pair<'_>) -> bool {
        let (before, after) = self.neighbors(index);
        if [before, after].into_iter().flatten().any(is_pair) {
            return false;
        }
        let nested = children
            .iter()
            .any(|n| is_pair(n) || matches!(n.kind, InlineKind::Delete(_)));
        if !nested && pair.original == pair.preferred {
            return true;
        }
        let left = before.and_then(|n| n.span.text(self.document.source).chars().next_back());
        let right = after.and_then(|n| n.span.text(self.document.source).chars().next());
        let literals = || {
            self.literals
                .get_or_init(|| LiteralRuns::in_scope(self.document.source, self.nodes))
        };
        if nested {
            if literals().count != 0 {
                return false;
            }
            let mut parts = Vec::new();
            return self.delimiters(index, children, pair, &mut parts)
                && nested_pairs_match(self.document.source, &parts, left, right);
        }

        let mark = pair.marker();
        let text = self.nodes[index].span.text(self.document.source);
        let body = &text[pair.original.len()..text.len() - pair.original.len()];
        let (opens, also_closes) = delimiter_flanking(mark, left, body.chars().next());
        if !opens
            || !delimiter_flanking(mark, body.chars().next_back(), right).1
            || also_closes && literals().open & bit(mark) != 0
        {
            return false;
        }
        let touches_text = before
            .is_some_and(|n| matches!(n.kind, InlineKind::Text) && is_marker(left))
            || after.is_some_and(|n| matches!(n.kind, InlineKind::Text) && is_marker(right));
        // A lone leftover of the old kind cannot pair. Other splits and
        // merges can change how many markers the next delimiter consumes.
        if touches_text
            && (left == Some(mark)
                || right == Some(mark)
                || !literals().isolated()
                || children.iter().any(|n| self.is_literal_text(n)))
        {
            return false;
        }

        !children
            .iter()
            .filter(|n| matches!(n.kind, InlineKind::Text))
            .any(|node| {
                marker_runs(self.document.source, node).any(|(literal, left, right)| {
                    literal == mark && delimiter_flanking(mark, left, right) != (false, false)
                })
            })
    }

    fn is_literal_text(&self, node: &Inline) -> bool {
        matches!(node.kind, InlineKind::Text)
            && node.span.text(self.document.source).contains(['*', '_'])
    }

    fn delimiters(
        &self,
        index: usize,
        children: &[Inline],
        pair: &Pair<'_>,
        parts: &mut Vec<Delimiter>,
    ) -> bool {
        let span = self.nodes[index].span;
        let opening = Delimiter {
            start: span.start,
            width: pair.preferred.len(),
            marker: pair.marker(),
            opens: true,
        };
        parts.push(opening);
        let scope = EmphasisScope::new(self.document, children);
        for (i, child) in children.iter().enumerate() {
            match &child.kind {
                InlineKind::Emphasis(inner) | InlineKind::Strong(inner) => {
                    let inner = self.document.inlines(*inner);
                    let pair = scope.preferred(i, inner, pair.context);
                    if !scope.delimiters(i, inner, &pair, parts) {
                        return false;
                    }
                }
                InlineKind::Delete(_) => return false,
                _ if self.is_literal_text(child) => return false,
                _ => {}
            }
        }
        parts.push(Delimiter {
            start: span.end - opening.width,
            opens: false,
            ..opening
        });
        true
    }
}

#[derive(Clone, Copy)]
struct Delimiter {
    start: usize,
    width: usize,
    marker: char,
    opens: bool,
}

// Every delimiter in this subtree belongs to a known AST pair. A close must
// consume precisely the innermost opening, even when several pairs share a run.
fn nested_pairs_match(
    source: &str,
    mut parts: &[Delimiter],
    left: Option<char>,
    right: Option<char>,
) -> bool {
    struct Opening {
        marker: char,
        width: usize,
        remaining: usize,
        can_close: bool,
        previous: u8,
    }
    let mut openings: Vec<Opening> = Vec::new();
    let mut open_marks = 0;
    let start = parts[0].start;
    while let [first, rest @ ..] = parts {
        let mut end = first.start + first.width;
        let mut len = 1;
        for next in rest {
            if next.start != end || next.marker != first.marker {
                break;
            }
            end += next.width;
            len += 1;
        }
        let (run, rest) = parts.split_at(len);
        parts = rest;
        let before = if first.start == start {
            left
        } else {
            source[..first.start].chars().next_back()
        };
        let after = if parts.is_empty() {
            right
        } else {
            source[end..].chars().next()
        };
        let (can_open, can_close) = delimiter_flanking(first.marker, before, after);
        let width = end - first.start;
        let mut remaining = width;
        for closing in run.iter().take_while(|p| !p.opens) {
            let Some(open) = openings.last_mut() else {
                return false;
            };
            if !can_close
                || open.marker != first.marker
                || !emphasis_runs_match(open.width, open.can_close, width, can_open)
                || open.remaining.min(remaining).min(2) != closing.width
            {
                return false;
            }
            open.remaining -= closing.width;
            remaining -= closing.width;
            if open.remaining == 0 {
                open_marks = open.previous;
                openings.pop();
            }
        }
        if remaining != 0 {
            // A dual-flanking run could close an earlier opener of this kind.
            if !can_open || can_close && open_marks & bit(first.marker) != 0 {
                return false;
            }
            openings.push(Opening {
                marker: first.marker,
                width,
                remaining,
                can_close,
                previous: open_marks,
            });
            open_marks |= bit(first.marker);
        }
    }
    openings.is_empty()
}

impl Pair<'_> {
    fn marker(&self) -> char {
        char::from(self.preferred.as_bytes()[0])
    }
}

fn is_pair(node: &Inline) -> bool {
    matches!(node.kind, InlineKind::Emphasis(_) | InlineKind::Strong(_))
}

fn is_marker(ch: Option<char>) -> bool {
    matches!(ch, Some('*' | '_'))
}

fn bit(mark: char) -> u8 {
    if mark == '*' { 1 } else { 2 }
}

fn marker_runs<'a>(
    source: &'a str,
    node: &Inline,
) -> impl Iterator<Item = (char, Option<char>, Option<char>)> + 'a {
    let mut rest = node.span.text(source);
    let end = node.span.end;
    std::iter::from_fn(move || {
        let run = &rest[rest.find(['*', '_'])?..];
        let mark = char::from(run.as_bytes()[0]);
        rest = run.trim_start_matches(mark);
        Some((
            mark,
            source[..end - run.len()].chars().next_back(),
            source[end - rest.len()..].chars().next(),
        ))
    })
}

struct LiteralRuns {
    open: u8,
    count: usize,
    pairs: usize,
}

impl LiteralRuns {
    fn isolated(&self) -> bool {
        self.count == 1 && self.pairs == 1
    }

    fn in_scope(source: &str, nodes: &[Inline]) -> Self {
        let mut count = 0;
        let mut pairs = 0;
        let mut open = 0;
        for node in nodes {
            if is_pair(node) {
                pairs += 1;
            }
            if matches!(node.kind, InlineKind::Text) {
                for (mark, left, right) in marker_runs(source, node) {
                    if delimiter_flanking(mark, left, right).0 {
                        open |= bit(mark);
                    }
                    count += 1;
                }
            }
        }
        Self { open, count, pairs }
    }
}
