use std::{
    borrow::Cow,
    cell::{Cell, OnceCell},
    collections::{HashMap, HashSet},
};

use smallvec::SmallVec;
use swc_next_simd::memchr::AsciiByteSet;
use unicode_properties::{GeneralCategoryGroup, UnicodeGeneralCategory};

use crate::inline_store::{Checkpoint, InlineStore};
use crate::{
    Dialect, Inline, InlineExtension, InlineKind, InlineNodes, Link, LinkTarget, ParseOptions,
    RecoveryReason, ReferenceKind, SourceLine, Span,
};

// A page contains at most this many line starts, even in a paragraph of empty lines.
const PAGE_BYTES: usize = 256;

/// Index for lookbehind in long paragraphs. A page points to the line containing
/// its first byte in the normalized text. The forward cursor stays monotonic.
struct EarlierLines {
    pages: Vec<usize>,
    last: Cell<usize>,
}

impl EarlierLines {
    fn new(lines: &[(usize, SourceLine)], text_len: usize) -> Self {
        let mut index = 0;
        let pages = (0..=text_len / PAGE_BYTES)
            .map(|page| {
                while lines
                    .get(index + 1)
                    .is_some_and(|&(start, _)| start <= page * PAGE_BYTES)
                {
                    index += 1;
                }
                index
            })
            .collect();
        Self {
            pages,
            last: Cell::new(0),
        }
    }

    fn line_index(&self, lines: &[(usize, SourceLine)], offset: usize) -> usize {
        let last = self.last.get();
        if lines[last].0 <= offset {
            if lines.get(last + 1).is_none_or(|&(start, _)| start > offset) {
                return last;
            }
            if lines.get(last + 2).is_none_or(|&(start, _)| start > offset) {
                self.last.set(last + 1);
                return last + 1;
            }
        }
        let page = offset / PAGE_BYTES;
        let first = self.pages[page];
        let end = self.pages.get(page + 1).copied().unwrap_or(lines.len() - 1) + 1;
        let index = first + lines[first + 1..end].partition_point(|&(start, _)| start <= offset);
        self.last.set(index);
        index
    }
}

/// One normalized paragraph and the original byte ranges contributing to it.
pub(crate) struct Input<'a> {
    pub(crate) text: Cow<'a, str>,
    base: usize,
    lines: Vec<(usize, SourceLine)>,
    last_line: Cell<usize>,
    earlier: OnceCell<Box<EarlierLines>>,
}
impl<'a> Input<'a> {
    fn direct(text: &'a str, base: usize) -> Self {
        Self {
            text: Cow::Borrowed(text),
            base,
            lines: Vec::new(),
            last_line: Cell::new(0),
            earlier: OnceCell::new(),
        }
    }

    pub(crate) fn for_inlines(source: &'a str, lines: &[SourceLine]) -> Self {
        if let Some(span) = crate::source::contiguous_content(source, lines) {
            return Self::direct(span.text(source), span.start);
        }
        Self::new(source, lines)
    }

    fn new(source: &'a str, lines: &[SourceLine]) -> Self {
        let mut input = Self::direct("", lines.first().map_or(0, |line| line.content.start));
        input.lines.reserve_exact(lines.len());
        for &line in lines {
            input.append_line(source, line);
        }
        input
    }

    pub(crate) fn push_line(&mut self, source: &'a str, first: SourceLine, line: SourceLine) {
        // A one-line definition borrows the source directly. Start its offset map
        // only when a label, destination or title actually continues on a new line.
        if self.lines.is_empty() {
            self.lines.push((0, first));
        }
        self.append_line(source, line);
        // New page boundaries fall in the appended line, including an empty line at EOF.
        if let Some(earlier) = self.earlier.get_mut() {
            earlier
                .pages
                .resize(self.text.len() / PAGE_BYTES + 1, self.lines.len() - 1);
        }
    }

    fn append_line(&mut self, source: &'a str, line: SourceLine) {
        let offset = self.text.len() + usize::from(!self.lines.is_empty());
        if line.virtual_spaces == 0
            && matches!(self.text, Cow::Borrowed(_))
            && self.lines.last().is_none_or(|(_, previous)| {
                previous.content.end + 1 == line.content.start
                    && source.as_bytes().get(previous.content.end) == Some(&b'\n')
            })
        {
            self.text = Cow::Borrowed(&source[line.content.start - offset..line.content.end]);
        } else {
            let text = self.text.to_mut();
            if !self.lines.is_empty() {
                text.push('\n');
            }
            line.append(source, text);
        }
        self.lines.push((offset, line));
    }

    fn line_index(&self, offset: usize) -> usize {
        // Inline nodes are normally visited in source order. Lookbehind for a
        // delimiter must not rewind the cursor through already mapped lines.
        let mut index = self.last_line.get();
        if offset < self.lines[index].0 {
            return self.earlier_line(offset, index);
        }
        while self
            .lines
            .get(index + 1)
            .is_some_and(|&(start, _)| start <= offset)
        {
            index += 1;
        }
        self.last_line.set(index);
        index
    }

    #[inline(never)]
    fn earlier_line(&self, offset: usize, high: usize) -> usize {
        // The known prefix already fits within the search bound of a single page.
        if high <= PAGE_BYTES {
            return self.lines[..high].partition_point(|&(start, _)| start <= offset) - 1;
        }
        self.earlier
            .get_or_init(|| Box::new(EarlierLines::new(&self.lines, self.text.len())))
            .line_index(&self.lines, offset)
    }

    fn byte_on_line(&self, offset: usize, index: usize) -> Span {
        let (start, line) = self.lines[index];
        let within = offset - start;
        let spaces = usize::from(line.virtual_spaces);
        if within < spaces {
            Span::new(line.content.start - 1, line.content.start)
        } else if within - spaces < line.content.len() {
            let physical = line.content.start + within - spaces;
            Span::new(physical, physical + 1)
        } else if offset < self.text.len() {
            line.ending
        } else {
            Span::new(line.content.end, line.content.end)
        }
    }

    #[inline]
    pub(crate) fn span(&self, start: usize, end: usize) -> Span {
        if matches!(self.text, Cow::Borrowed(_)) {
            Span::new(self.base + start, self.base + end)
        } else {
            self.projected_span(start, end)
        }
    }

    #[inline(never)]
    fn projected_span(&self, start: usize, end: usize) -> Span {
        let index = self.line_index(start);
        let (base, line) = self.lines[index];
        let content = base + usize::from(line.virtual_spaces);
        // Within a physical run both endpoints have the same displacement.
        // An empty span at its end can instead start on the original line ending.
        if start < end && start >= content && end <= content + line.content.len() {
            let begin = line.content.start + (start - content);
            return Span::new(begin, begin + (end - start));
        }
        let begin = self.byte_on_line(start, index).start;
        Span::new(
            begin,
            if start == end {
                begin
            } else {
                self.byte_on_line(end - 1, self.line_index(end - 1)).end
            },
        )
    }
}

#[derive(Clone, Copy)]
pub(crate) enum References<'a> {
    Pending(&'a Cell<bool>),
    Indexed(&'a HashMap<String, usize>),
}

impl References<'_> {
    fn may_resolve(self, identifier: &str) -> bool {
        match self {
            Self::Pending(needs_lookup) => {
                needs_lookup.set(true);
                true
            }
            Self::Indexed(definitions) => definitions.contains_key(identifier),
        }
    }
}

pub(super) type PendingNodes = SmallVec<[Inline; 1]>;

pub(crate) struct Options<'a> {
    pub parse: ParseOptions,
    pub store: &'a mut InlineStore,
    pub scratch: &'a mut PendingNodes,
    pub references: References<'a>,
}

pub(crate) fn parse(source: &str, lines: &[SourceLine], options: Options<'_>) -> InlineNodes {
    parse_with_original_lines(source, lines, lines, options)
}

pub(crate) fn parse_with_original_lines(
    source: &str,
    lines: &[SourceLine],
    original_lines: &[SourceLine],
    options: Options<'_>,
) -> InlineNodes {
    if let [line] = lines
        && line.virtual_spaces == 0
        && options.parse.max_nesting > 0
        && options
            .parse
            .max_inline_steps
            .is_none_or(|limit| limit >= line.content.len())
    {
        let text = line.content.text(source);
        if text.is_empty() {
            return InlineNodes::default();
        }
        // All literal autolinks need a dot in their domain. A line with no
        // inline markers or dot has just one source-backed text node.
        if INLINE_MARKERS_AND_DOT.find(text.as_bytes()).is_none() {
            return options.store.append(&[Inline {
                span: line.content,
                kind: InlineKind::Text,
            }]);
        }
    }
    if lines.len() > 1
        && options.parse.max_nesting > 0
        && options.parse.max_inline_steps.is_none()
        && lines.iter().all(|line| {
            line.virtual_spaces == 0
                && INLINE_MARKERS_AND_DOT
                    .find(line.content.bytes(source))
                    .is_none()
        })
    {
        // These physical spans already contain every token. A synthetic paragraph
        // and its index would only be used to translate them back to source offsets.
        let nodes = options.scratch;
        nodes.clear();
        nodes.reserve(lines.len() * 2 - 1);
        for (index, line) in lines.iter().enumerate() {
            let text = line.content.text(source);
            let last = index + 1 == lines.len();
            let hard = !last && text.ends_with("  ");
            let end = if hard {
                line.content.start + text.trim_end_matches(' ').len()
            } else {
                line.content.end
            };
            if end > line.content.start {
                nodes.push(Inline {
                    span: Span::new(line.content.start, end),
                    kind: InlineKind::Text,
                });
            }
            if !last {
                let (span, kind) = if hard {
                    (Span::new(end, line.ending.end), InlineKind::HardBreak)
                } else {
                    (line.ending, InlineKind::SoftBreak)
                };
                nodes.push(Inline { span, kind });
            }
        }
        return options.store.append(nodes);
    }
    let input = Input::for_inlines(source, lines);
    if input.text.is_empty() {
        return InlineNodes::default();
    }
    let bytes = input.text.as_bytes();
    let has_email = memchr::memchr(b'@', bytes).is_some();
    let starts = if has_email {
        &INLINE_STARTS_WITH_EMAIL
    } else if memchr::memchr2(b':', b'.', bytes).is_some() {
        &INLINE_STARTS
    } else {
        &INLINE_MARKERS
    };
    let mut parser = Parser {
        input: &input,
        store: options.store,
        source,
        original_lines,
        ticks: TickClosers::default(),
        has_email,
        starts,
        ascii: input.text.is_ascii(),
        label_angles: HashMap::new(),
        link_openers: SmallVec::new(),
        inactive_links: HashSet::new(),
        options: options.parse,
        references: options.references,
        work: bytes
            .len()
            .saturating_mul(24)
            .saturating_add(128)
            .min(options.parse.max_inline_steps.unwrap_or(usize::MAX)),
    };
    let nodes = options.scratch;
    parser.nodes(0, bytes.len(), 0, None, nodes);
    parser.store.append(merged_text(nodes))
}

/// Consecutive equal-width runs pair directly. Index the whole inline input once
/// if different widths intervene or a link lookahead revisits an earlier run.
#[derive(Default)]
struct TickClosers {
    scanned_to: usize,
    pairs: Option<HashMap<usize, usize>>,
}

impl TickClosers {
    fn closing(&mut self, bytes: &[u8], start: usize, width: usize, end: usize) -> Option<usize> {
        if start != 0 && bytes[start - 1] == b'\x60' {
            return None;
        }
        if self.pairs.is_none() && start >= self.scanned_to {
            let after = start + width;
            let next = memchr::memchr(b'\x60', &bytes[after..end]).map(|offset| after + offset);
            self.scanned_to = next.unwrap_or(end);
            let next = next?;
            if run(bytes, next, bytes.len()) == width {
                return (next + width <= end).then_some(next);
            }
        }
        self.pairs
            .get_or_insert_with(|| Self::index(bytes))
            .get(&start)
            .copied()
            .filter(|&next| next + width <= end)
    }

    fn index(bytes: &[u8]) -> HashMap<usize, usize> {
        let mut pairs = HashMap::new();
        let mut last = HashMap::new();
        let mut pos = bytes.len();
        while let Some(tick) = memchr::memrchr(b'\x60', &bytes[..pos]) {
            let end = tick + 1;
            pos = tick;
            while pos > 0 && bytes[pos - 1] == b'\x60' {
                pos -= 1;
            }
            if let Some(next) = last.insert(end - pos, pos) {
                pairs.insert(pos, next);
            }
        }
        pairs
    }
}

struct Parser<'a> {
    store: &'a mut InlineStore,
    input: &'a Input<'a>,
    source: &'a str,
    original_lines: &'a [SourceLine],
    ticks: TickClosers,
    has_email: bool,
    starts: &'static AsciiByteSet,
    ascii: bool,
    /// Complete tokens found during label lookahead are reused when parsing its children.
    label_angles: HashMap<usize, (usize, AngleKind)>,
    /// Open non-image link labels while parsing their contents.
    link_openers: SmallVec<[usize; 2]>,
    /// Openers deactivated by an inner link stay literal when scanning resumes.
    inactive_links: HashSet<usize>,
    work: usize,
    options: ParseOptions,
    references: References<'a>,
}
const INLINE_MARKERS: AsciiByteSet = AsciiByteSet::new(b"\\\n\x60*_~&[$!<{");
const INLINE_MARKERS_AND_DOT: AsciiByteSet = INLINE_MARKERS.with_range(b'.', b'.');

// Bare URL schemes start with H, W or F. Email local parts may start with a letter or digit.
const INLINE_STARTS: AsciiByteSet = INLINE_MARKERS.with_bytes(b"HhWwFf");
const INLINE_STARTS_WITH_EMAIL: AsciiByteSet = INLINE_MARKERS
    .with_bytes(b"+")
    .with_range(b'a', b'z')
    .with_range(b'A', b'Z')
    .with_range(b'0', b'9');

impl<'a> Parser<'a> {
    fn nodes(
        &mut self,
        start: usize,
        end: usize,
        depth: usize,
        link_opener: Option<usize>,
        nodes: &mut PendingNodes,
    ) {
        nodes.clear();
        if start == end {
            return;
        }
        if depth >= self.options.max_nesting || self.work == 0 {
            nodes.push(Inline {
                span: self.input.span(start, end),
                kind: InlineKind::Recovery(if self.work == 0 {
                    RecoveryReason::InlineWork
                } else {
                    RecoveryReason::Nesting
                }),
            });
            return;
        }
        let bytes = self.input.text.as_bytes();
        let starts = self.starts;
        let mut openings = SmallVec::new();
        let mut text_start = start;
        let mut p = start;
        while p < end {
            if link_opener.is_some() && self.link_openers.last().copied() != link_opener {
                nodes.clear();
                return;
            }
            if self.work == 0 {
                if text_start < p {
                    nodes.push(Inline {
                        span: self.input.span(text_start, p),
                        kind: InlineKind::Text,
                    });
                }
                nodes.push(Inline {
                    span: self.input.span(p, end),
                    kind: InlineKind::Recovery(RecoveryReason::InlineWork),
                });
                return;
            }
            if !starts.contains(bytes[p]) {
                // A literal character consumes one step, including a multibyte scalar.
                let limit = end.min(p.saturating_add(self.work));
                let limit = if self.ascii {
                    limit
                } else {
                    self.input.text.ceil_char_boundary(limit)
                };
                let plain = &bytes[p..limit];
                let len = starts.find(plain).unwrap_or(plain.len());
                self.work -= if self.ascii {
                    len
                } else {
                    self.input.text[p..p + len].chars().count()
                };
                p += len;
                continue;
            }
            self.work -= 1;
            let value = match bytes[p] {
                b'\\' if bytes.get(p + 1) == Some(&b'\n') && p + 1 < end => {
                    Some((p, p + 2, InlineKind::HardBreak))
                }
                b'\\' if p + 1 < end && bytes[p + 1].is_ascii_punctuation() => {
                    Some((p, p + 2, InlineKind::Escape))
                }
                b'\n' => {
                    let mut begin = p;
                    while begin > text_start && bytes[begin - 1] == b' ' {
                        begin -= 1;
                    }
                    if p - begin >= 2 {
                        Some((begin, p + 1, InlineKind::HardBreak))
                    } else {
                        Some((p, p + 1, InlineKind::SoftBreak))
                    }
                }
                b'\x60' => {
                    let size = run(bytes, p, end);
                    if let Some(close) = self.ticks.closing(bytes, p, size, end) {
                        let raw = self.code_span_content(p + size, close);
                        let value = if raw.starts_with(' ')
                            && raw.ends_with(' ')
                            && raw.bytes().any(|b| b != b' ')
                        {
                            &raw[1..raw.len() - 1]
                        } else {
                            &raw
                        };
                        let value = self.store.push_text(value);
                        Some((p, close + size, InlineKind::CodeSpan { width: size, value }))
                    } else {
                        p += size;
                        continue;
                    }
                }
                b'*' | b'_' | b'~' => {
                    let size = run(bytes, p, end);
                    if self.can_delimit(bytes[p], size) {
                        if text_start < p {
                            nodes.push(Inline {
                                span: self.input.span(text_start, p),
                                kind: InlineKind::Text,
                            });
                        }
                        self.delimiter(p, size, depth, &mut openings, nodes);
                        text_start = p + size;
                    }
                    p += size;
                    continue;
                }
                b'&' if self.options.dialect == Dialect::Extended => {
                    crate::reference::length(&self.input.text[p..end])
                        .map(|n| (p, p + n, InlineKind::Extension(InlineExtension::Entity)))
                }
                b'[' if self.options.dialect == Dialect::Extended
                    && bytes.get(p + 1) == Some(&b'^') =>
                {
                    self.find(p + 2, end, "]")
                        .filter(|&c| c > p + 2 && !self.input.text[p + 2..c].contains(['[', '\n']))
                        .map(|c| {
                            (
                                p,
                                c + 1,
                                InlineKind::Extension(InlineExtension::Footnote {
                                    label: self.input.span(p + 2, c),
                                }),
                            )
                        })
                }
                b'$' if self.options.dialect == Dialect::Extended => {
                    let width = run(bytes, p, end);
                    let matched = self.math(p, end, width);
                    if matched.is_none() {
                        p += width;
                        continue;
                    }
                    matched
                }
                b'!' if p + 1 < end && bytes[p + 1] == b'[' => {
                    self.link(p, end, depth, true, &openings)
                }
                b'[' if bytes.get(p + 1) == Some(&b'[') => self.wiki_link(p, end),
                b'[' => self.link(p, end, depth, false, &openings),
                b'<' => self.angle(p, end).map(|(next, kind)| {
                    let kind = match kind {
                        AngleKind::Html => InlineKind::Html,
                        AngleKind::Autolink => InlineKind::Autolink,
                    };
                    (p, next, kind)
                }),
                b'{' if matches!(bytes.get(p + 1), Some(b'{' | b'%')) => {
                    let close = if bytes[p + 1] == b'{' { "}}" } else { "%}" };
                    self.delimited(
                        p,
                        end,
                        close,
                        InlineKind::Extension(InlineExtension::Liquid),
                    )
                }
                b'$' if bytes.get(p + 1) == Some(&b'$') => {
                    self.delimited(p, end, "$$", InlineKind::Raw)
                }
                b if (b.is_ascii_alphanumeric()
                    || self.options.dialect == Dialect::Extended && b == b'+')
                    && (p == start || !email_character(bytes[p - 1])) =>
                {
                    bare_autolink(
                        &self.input.text[p..end],
                        self.options.dialect,
                        self.has_email,
                        &mut self.work,
                    )
                    .map(|size| (p, p + size, InlineKind::Autolink))
                }
                _ => None,
            };
            if let Some((begin, next, kind)) = value {
                if text_start < begin {
                    nodes.push(Inline {
                        span: self.input.span(text_start, begin),
                        kind: InlineKind::Text,
                    });
                }
                nodes.push(Inline {
                    span: self.input.span(begin, next),
                    kind,
                });
                text_start = next;
                p = next;
            } else {
                p += self.input.text[p..end].chars().next().unwrap().len_utf8();
            }
        }
        if text_start < end {
            nodes.push(Inline {
                span: self.input.span(text_start, end),
                kind: InlineKind::Text,
            });
        }
    }

    fn code_span_content(&self, start: usize, end: usize) -> Cow<'a, str> {
        let text = &self.input.text[start..end];
        if !text.as_bytes().contains(&b'\n') {
            return Cow::Borrowed(text);
        }
        if matches!(self.input.text, Cow::Borrowed(_)) {
            return Cow::Owned(text.replace('\n', " "));
        }
        let mut fragments = text.split('\n');
        let mut result = fragments.next().unwrap_or_default().to_owned();

        // Paragraphs remove line prefixes for prose. Code spans retain that
        // indentation after each normalized line ending.
        let next_line = self.input.line_index(start) + 1;
        for (index, fragment) in (next_line..).zip(fragments) {
            let original = self.original_lines[index];
            let (_, trimmed) = self.input.lines[index];
            result.push(' ');
            let virtual_spaces = original.virtual_spaces - trimmed.virtual_spaces;
            result.extend(std::iter::repeat_n(' ', usize::from(virtual_spaces)));
            result.push_str(&self.source[original.content.start..trimmed.content.start]);
            result.push_str(fragment);
        }
        Cow::Owned(result)
    }

    fn find(&mut self, start: usize, end: usize, needle: &str) -> Option<usize> {
        let hay = &self.input.text.as_bytes()[start..end];
        let limit = self.work.min(hay.len());
        let hit = hay[..limit]
            .windows(needle.len())
            .position(|w| w == needle.as_bytes());
        self.work = self.work.saturating_sub(hit.map_or(limit, |n| n + 1));
        hit.map(|n| start + n)
    }
    fn wiki_link(&mut self, start: usize, end: usize) -> Option<(usize, usize, InlineKind)> {
        let body = start + 2;
        if body >= end {
            return None;
        }
        let limit = end.min(body.saturating_add(self.work));
        let bytes = self.input.text.as_bytes();
        let stop = bytes[body..limit]
            .iter()
            .position(|b| matches!(b, b']' | b'\n'));
        self.work = self
            .work
            .saturating_sub(stop.map_or(limit - body, |n| n + 1));
        let close = body + stop?;
        (close + 2 <= end
            && &bytes[close..close + 2] == b"]]"
            && bytes[body..close]
                .iter()
                .any(|b| !matches!(b, b' ' | b'\t')))
        .then_some((
            start,
            close + 2,
            InlineKind::Extension(InlineExtension::WikiLink),
        ))
    }

    fn delimited(
        &mut self,
        p: usize,
        end: usize,
        close: &str,
        kind: InlineKind,
    ) -> Option<(usize, usize, InlineKind)> {
        if p + 2 > end {
            return None;
        }
        let next = self.find(p + 2, end, close)? + close.len();
        Some((p, next, kind))
    }
    fn math(&mut self, p: usize, end: usize, width: usize) -> Option<(usize, usize, InlineKind)> {
        let bytes = self.input.text.as_bytes();
        let body = p + width;
        if width > 2 || body >= end || width == 1 && bytes[body].is_ascii_whitespace() {
            return None;
        }
        let mut i = body;
        while i < end && self.work > 0 {
            self.work -= 1;
            match bytes[i] {
                b'\\' => i = (i + 2).min(end),
                b'$' => {
                    let count = run(bytes, i, end);
                    let after = i + count;
                    if count == width
                        && i > body
                        && (width == 2
                            || !bytes[i - 1].is_ascii_whitespace()
                                && !bytes.get(after).is_some_and(u8::is_ascii_digit))
                    {
                        return Some((
                            p,
                            after,
                            InlineKind::Extension(InlineExtension::Math {
                                display: width == 2,
                            }),
                        ));
                    }
                    i = after;
                }
                _ => i += 1,
            }
        }
        None
    }
    fn angle(&mut self, p: usize, end: usize) -> Option<(usize, AngleKind)> {
        if let Some(&(next, kind)) = self.label_angles.get(&p) {
            return (next <= end).then_some((next, kind));
        }
        let src = &self.input.text[p..end];
        let special = if let Some(body) = src.strip_prefix("<!--") {
            if body.starts_with('>') {
                return Some((p + 5, AngleKind::Html));
            }
            if body.starts_with("->") {
                return Some((p + 6, AngleKind::Html));
            }
            Some(("-->", 4))
        } else if src.starts_with("<?") {
            Some(("?>", 2))
        } else if src.starts_with("<![CDATA[") {
            Some(("]]>", 9))
        } else if src.starts_with("<!") {
            Some((">", 2))
        } else {
            None
        };
        if let Some((close, opening)) = special {
            let next = self.find(p + opening, end, close)?;
            return Some((next + close.len(), AngleKind::Html));
        }

        let limit = src.len().min(self.work.saturating_add(1));
        match html_tag(&src.as_bytes()[..limit]) {
            Ok(length) => {
                self.work = self.work.saturating_sub(length - 1);
                return Some((p + length, AngleKind::Html));
            }
            Err(inspected) => self.work = self.work.saturating_sub(inspected.saturating_sub(1)),
        }

        let bytes = &src.as_bytes()[1..src.len().min(self.work.saturating_add(1))];
        let closing = bytes
            .iter()
            .position(|byte| matches!(byte, b'<' | b'>' | b' ' | b'\t' | b'\r' | b'\n'));
        self.work = self
            .work
            .saturating_sub(closing.map_or(bytes.len(), |index| index + 1));
        let closing = closing.filter(|&index| bytes[index] == b'>')?;
        let next = p + 1 + closing;
        let value = &self.input.text[p + 1..next];
        let scheme = value.split_once(':').is_some_and(|(key, _)| {
            (2..=32).contains(&key.len())
                && key.as_bytes()[0].is_ascii_alphabetic()
                && key
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"+.-".contains(&b))
        });
        let extended = self.options.dialect == Dialect::Extended;
        let email = value.split_once('@').is_some_and(|(local, domain)| {
            !local.is_empty()
                && if extended {
                    local
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"!#$%&'*+-/=?^_`{|}~.".contains(&b))
                        && domain.split('.').all(|label| {
                            (1..=63).contains(&label.len())
                                && !label.starts_with('-')
                                && !label.ends_with('-')
                                && label
                                    .bytes()
                                    .all(|b| b.is_ascii_alphanumeric() || b == b'-')
                        })
                } else {
                    domain.contains('.')
                }
        });
        let auto = (scheme || email) && !value.chars().any(|c| c.is_whitespace() || c == '<');
        auto.then_some((next + 1, AngleKind::Autolink))
    }
    fn delimiter(
        &mut self,
        start: usize,
        width: usize,
        depth: usize,
        openings: &mut SmallVec<[Opening; 2]>,
        nodes: &mut PendingNodes,
    ) {
        let mark = self.input.text.as_bytes()[start];
        let (can_open, can_close) = self.delimiter_flanks(start, width, mark);
        let mut right = start;
        while can_close && right < start + width && self.work != 0 {
            let Some(index) = self.opener(openings, mark, width, can_open) else {
                break;
            };
            let opening = &mut openings[index];
            let count = if mark == b'~' {
                width
            } else {
                opening.remaining.min(start + width - right).min(2)
            };
            opening.remaining -= count;
            right += count;
            let span = self.input.span(opening.start + opening.remaining, right);
            let first_child = opening.node + 1;
            let too_deep = nodes[first_child..]
                .iter()
                .any(|node| inline_depth(self.store, node) + depth >= self.options.max_nesting);
            let kind = if too_deep {
                self.store.rewind(opening.checkpoint);
                InlineKind::Recovery(RecoveryReason::Nesting)
            } else {
                let children = self.store.append(merged_text(&mut nodes[first_child..]));
                if mark == b'~' {
                    InlineKind::Delete(children)
                } else if count == 2 {
                    InlineKind::Strong(children)
                } else {
                    InlineKind::Emphasis(children)
                }
            };
            nodes.truncate(first_child);
            if opening.remaining == 0 {
                nodes.pop();
                openings.truncate(index);
            } else {
                nodes[opening.node].span = self
                    .input
                    .span(opening.start, opening.start + opening.remaining);
                openings.truncate(index + 1);
            }
            nodes.push(Inline { span, kind });
        }
        if right != start + width {
            if can_open {
                openings.push(Opening {
                    mark,
                    start: right,
                    width,
                    remaining: start + width - right,
                    can_close,
                    node: nodes.len(),
                    checkpoint: self.store.checkpoint(),
                });
            }
            nodes.push(Inline {
                span: self.input.span(right, start + width),
                kind: InlineKind::Text,
            });
        }
    }

    fn can_delimit(&self, mark: u8, width: usize) -> bool {
        mark != b'~' || width == 2 || self.options.dialect == Dialect::Extended && width == 1
    }

    fn delimiter_flanks(&self, start: usize, width: usize, mark: u8) -> (bool, bool) {
        let flank_mark = if mark == b'~' && width == 1 {
            b'_'
        } else {
            mark
        };
        let text = &self.input.text;
        if self.ascii {
            let bytes = text.as_bytes();
            let before = start.checked_sub(1).map_or(b' ', |pos| bytes[pos]);
            let after = bytes.get(start + width).copied().unwrap_or(b' ');
            let classify = |b: u8| (matches!(b, b'\t'..=b'\r' | b' '), b.is_ascii_punctuation());
            return flanking(char::from(flank_mark), classify(before), classify(after));
        }
        delimiter_flanking(
            char::from(flank_mark),
            text[..start].chars().next_back(),
            text[start + width..].chars().next(),
        )
    }

    fn opener(
        &mut self,
        openings: &[Opening],
        mark: u8,
        width: usize,
        can_open: bool,
    ) -> Option<usize> {
        for (index, opening) in openings.iter().enumerate().rev() {
            self.work = self.work.checked_sub(1)?;
            if opening.mark != mark {
                continue;
            }
            let matches = if mark == b'~' {
                opening.width == width
            } else {
                emphasis_runs_match(opening.width, opening.can_close, width, can_open)
            };
            if matches {
                return Some(index);
            }
        }
        None
    }

    fn link(
        &mut self,
        p: usize,
        end: usize,
        depth: usize,
        image: bool,
        openings: &[Opening],
    ) -> Option<(usize, usize, InlineKind)> {
        if !image && self.inactive_links.contains(&p) {
            return None;
        }
        let bytes = self.input.text.as_bytes();
        let open = p + usize::from(image);
        let mut pos = open + 1;
        let mut nested = 0usize;
        let mut closing_delimiters = SmallVec::<[Span; 2]>::new();
        const LABEL_MARKERS: AsciiByteSet = AsciiByteSet::new(b"\\\x60<[]");
        const LABEL_AND_EMPHASIS: AsciiByteSet = LABEL_MARKERS.with_bytes(b"*_~");
        let markers = if openings.is_empty() {
            &LABEL_MARKERS
        } else {
            &LABEL_AND_EMPHASIS
        };
        while pos < end {
            if self.work == 0 {
                return None;
            }
            if !markers.contains(bytes[pos]) {
                let limit = end.min(pos.saturating_add(self.work));
                let plain = &bytes[pos..limit];
                let n = markers.find(plain).unwrap_or(plain.len());
                pos += n;
                self.work -= n;
                continue;
            }
            self.work -= 1;
            match bytes[pos] {
                b'\\' => {
                    pos = (pos + 2).min(end);
                    continue;
                }
                b'\x60' => {
                    let n = run(bytes, pos, end);
                    pos = self
                        .ticks
                        .closing(bytes, pos, n, end)
                        .map_or(pos + n, |c| c + n);
                    continue;
                }
                mark @ (b'*' | b'_' | b'~') if !openings.is_empty() => {
                    let limit = pos.saturating_add(self.work).saturating_add(1).min(end);
                    let width = run(bytes, pos, limit);
                    self.work -= width - 1;
                    if self.can_delimit(mark, width) {
                        let (can_open, can_close) = self.delimiter_flanks(pos, width, mark);
                        if can_close && self.opener(openings, mark, width, can_open).is_some() {
                            closing_delimiters.push(self.input.span(pos, pos + width));
                        }
                    }
                    pos += width;
                    continue;
                }
                b'<' => {
                    if let Some((next, kind)) = self.angle(pos, end) {
                        self.label_angles.insert(pos, (next, kind));
                        pos = next;
                        continue;
                    }
                }
                b'[' => nested += 1,
                b']' if nested == 0 => break,
                b']' => nested -= 1,
                _ => {}
            }
            pos += 1;
        }
        if pos == end {
            return None;
        }
        let label = self.input.span(open + 1, pos);
        let after = pos + 1;
        let mut next = after;
        let checkpoint = self.store.checkpoint();
        let target = if bytes.get(after) == Some(&b'(') && after < end {
            let begin = after + 1;
            let limit = end.min(begin.saturating_add(self.work));
            let parsed = destination_and_title(&self.input.text, begin, limit, true);
            let inspected = match &parsed {
                Ok((finish, _, _)) => *finish,
                Err(at) => *at,
            };
            self.work = self.work.saturating_sub(inspected - begin);
            match parsed {
                Ok((finish, dest, title)) => {
                    next = finish;
                    LinkTarget::Inline {
                        destination: self.input.span(dest.start, dest.end),
                        title: title.map(|s| self.input.span(s.start, s.end)),
                    }
                }
                Err(_) if self.options.dialect == Dialect::Extended => LinkTarget::Reference {
                    label: None,
                    identifier: self.store.push_identifier(&self.input.text[open + 1..pos]),
                    kind: ReferenceKind::Shortcut,
                },
                Err(_) => return None,
            }
        } else {
            let (raw_label, reference, kind) = if bytes.get(after) == Some(&b'[') && after < end {
                let mut close = after + 1;
                while close < end && self.work > 0 {
                    self.work -= 1;
                    match bytes[close] {
                        b'\\' => {
                            close = (close + 2).min(end);
                            continue;
                        }
                        b'[' => return None,
                        b']' => break,
                        _ => {
                            let start = close + 1;
                            let limit = end.min(start.saturating_add(self.work));
                            let rest = &bytes[start..limit];
                            let plain =
                                memchr::memchr3(b'\\', b'[', b']', rest).unwrap_or(rest.len());
                            close = start + plain;
                            self.work -= plain;
                        }
                    }
                }
                if close >= end || bytes[close] != b']' {
                    return None;
                }
                next = close + 1;
                if close == after + 1 {
                    (
                        &self.input.text[open + 1..pos],
                        Some(self.input.span(close, close)),
                        ReferenceKind::Collapsed,
                    )
                } else {
                    (
                        &self.input.text[after + 1..close],
                        Some(self.input.span(after + 1, close)),
                        ReferenceKind::Full,
                    )
                }
            } else {
                (
                    &self.input.text[open + 1..pos],
                    None,
                    ReferenceKind::Shortcut,
                )
            };
            if !link_label_has_text(raw_label) {
                return None;
            }
            LinkTarget::Reference {
                label: reference,
                identifier: self.store.push_identifier(raw_label),
                kind,
            }
        };
        let opener = (!image).then_some(open);
        if let Some(open) = opener {
            self.link_openers.push(open);
        }
        let mut children = PendingNodes::new();
        let raw = &self.input.text[open + 1..pos];
        if !raw.is_empty()
            && depth + 1 < self.options.max_nesting
            && self.work >= raw.len()
            && self.starts.find(raw.as_bytes()).is_none()
        {
            // The label scan already mapped the source span. Plain text charges
            // the same scalar budget as the ordinary inline traversal.
            self.work -= if self.ascii {
                raw.len()
            } else {
                raw.chars().count()
            };
            children.push(Inline {
                span: label,
                kind: InlineKind::Text,
            });
        } else {
            self.nodes(open + 1, pos, depth + 1, opener, &mut children);
        }
        if let Some(open) = opener {
            if self.link_openers.last() != Some(&open) {
                self.store.rewind(checkpoint);
                return None;
            }
            self.link_openers.pop();
        }
        if let LinkTarget::Reference { identifier, .. } = &target
            && has_text_delimiter(&children, &closing_delimiters)
            && !self.references.may_resolve(self.store.text(*identifier))
        {
            self.store.rewind(checkpoint);
            return None;
        }
        if !image && !self.link_openers.is_empty() {
            let resolved = match &target {
                LinkTarget::Inline { .. } => true,
                LinkTarget::Reference { identifier, .. } => {
                    self.references.may_resolve(self.store.text(*identifier))
                }
            };
            if resolved {
                self.inactive_links.extend(self.link_openers.drain(..));
            }
        }
        let children = self.store.append(merged_text(&mut children));
        let link = self.store.push_link(Link {
            brackets: [
                self.input.span(open, open + 1),
                self.input.span(pos, pos + 1),
            ],
            target_delimiters: (next != after).then(|| {
                [
                    self.input.span(after, after + 1),
                    self.input.span(next - 1, next),
                ]
            }),
            label,
            children,
            target,
        });
        Some((
            p,
            next,
            if image {
                InlineKind::Image(link)
            } else {
                InlineKind::Link(link)
            },
        ))
    }
}

#[derive(Clone, Copy)]
enum AngleKind {
    Html,
    Autolink,
}

/// An unpaired closer in a tentative reference label can belong to an outer run.
fn has_text_delimiter(children: &[Inline], candidates: &[Span]) -> bool {
    if candidates.is_empty() {
        return false;
    }
    let mut candidates = candidates.iter().peekable();
    children.iter().any(|child| {
        while candidates
            .peek()
            .is_some_and(|span| span.end <= child.span.start)
        {
            candidates.next();
        }
        matches!(child.kind, InlineKind::Text)
            && candidates
                .peek()
                .is_some_and(|span| span.start < child.span.end)
    })
}

struct Opening {
    mark: u8,
    start: usize,
    width: usize,
    remaining: usize,
    can_close: bool,
    node: usize,
    checkpoint: Checkpoint,
}

fn inline_depth(store: &InlineStore, node: &Inline) -> usize {
    let group = match node.kind {
        InlineKind::Emphasis(nodes) | InlineKind::Strong(nodes) | InlineKind::Delete(nodes) => {
            nodes
        }
        InlineKind::Link(id) | InlineKind::Image(id) => store.link(id).children,
        _ => return 0,
    };
    1 + store
        .nodes(group)
        .iter()
        .map(|child| inline_depth(store, child))
        .max()
        .unwrap_or_default()
}

/// Coalesce adjacent physical text runs before publishing a sibling group.
fn merged_text(nodes: &mut [Inline]) -> &[Inline] {
    let joins = |left: &Inline, right: &Inline| {
        matches!(left.kind, InlineKind::Text)
            && matches!(right.kind, InlineKind::Text)
            && left.span.end == right.span.start
    };
    let Some(mut last) = nodes.windows(2).position(|pair| joins(&pair[0], &pair[1])) else {
        return nodes;
    };
    for index in last + 1..nodes.len() {
        if joins(&nodes[last], &nodes[index]) {
            nodes[last].span.end = nodes[index].span.end;
        } else {
            last += 1;
            nodes[last] = nodes[index];
        }
    }
    &nodes[..=last]
}

pub(crate) fn link_label_has_text(label: &str) -> bool {
    label
        .bytes()
        .any(|byte| !matches!(byte, b' ' | b'\t' | b'\n' | b'\r'))
}

/// Scan one ordinary HTML tag and report its byte length or the inspected prefix.
pub(crate) fn html_tag(bytes: &[u8]) -> Result<usize, usize> {
    let mut pos = 1;
    let closing = bytes.get(pos) == Some(&b'/');
    pos += usize::from(closing);
    if !bytes.get(pos).is_some_and(u8::is_ascii_alphabetic) {
        return Err(pos);
    }
    pos += 1;
    while bytes
        .get(pos)
        .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'-')
    {
        pos += 1;
    }
    if closing {
        pos = skip_html_space(bytes, pos);
        return if bytes.get(pos) == Some(&b'>') {
            Ok(pos + 1)
        } else {
            Err(pos)
        };
    }

    loop {
        let before_space = pos;
        pos = skip_html_space(bytes, pos);
        match bytes.get(pos) {
            Some(b'>') => return Ok(pos + 1),
            Some(b'/') if bytes.get(pos + 1) == Some(&b'>') => return Ok(pos + 2),
            Some(byte)
                if pos > before_space
                    && (byte.is_ascii_alphabetic() || matches!(byte, b'_' | b':')) =>
            {
                pos += 1;
            }
            _ => return Err(pos),
        }
        while bytes.get(pos).is_some_and(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b':' | b'.' | b'-')
        }) {
            pos += 1;
        }
        let equals = skip_html_space(bytes, pos);
        if bytes.get(equals) != Some(&b'=') {
            continue;
        }
        pos = skip_html_space(bytes, equals + 1);
        if let Some(&quote @ (b'\'' | b'"')) = bytes.get(pos) {
            pos += 1;
            while bytes.get(pos).is_some_and(|&byte| byte != quote) {
                pos += 1;
            }
            if bytes.get(pos) != Some(&quote) {
                return Err(pos);
            }
            pos += 1;
        } else {
            let start = pos;
            while bytes.get(pos).is_some_and(|byte| {
                !matches!(
                    byte,
                    b' ' | b'\t' | b'\r' | b'\n' | b'\'' | b'"' | b'=' | b'<' | b'>' | b'\x60'
                )
            }) {
                pos += 1;
            }
            if pos == start {
                return Err(pos);
            }
        }
    }
}

fn skip_html_space(bytes: &[u8], mut pos: usize) -> usize {
    while matches!(bytes.get(pos), Some(b' ' | b'\t' | b'\r' | b'\n')) {
        pos += 1;
    }
    pos
}

/// Whether a delimiter run between these characters can open and can close, in that order.
/// An underscore also observes CommonMark's restriction on intraword emphasis.
pub fn delimiter_flanking(mark: char, prev: Option<char>, next: Option<char>) -> (bool, bool) {
    let pw = prev.is_none_or(char::is_whitespace);
    let nw = next.is_none_or(char::is_whitespace);
    let punct = |c: char| {
        if c.is_ascii() {
            c.is_ascii_punctuation()
        } else {
            matches!(
                c.general_category_group(),
                GeneralCategoryGroup::Punctuation | GeneralCategoryGroup::Symbol
            )
        }
    };
    let pp = prev.is_some_and(punct);
    let np = next.is_some_and(punct);
    flanking(mark, (pw, pp), (nw, np))
}

fn flanking(mark: char, (pw, pp): (bool, bool), (nw, np): (bool, bool)) -> (bool, bool) {
    let left = !nw && (!np || pw || pp);
    let right = !pw && (!pp || nw || np);
    if mark == '_' {
        (left && (!right || pp), right && (!left || np))
    } else {
        (left, right)
    }
}

/// Apply CommonMark's multiple-of-three rule to two flanking emphasis runs of the same marker.
/// Widths are those of the original runs, including any markers already consumed.
pub fn emphasis_runs_match(
    open: usize,
    open_can_close: bool,
    close: usize,
    close_can_open: bool,
) -> bool {
    !(open_can_close || close_can_open) || open % 3 + close % 3 != 3
}

pub(crate) fn run(bytes: &[u8], start: usize, end: usize) -> usize {
    bytes[start..end]
        .iter()
        .take_while(|&&b| b == bytes[start])
        .count()
}

pub(crate) fn destination_and_title(
    text: &str,
    start: usize,
    end: usize,
    link: bool,
) -> Result<(usize, Span, Option<Span>), usize> {
    let bytes = text.as_bytes();
    let spaces = |mut p: usize| {
        while p < end && bytes[p].is_ascii_whitespace() {
            p += 1;
        }
        p
    };
    let begin = spaces(start);
    let mut p = begin;
    if p < end && bytes[p] == b'<' {
        p += 1;
        while p < end {
            match bytes[p] {
                b'\\' if p + 1 < end => p += 2,
                b'<' | b'\n' => return Err(p),
                b'>' => break,
                _ => p += 1,
            }
        }
        if p == end {
            return Err(p);
        }
        p += 1;
    } else {
        const DELIMITERS: AsciiByteSet = AsciiByteSet::new(b"\\() \t\n\x0c\r");
        let mut level = 0;
        while p < end {
            let Some(offset) = DELIMITERS.find(&bytes[p..end]) else {
                p = end;
                break;
            };
            p += offset;
            match bytes[p] {
                b'\\' if p + 1 < end => {
                    p += 2;
                    continue;
                }
                b'(' => level += 1,
                b')' if level == 0 => break,
                b')' => level -= 1,
                b if b.is_ascii_whitespace() => break,
                _ => {}
            }
            p += 1;
        }
        if level > 0 || !link && p == begin {
            return Err(p);
        }
    }
    let dest = Span::new(begin, p);
    let before = p;
    p = spaces(p);
    let title = if p < end && p != before && matches!(bytes[p], b'\'' | b'"' | b'(') {
        let title_start = p;
        let close = if bytes[p] == b'(' { b')' } else { bytes[p] };
        p += 1;
        while p < end && bytes[p] != close {
            if bytes[p] == b'\\' && p + 1 < end {
                p += 1;
            }
            p += 1;
        }
        if p == end {
            return Err(p);
        }
        p += 1;
        let title = Span::new(title_start, p);
        p = spaces(p);
        Some(title)
    } else {
        None
    };
    if link {
        (p < end && bytes[p] == b')')
            .then_some((p + 1, dest, title))
            .ok_or(p)
    } else {
        (p == end).then_some((p, dest, title)).ok_or(p)
    }
}

fn email_character(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'+' | b'-')
}

/// A budget boundary is not a token boundary: never accept a partial link.
fn scan_link_part(text: &str, work: &mut usize, stop: impl Fn(char) -> bool) -> Option<usize> {
    let limit = text.floor_char_boundary((*work).min(text.len()));
    if let Some((end, ch)) = text[..limit].char_indices().find(|&(_, ch)| stop(ch)) {
        *work -= end + ch.len_utf8();
        return Some(end);
    }
    if limit < text.len() {
        *work = 0;
        return None;
    }
    *work -= limit;
    Some(limit)
}

/// GFM literal links consume a single word; punctuation outside a balanced URL stays ordinary text.
fn bare_autolink(text: &str, dialect: Dialect, has_email: bool, work: &mut usize) -> Option<usize> {
    let lower_prefix = |prefix: &str| {
        text.get(..prefix.len())
            .is_some_and(|s| s.eq_ignore_ascii_case(prefix))
    };
    let scheme = if lower_prefix("https://") {
        8
    } else if lower_prefix("http://") {
        7
    } else if dialect == Dialect::Extended && lower_prefix("ftp://") {
        6
    } else if lower_prefix("www.") {
        0
    } else if has_email {
        let bytes = text.as_bytes();
        let at = scan_link_part(text, work, |ch| {
            !ch.is_ascii() || !email_character(ch as u8)
        })?;
        if bytes.get(at) != Some(&b'@') {
            return None;
        }
        let domain = &text[at + 1..];
        let count = scan_link_part(domain, work, |ch| {
            !ch.is_ascii_alphanumeric() && !matches!(ch, '-' | '.')
        })?;
        let domain = domain[..count].trim_end_matches('.');
        return (domain.contains('.')
            && domain.split('.').all(|segment| {
                !segment.is_empty() && !segment.starts_with('-') && !segment.ends_with('-')
            }))
        .then_some(at + 1 + domain.len());
    } else {
        return None;
    };
    let terminates = |ch: char| {
        ch.is_whitespace() || ch == '<' || dialect != Dialect::Extended && matches!(ch, '>' | '"')
    };
    *work = work.saturating_sub(scheme);
    let host_end = scheme
        + scan_link_part(&text[scheme..], work, |ch| {
            matches!(ch, '/' | ':' | '?' | '#') || terminates(ch)
        })?;
    let authority = &text[scheme..host_end];
    if !authority.contains('.') || authority.starts_with('.') {
        return None;
    }
    let rest = &text[host_end..];
    let end = if let Some(path) = rest.strip_prefix(['/', ':', '?', '#']) {
        host_end + 1 + scan_link_part(path, work, terminates)?
    } else {
        host_end
    };
    let url = &text[..end];
    // GFM excludes a trailing entity-shaped suffix from a literal URL, even
    // when the name itself is unknown.
    let mut adjusted = url;
    while let Some(last) = adjusted.as_bytes().last() {
        if dialect == Dialect::Extended
            && *last == b';'
            && let Some(before) = adjusted[..adjusted.len() - 1]
                .trim_end_matches(|c: char| c.is_ascii_alphanumeric())
                .strip_suffix('&')
            && before.len() + 2 < adjusted.len()
        {
            adjusted = before;
        } else if matches!(
            last,
            b'.' | b',' | b'!' | b'?' | b':' | b';' | b'*' | b'_' | b'~'
        ) {
            adjusted = &adjusted[..adjusted.len() - 1];
        } else {
            break;
        }
    }
    let opening = adjusted.bytes().filter(|&b| b == b'(').count();
    let closing = adjusted.bytes().filter(|&b| b == b')').count();
    for _ in 0..closing.saturating_sub(opening) {
        let Some(cut) = adjusted.strip_suffix(')') else {
            break;
        };
        adjusted = cut;
    }
    Some(adjusted.len())
}
