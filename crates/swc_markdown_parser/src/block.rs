mod container;

use std::{borrow::Cow, cell::Cell, collections::HashMap};

use self::container::ContainerCursor;
use crate::{
    Alignment, Block, BlockExtension, BlockKind, Code, Definition, DefinitionEntry, Description,
    DescriptionEntry, Dialect, Fence, FrontMatter, Heading, Ignore, ParseOptions, RecoveryReason,
    SourceLine, Span, Table, TableCell, identifier, inline,
};

pub(crate) struct Parser<'a> {
    source: &'a str,
    options: ParseOptions,
    reference_definitions: Option<&'a HashMap<String, usize>>,
    /// Reused to collect one container's projected rows before building its children.
    projected_lines: Vec<SourceLine>,
    pub needs_reference_lookup: Cell<bool>,
    pub definitions: Vec<DefinitionEntry>,
    pub first_definitions: HashMap<String, usize>,
}
impl<'a> Parser<'a> {
    pub fn new(source: &'a str, options: ParseOptions) -> Self {
        Self {
            source,
            options,
            reference_definitions: None,
            projected_lines: Vec::new(),
            needs_reference_lookup: Cell::new(false),
            definitions: Vec::new(),
            first_definitions: HashMap::new(),
        }
    }

    pub fn with_reference_definitions(mut self, definitions: &'a HashMap<String, usize>) -> Self {
        self.reference_definitions = Some(definitions);
        self
    }

    fn inline_options(&self) -> inline::Options<'_> {
        inline::Options {
            parse: self.options,
            references: match self.reference_definitions {
                Some(definitions) => inline::References::Indexed(definitions),
                None => inline::References::Pending(&self.needs_reference_lookup),
            },
        }
    }

    pub fn blocks(&mut self, lines: Cow<'_, [SourceLine]>, depth: usize, root: bool) -> Vec<Block> {
        self.blocks_in(lines, depth, root, None)
    }

    fn blocks_in(
        &mut self,
        lines: Cow<'_, [SourceLine]>,
        depth: usize,
        root: bool,
        mut containers: Option<ContainerCursor<'_>>,
    ) -> Vec<Block> {
        let source = self.source;
        let mut result = Vec::new();
        // Unclosed template tags may appear on many lines. Each delimiter search
        // resumes only after advancing past its previous result.
        let mut next_liquid_end = [None; 2];
        let mut p = 0;
        let mut definition_paragraph = false;
        while p < lines.len() {
            let rest = &lines[p..];
            let line = lines[p];
            let first = line.first_content_byte(source);
            let (n, kind) = if depth >= self.options.max_nesting {
                (rest.len(), BlockKind::Recovery(RecoveryReason::Nesting))
            } else if definition_paragraph && continues_paragraph(source, line) {
                // Leading definitions are removed from the open paragraph. Its
                // remaining lines still follow paragraph continuation rules.
                self.definition(rest)
                    .unwrap_or_else(|| self.paragraph(rest))
            } else if first.is_none() {
                (
                    1 + rest[1..].iter().take_while(|l| l.blank(source)).count(),
                    BlockKind::Blank,
                )
            } else if let Some(front) = (root && p == 0)
                .then(|| frontmatter(source, rest, self.options.dialect))
                .flatten()
            {
                front
            } else if line.indent(source) >= 4 {
                self.indented(rest)
            } else if matches!(first, Some(b'\x60' | b'~'))
                && let Some(mark) = fence_open(source, line)
            {
                self.fence(rest, mark)
            } else if first == Some(b'#')
                && let Some(head) = atx(source, line, self.inline_options())
            {
                (1, BlockKind::Heading(head))
            } else if matches!(first, Some(b'*' | b'_' | b'-')) && thematic(source, line) {
                (1, BlockKind::ThematicBreak)
            } else if first == Some(b'>')
                && let Some(inner) = quote(source, line)
            {
                self.quote(
                    rest,
                    depth,
                    inner,
                    containers.as_mut().map(|scan| scan.at(p)),
                )
            } else if matches!(first, Some(b'*' | b'+' | b'-' | b'0'..=b'9'))
                && let Some(marker) = list_marker(source, line)
            {
                self.list(
                    rest,
                    depth,
                    marker,
                    containers.as_mut().map(|scan| scan.at(p)),
                )
            } else if matches!(first, Some(b'[' | b'$'))
                && let Some(special) = self.extension(rest, depth)
            {
                special
            } else if first == Some(b'[')
                && let Some(definition) = self.definition(rest)
            {
                definition
            } else if matches!(first, Some(b'<' | b'{'))
                && let Some(special) = opaque(source, rest, |delimiter| {
                    let index = usize::from(delimiter == "}}");
                    let at = next_liquid_end[index]
                        .filter(|&i| i >= p)
                        .unwrap_or_else(|| {
                            let next = rest
                                .iter()
                                .position(|line| line.content.text(source).contains(delimiter))
                                .map_or(usize::MAX, |i| p + i);
                            next_liquid_end[index] = Some(next);
                            next
                        });
                    (at != usize::MAX).then(|| at - p)
                })
            {
                special
            } else if self.options.dialect == Dialect::Extended
                && let Some(list) = self.description_list(rest, depth)
            {
                list
            } else {
                self.paragraph(rest)
            };
            definition_paragraph = matches!(kind, BlockKind::Definition(_));
            let span = extent(&rest[..n]);
            if p == 0 && n == lines.len() {
                return vec![Block {
                    span,
                    lines: lines.into_owned(),
                    kind,
                }];
            }
            result.push(Block {
                span,
                lines: rest[..n].to_vec(),
                kind,
            });
            p += n;
        }
        result
    }

    fn definition(&mut self, lines: &[SourceLine]) -> Option<(usize, BlockKind)> {
        let (n, definition) = parse_definition(self.source, lines)?;
        if self.reference_definitions.is_none() {
            self.first_definitions
                .entry(definition.identifier.clone())
                .or_insert(self.definitions.len());
            self.definitions.push(DefinitionEntry {
                span: extent(&lines[..n]),
                definition: definition.clone(),
            });
        }
        Some((n, BlockKind::Definition(definition)))
    }

    fn fence(&self, lines: &[SourceLine], mark: FenceMark) -> (usize, BlockKind) {
        let source = self.source;
        let mut body = Vec::new();
        let mut closing = None;
        let mut n = 1;
        for &line in &lines[1..] {
            n += 1;
            if let Some(marker) = fence_close(source, line, mark.byte, mark.run.len()) {
                closing = Some(marker);
                break;
            }
            body.push(line.strip(source, mark.indent));
        }
        let first = lines[0];
        let info = trim(source, Span::new(mark.run.end, first.content.end));
        let (lang, meta) = if info.is_empty() {
            (None, None)
        } else {
            let end = info
                .text(source)
                .bytes()
                .position(|b| matches!(b, b' ' | b'\t'))
                .map_or(info.end, |n| info.start + n);
            let meta = trim(source, Span::new(end, info.end));
            (
                Some(Span::new(info.start, end)),
                (!meta.is_empty()).then_some(meta),
            )
        };
        (
            n,
            BlockKind::Fence(Fence {
                opening: mark.run,
                closing,
                info: (!info.is_empty()).then_some(info),
                lang,
                meta,
                code: Code::from_lines(source, body, lines[0].ending.end),
            }),
        )
    }

    fn indented(&self, lines: &[SourceLine]) -> (usize, BlockKind) {
        let source = self.source;
        let mut n = 1;
        while n < lines.len() && (lines[n].blank(source) || lines[n].indent(source) >= 4) {
            n += 1;
        }
        while n > 1 && lines[n - 1].blank(source) {
            n -= 1;
        }
        let body = lines[..n].iter().map(|l| l.strip(source, 4)).collect();
        (
            n,
            BlockKind::IndentedCode(Code::from_lines(source, body, lines[0].content.start)),
        )
    }

    fn paragraph(&self, lines: &[SourceLine]) -> (usize, BlockKind) {
        let source = self.source;
        let mut n = 1;
        while let Some(&line) = lines.get(n) {
            match paragraph_continuation(source, lines[n - 1].content, line) {
                ParagraphContinuation::Text => n += 1,
                ParagraphContinuation::End => break,
                ParagraphContinuation::Heading(level, closing) => {
                    let children = paragraph_inline(source, &lines[..n], self.inline_options());
                    return (
                        n + 1,
                        BlockKind::Heading(Heading {
                            level,
                            opening: None,
                            closing: Some(closing),
                            children,
                        }),
                    );
                }
                ParagraphContinuation::Table(header, alignments) => {
                    if n == 1 {
                        return self.table(lines, header, alignments);
                    }
                    // The last paragraph line is the header of the next block.
                    n -= 1;
                    break;
                }
            }
        }
        (
            n,
            BlockKind::Paragraph(paragraph_inline(source, &lines[..n], self.inline_options())),
        )
    }

    fn table(
        &self,
        lines: &[SourceLine],
        header: Vec<Span>,
        alignments: Vec<Alignment>,
    ) -> (usize, BlockKind) {
        let cells = |line, spans: Vec<Span>| {
            spans
                .into_iter()
                .map(|span| table_cell(self.source, line, span, self.inline_options()))
                .collect()
        };
        let rows: Vec<_> = lines[2..]
            .iter()
            .copied()
            .take_while(|&line| table_row(self.source, line))
            .map(|line| cells(line, cell_spans(self.source, line.content)))
            .collect();
        (
            2 + rows.len(),
            BlockKind::Table(Table {
                header: cells(lines[0], header),
                delimiter: lines[1].content,
                alignments,
                rows,
            }),
        )
    }
}

fn extent(lines: &[SourceLine]) -> Span {
    Span::new(lines[0].start(), lines[lines.len() - 1].content.end)
}

fn trim(source: &str, span: Span) -> Span {
    let text = span.text(source);
    let left = text.trim_start_matches([' ', '\t']);
    let start = span.end - left.len();
    Span::new(start, start + left.trim_end_matches([' ', '\t']).len())
}
fn visible(source: &str, line: SourceLine) -> Option<SourceLine> {
    match line.indent(source) {
        0 => Some(line),
        indent @ 1..=3 => Some(line.strip(source, indent)),
        _ => None,
    }
}
fn paragraph_inline(
    source: &str,
    lines: &[SourceLine],
    options: inline::Options<'_>,
) -> Vec<crate::Inline> {
    let mut body = Cow::Borrowed(lines);
    for (index, &line) in lines.iter().enumerate() {
        let indent = line.indent(source);
        let mut normalized = if indent == 0 {
            line
        } else {
            line.strip(source, indent)
        };
        if index + 1 == lines.len() {
            normalized.content.end = normalized.content.start
                + normalized
                    .content
                    .text(source)
                    .trim_end_matches([' ', '\t'])
                    .len();
        }
        if indent != 0 || normalized.content.end != line.content.end {
            body.to_mut()[index] = normalized;
        }
    }
    inline::parse_with_original_lines(source, &body, lines, options)
}
fn setext(source: &str, line: SourceLine) -> Option<(u8, Span)> {
    if line.lazy {
        return None;
    }
    let span = trim(source, visible(source, line)?.content);
    let text = span.text(source).as_bytes();
    let &byte = text.first()?;
    let level = match byte {
        b'=' => 1,
        b'-' => 2,
        _ => return None,
    };
    text.iter().all(|&b| b == byte).then_some((level, span))
}
fn thematic(source: &str, line: SourceLine) -> bool {
    let Some(line) = visible(source, line) else {
        return false;
    };
    let mut bytes = line
        .content
        .text(source)
        .bytes()
        .filter(|b| !matches!(b, b' ' | b'\t'));
    let Some(mark @ (b'-' | b'*' | b'_')) = bytes.next() else {
        return false;
    };
    let mut n = 1;
    for b in bytes {
        if b != mark {
            return false;
        }
        n += 1;
    }
    n >= 3
}
fn atx(source: &str, line: SourceLine, options: inline::Options<'_>) -> Option<Heading> {
    let mut line = visible(source, line)?;
    let bytes = line.content.text(source).as_bytes();
    if bytes.first() != Some(&b'#') {
        return None;
    }
    let n = inline::run(bytes, 0, bytes.len());
    if n > 6 || bytes.get(n).is_some_and(|b| !matches!(b, b' ' | b'\t')) {
        return None;
    }
    let opening = Span::new(line.content.start, line.content.start + n);
    line = line.marker(n);
    let after = line.content;
    line.content = trim(source, after);
    let value = line.content.text(source);
    let rest = value.trim_end_matches('#');
    let closing = (rest.len() != value.len() && (rest.is_empty() || rest.ends_with([' ', '\t'])))
        .then(|| Span::new(line.content.start + rest.len(), line.content.end));
    if let Some(close) = closing {
        line.content = trim(source, Span::new(line.content.start, close.start));
    }
    Some(Heading {
        level: n as u8,
        opening: Some(opening),
        closing,
        children: inline::parse(source, &[line], options),
    })
}

#[derive(Clone, Copy)]
struct FenceMark {
    run: Span,
    byte: u8,
    indent: usize,
}
fn fence_open(source: &str, line: SourceLine) -> Option<FenceMark> {
    let indent = line.indent(source);
    let line = visible(source, line)?;
    let text = line.content.text(source).as_bytes();
    let &byte @ (b'\x60' | b'~') = text.first()? else {
        return None;
    };
    let n = inline::run(text, 0, text.len());
    if n < 3 || byte == b'\x60' && text[n..].contains(&byte) {
        return None;
    }
    Some(FenceMark {
        run: Span::new(line.content.start, line.content.start + n),
        byte,
        indent,
    })
}
fn fence_close(source: &str, line: SourceLine, byte: u8, width: usize) -> Option<Span> {
    let line = visible(source, line)?;
    let text = line.content.text(source).as_bytes();
    if text.first() != Some(&byte) {
        return None;
    }
    let n = inline::run(text, 0, text.len());
    (n >= width && text[n..].iter().all(|b| matches!(b, b' ' | b'\t')))
        .then_some(Span::new(line.content.start, line.content.start + n))
}

#[inline]
pub(crate) fn quote(source: &str, line: SourceLine) -> Option<SourceLine> {
    let bytes = line.content.bytes(source);
    let line = if line.virtual_spaces == 0 && bytes.first() == Some(&b'>') {
        line
    } else {
        let visible = visible(source, line)?;
        if visible.content.bytes(source).first() != Some(&b'>') {
            return None;
        }
        visible
    };
    let after = line.marker(1);
    match after.content.bytes(source).first() {
        Some(b' ') => Some(after.marker(1)),
        Some(b'\t') => Some(after.strip(source, 1)),
        _ => Some(after),
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Marker {
    token: Span,
    pub(crate) body: SourceLine,
    width: usize,
    delimiter: u8,
    number: Option<u64>,
}
pub(crate) fn list_marker(source: &str, line: SourceLine) -> Option<Marker> {
    let i = line.indent(source);
    let start = visible(source, line)?;
    let s = start.content.text(source);
    let bytes = s.as_bytes();
    let first = *bytes.first()?;
    let (n, number) = match first {
        b'-' | b'+' | b'*' => (1, None),
        b'0'..=b'9' => {
            let digits = bytes
                .iter()
                .take(10)
                .take_while(|b| b.is_ascii_digit())
                .count();
            if digits == 10 || !matches!(bytes.get(digits), Some(b'.' | b')')) {
                return None;
            }
            (digits + 1, Some(s[..digits].parse().ok()?))
        }
        _ => return None,
    };
    if bytes.get(n).is_some_and(|b| !matches!(b, b' ' | b'\t')) {
        return None;
    }
    let token = Span::new(start.content.start, start.content.start + n);
    let after = start.marker(n);
    let indent = after.indent(source);
    let pad = if (1..=4).contains(&indent) && !after.blank(source) {
        indent
    } else {
        1
    };
    Some(Marker {
        token,
        body: after.strip(source, pad),
        width: i + n + pad,
        delimiter: bytes[n - 1],
        number,
    })
}
enum ParagraphContinuation {
    Text,
    End,
    Heading(u8, Span),
    Table(Vec<Span>, Vec<Alignment>),
}

fn paragraph_continuation(source: &str, previous: Span, line: SourceLine) -> ParagraphContinuation {
    let first = line.first_content_byte(source);
    if matches!(first, Some(b'-' | b'='))
        && let Some((level, marker)) = setext(source, line)
    {
        ParagraphContinuation::Heading(level, marker)
    } else if !continues_paragraph(source, line) {
        ParagraphContinuation::End
    } else if !line.lazy
        && matches!(first, Some(b'-' | b':' | b'|'))
        && let Some((header, alignments)) = table_head(source, previous, line)
    {
        ParagraphContinuation::Table(header, alignments)
    } else {
        ParagraphContinuation::Text
    }
}

fn can_continue(source: &str, line: SourceLine) -> bool {
    !line.blank(source) && !interrupts(source, line)
}
fn lazy_candidate(source: &str, line: SourceLine) -> bool {
    // Even an empty item or a list starting after one can open a sibling block.
    can_continue(source, line) && list_marker(source, line).is_none()
}
fn continues_paragraph(source: &str, line: SourceLine) -> bool {
    can_continue(source, line) && !(line.lazy && complete_html_tag(source, line))
}
fn interrupts(source: &str, line: SourceLine) -> bool {
    let Some(v) = visible(source, line) else {
        return false;
    };
    let s = v.content.text(source);
    let Some(&b) = s.as_bytes().first() else {
        return false;
    };
    match b {
        b'{' => liquid_open(s).is_some(),
        b'[' => footnote_label(s).is_some(),
        b'$' => math_open(s).is_some(),
        b'>' => true,
        b'#' => {
            let run = s.bytes().take_while(|&b| b == b'#').count();
            run <= 6
                && s.as_bytes()
                    .get(run)
                    .is_none_or(|b| matches!(b, b' ' | b'\t'))
        }
        b'\x60' | b'~' => fence_open(source, line).is_some(),
        b'-' | b'*' | b'_' | b'+' | b'0'..=b'9' => {
            thematic(source, line)
                || list_marker(source, line)
                    .is_some_and(|m| m.number.is_none_or(|n| n == 1) && !m.body.blank(source))
        }
        b'<' => html_mode(s).is_some_and(|m| !matches!(m, HtmlEnd::CompleteTag)),
        _ => false,
    }
}

fn frontmatter(source: &str, lines: &[SourceLine], dialect: Dialect) -> Option<(usize, BlockKind)> {
    let s = lines[0].content.text(source);
    let mark = if s.starts_with("---") {
        "---"
    } else if s.starts_with("+++") {
        "+++"
    } else {
        return None;
    };
    let language = s[3..].trim();
    if dialect == Dialect::Extended && !s[3..].trim_matches([' ', '\t']).is_empty() {
        return None;
    }
    if !language
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
    {
        return None;
    }
    let i = lines[1..].iter().position(|l| {
        let closing = l.content.text(source).trim_end_matches([' ', '\t']);
        closing == mark
            || mark == "---" && (language.is_empty() || language == "yaml") && closing == "..."
    })? + 1;
    let body = Span::new(lines[0].ending.end, lines[i].raw.start);
    Some((
        i + 1,
        BlockKind::FrontMatter(FrontMatter {
            opening: Span::new(lines[0].content.start, lines[0].content.start + 3),
            closing: Span::new(lines[i].content.start, lines[i].content.start + 3),
            body,
        }),
    ))
}

fn parse_definition(source: &str, lines: &[SourceLine]) -> Option<(usize, Definition)> {
    let first_text = lines[0]
        .content
        .text(source)
        .trim_start_matches([' ', '\t']);
    if !first_text.starts_with('[') {
        return None;
    }
    let continuation =
        |line: SourceLine| continues_paragraph(source, line) && setext(source, line).is_none();
    let mut input = inline::Input::new(source, &lines[..1]);
    let start = input.text.len() - input.text.trim_start_matches([' ', '\t']).len();
    let mut close = start + 1;
    let mut n = 1;
    while close - start <= 4000 {
        let bytes = input.text.as_bytes();
        match bytes.get(close) {
            Some(b'\\') if close + 1 < bytes.len() => close += 2,
            Some(b'[') => return None,
            Some(b']') => break,
            Some(_) => close += 1,
            None => {
                let line = lines
                    .get(n)
                    .copied()
                    .filter(|&line| n < 32 && continuation(line))?;
                input.push_line(source, line);
                n += 1;
            }
        }
    }
    if close - start > 4000
        || input.text.as_bytes().get(close + 1) != Some(&b':')
        || !inline::link_label_has_text(&input.text[start + 1..close])
    {
        return None;
    }

    let after = close + 2;
    loop {
        let following = lines.get(n).copied().filter(|&line| continuation(line));
        let candidate = inline::destination_and_title(&input.text, after, input.text.len(), false);
        if let Ok((_, destination, title)) = candidate {
            let mut definition = Definition {
                brackets: [input.span(start, start + 1), input.span(close, close + 1)],
                separator: input.span(close + 1, after),
                label: input.span(start + 1, close),
                identifier: identifier(&input.text[start + 1..close]),
                destination: input.span(destination.start, destination.end),
                title: title.map(|t| input.span(t.start, t.end)),
            };
            if title.is_none()
                && following.is_some_and(|l| {
                    l.content
                        .text(source)
                        .trim_start()
                        .starts_with(['\'', '"', '('])
                })
            {
                for (index, &line) in lines.iter().enumerate().skip(n).take(32) {
                    if !continuation(line) {
                        break;
                    }
                    input.push_line(source, line);
                    let title =
                        inline::destination_and_title(&input.text, after, input.text.len(), false);
                    if let Ok((_, _, Some(title))) = title {
                        definition.title = Some(input.span(title.start, title.end));
                        return Some((index + 1, definition));
                    }
                }
            }
            return Some((n, definition));
        }
        let following = following.filter(|_| n < 32)?;
        let tail = input.text[after..].trim();
        let may_continue = tail.is_empty()
            || tail
                .split_once(char::is_whitespace)
                .is_some_and(|(_, title)| title.trim_start().starts_with(['\'', '"', '(']));
        if !may_continue {
            return None;
        }
        input.push_line(source, following);
        n += 1;
    }
}

fn cell_spans(source: &str, span: Span) -> Vec<Span> {
    let span = trim(source, span);
    let bytes = span.text(source).as_bytes();
    if bytes.is_empty() {
        return Vec::new();
    }
    let mut cells = Vec::new();
    let mut begin = usize::from(bytes[0] == b'|');
    let mut p = begin;
    while let Some(offset) = memchr::memchr2(b'\\', b'|', &bytes[p..]) {
        p += offset;
        match bytes[p] {
            b'\\' if p + 1 < bytes.len() => {
                p += 2;
                continue;
            }
            b'|' => {
                cells.push(trim(source, Span::new(span.start + begin, span.start + p)));
                begin = p + 1;
            }
            _ => {}
        }
        p += 1;
    }
    if begin < bytes.len() || cells.is_empty() {
        cells.push(trim(source, Span::new(span.start + begin, span.end)));
    }
    cells
}
fn table_head(
    source: &str,
    header: Span,
    delimiter: SourceLine,
) -> Option<(Vec<Span>, Vec<Alignment>)> {
    let delimiter = visible(source, delimiter)?.content;
    let text = delimiter.text(source);
    if !text.starts_with(['|', ':', '-'])
        || !(text.contains('|') || header.text(source).contains('|'))
    {
        return None;
    }
    let cells = cell_spans(source, delimiter);
    let mut alignments = Vec::new();
    for cell in cells {
        let s = cell.text(source);
        let left = s.starts_with(':');
        let right = s.ends_with(':');
        let dashes = s.strip_prefix(':').unwrap_or(s);
        let dashes = dashes.strip_suffix(':').unwrap_or(dashes);
        if dashes.is_empty() || !dashes.bytes().all(|b| b == b'-') {
            return None;
        }
        alignments.push(match (left, right) {
            (true, true) => Alignment::Center,
            (true, false) => Alignment::Left,
            (false, true) => Alignment::Right,
            _ => Alignment::None,
        });
    }
    let header = cell_spans(source, header);
    (!header.is_empty() && header.len() == alignments.len()).then_some((header, alignments))
}
fn table_row(source: &str, line: SourceLine) -> bool {
    !line.lazy
        && line.indent(source) < 4
        && can_continue(source, line)
        && list_marker(source, line).is_none()
        && !complete_html_tag(source, line)
}

fn table_cell(
    source: &str,
    mut line: SourceLine,
    span: Span,
    options: inline::Options<'_>,
) -> TableCell {
    line.content = span;
    line.virtual_spaces = 0;
    TableCell {
        span,
        children: inline::parse(source, &[line], options),
    }
}

#[derive(Clone, Copy)]
enum HtmlEnd {
    Marker(&'static str),
    RawTag(&'static str),
    Blank,
    CompleteTag,
}
impl HtmlEnd {
    fn closes_on(self, source: &str, line: SourceLine) -> bool {
        match self {
            Self::Marker(end) => line.content.text(source).contains(end),
            Self::RawTag(end) => line
                .content
                .text(source)
                .as_bytes()
                .windows(end.len())
                .any(|bytes| bytes.eq_ignore_ascii_case(end.as_bytes())),
            Self::Blank | Self::CompleteTag => line.blank(source),
        }
    }
}

fn complete_html_tag(source: &str, line: SourceLine) -> bool {
    visible(source, line).is_some_and(|line| {
        matches!(
            html_mode(line.content.text(source)),
            Some(HtmlEnd::CompleteTag)
        )
    })
}

fn html_mode(s: &str) -> Option<HtmlEnd> {
    let rest = s.strip_prefix('<')?;
    for (open, close) in [("!--", "-->"), ("?", "?>"), ("![CDATA[", "]]>")] {
        if rest.starts_with(open) {
            return Some(HtmlEnd::Marker(close));
        }
    }
    if rest
        .strip_prefix('!')
        .and_then(|s| s.as_bytes().first())
        .is_some_and(u8::is_ascii_alphabetic)
    {
        return Some(HtmlEnd::Marker(">"));
    }
    let name = rest.strip_prefix('/').unwrap_or(rest);
    let end = name
        .bytes()
        .take_while(|b| b.is_ascii_alphanumeric() || *b == b'-')
        .count();
    if end == 0
        || name
            .as_bytes()
            .get(end)
            .is_some_and(|b| !matches!(b, b'>' | b' ' | b'\t' | b'/'))
    {
        return None;
    }
    let tag = &name[..end];
    if !rest.starts_with('/') {
        for (raw, closing) in [
            ("pre", "</pre>"),
            ("script", "</script>"),
            ("style", "</style>"),
            ("textarea", "</textarea>"),
        ] {
            if tag.eq_ignore_ascii_case(raw) {
                return Some(HtmlEnd::RawTag(closing));
            }
        }
    }
    const BLOCK: &[&str] = &[
        "address",
        "article",
        "aside",
        "base",
        "basefont",
        "blockquote",
        "body",
        "caption",
        "center",
        "col",
        "colgroup",
        "dd",
        "details",
        "dialog",
        "dir",
        "div",
        "dl",
        "dt",
        "fieldset",
        "figcaption",
        "figure",
        "footer",
        "form",
        "frame",
        "frameset",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "head",
        "header",
        "hgroup",
        "hr",
        "html",
        "iframe",
        "legend",
        "li",
        "link",
        "main",
        "menu",
        "menuitem",
        "nav",
        "noframes",
        "ol",
        "optgroup",
        "option",
        "p",
        "param",
        "search",
        "section",
        "summary",
        "table",
        "tbody",
        "td",
        "tfoot",
        "th",
        "thead",
        "title",
        "tr",
        "track",
        "ul",
    ];
    if BLOCK.iter().any(|name| tag.eq_ignore_ascii_case(name)) {
        return Some(HtmlEnd::Blank);
    }
    // A complete standalone tag can start its own block, but cannot interrupt a paragraph.
    inline::html_tag(s.as_bytes())
        .ok()
        .filter(|&end| {
            s.as_bytes()[end..]
                .iter()
                .all(|byte| matches!(byte, b' ' | b'\t'))
        })
        .map(|_| HtmlEnd::CompleteTag)
}

fn opaque(
    source: &str,
    lines: &[SourceLine],
    liquid_end: impl FnOnce(&str) -> Option<usize>,
) -> Option<(usize, BlockKind)> {
    let first = visible(source, lines[0])?;
    let s = first.content.text(source).trim_end_matches([' ', '\t']);
    if let Some(content) = s.strip_prefix("<!--").and_then(|s| s.strip_suffix("-->")) {
        let kind = match content.trim() {
            "prettier-ignore" => Some(Ignore::Next),
            "prettier-ignore-start" => Some(Ignore::Start),
            "prettier-ignore-end" => Some(Ignore::End),
            _ => None,
        };
        if let Some(kind) = kind {
            return Some((1, BlockKind::Ignore(kind)));
        }
    }
    if let Some(mode) = html_mode(s) {
        let includes_closer = matches!(mode, HtmlEnd::Marker(_) | HtmlEnd::RawTag(_));
        let n = lines
            .iter()
            .position(|&line| mode.closes_on(source, line))
            .map_or(lines.len(), |i| i + usize::from(includes_closer));
        return Some((n, BlockKind::Html));
    }
    if let Some(delimiter) = liquid_open(s) {
        let i = liquid_end(delimiter)?;
        let (prefix, tail) = lines[i].content.text(source).split_once(delimiter)?;
        if tail.trim().is_empty() {
            let open = first.content.start;
            let close = lines[i].content.start + prefix.len();
            return Some((
                i + 1,
                BlockKind::Extension(BlockExtension::Liquid {
                    opening: Span::new(open, open + 2),
                    closing: Span::new(close, close + delimiter.len()),
                }),
            ));
        }
    }
    None
}

fn math_open(text: &str) -> Option<usize> {
    let width = text.bytes().take_while(|b| *b == b'$').count();
    (width >= 2 && !text[width..].contains('$')).then_some(width)
}

fn footnote_label(text: &str) -> Option<&str> {
    let rest = text.strip_prefix("[^")?;
    let mut bytes = rest.bytes().enumerate();
    while let Some((i, byte)) = bytes.next() {
        match byte {
            b'\\'
                if rest
                    .as_bytes()
                    .get(i + 1)
                    .is_some_and(|b| matches!(b, b'[' | b']' | b'\\')) =>
            {
                bytes.next();
            }
            b']' => {
                return (i > 0 && rest.as_bytes().get(i + 1) == Some(&b':')).then(|| &rest[..i]);
            }
            b'[' | b' ' | b'\t' => return None,
            _ => {}
        }
    }
    None
}

fn liquid_open(text: &str) -> Option<&'static str> {
    let close = if text.starts_with("{%") {
        "%}"
    } else if text.starts_with("{{") {
        "}}"
    } else {
        return None;
    };
    text.split_once(close)
        .is_none_or(|(_, tail)| tail.trim().is_empty())
        .then_some(close)
}

impl Parser<'_> {
    fn extension(&mut self, lines: &[SourceLine], depth: usize) -> Option<(usize, BlockKind)> {
        let first = visible(self.source, lines[0])?;
        let text = first.content.text(self.source);
        if let Some(label) = footnote_label(text) {
            let label = Span::new(
                first.content.start + 2,
                first.content.start + 2 + label.len(),
            );
            let after = first.marker(label.len() + 4);
            return Some(self.footnote(lines, after, label, depth));
        }
        self.display_math(lines, first)
    }

    fn display_math(&self, lines: &[SourceLine], first: SourceLine) -> Option<(usize, BlockKind)> {
        let text = first.content.text(self.source);
        let extended = self.options.dialect == Dialect::Extended;
        let width = math_open(text)?;
        if extended && text.trim_end_matches([' ', '\t']) != "$$" {
            return None;
        }
        let opening = Span::new(first.content.start, first.content.start + width);
        let meta = trim(self.source, Span::new(opening.end, first.content.end));
        let closing = lines[1..]
            .iter()
            .take_while(|l| !extended || !l.blank(self.source))
            .enumerate()
            .find_map(|(i, line)| {
                let span = trim(self.source, visible(self.source, *line)?.content);
                let text = span.text(self.source);
                (text.len() >= width
                    && (!extended || text.len() == width)
                    && text.bytes().all(|b| b == b'$'))
                .then_some((i + 1, span))
            });
        if extended && closing.is_none() {
            return None;
        }
        let body_end = closing.map_or(lines.len(), |(i, _)| i);
        let indent = lines[0].indent(self.source);
        let body = lines[1..body_end]
            .iter()
            .map(|l| l.strip(self.source, indent))
            .collect();
        Some((
            body_end + usize::from(closing.is_some()),
            BlockKind::Extension(BlockExtension::Math {
                opening,
                closing: closing.map(|(_, span)| span),
                meta: (!meta.is_empty()).then_some(meta),
                body,
            }),
        ))
    }

    fn description_list(
        &mut self,
        lines: &[SourceLine],
        depth: usize,
    ) -> Option<(usize, BlockKind)> {
        let source = self.source;
        let marker = |line: SourceLine| -> Option<SourceLine> {
            let line = visible(source, line)?;
            let s = line.content.text(source).as_bytes();
            (!line.lazy
                && s.first() == Some(&b':')
                && s.get(1).is_none_or(|b| matches!(b, b' ' | b'\t')))
            .then_some(line)
        };
        // Bound the lookahead to one candidate paragraph. A blank before the
        // first definition is permitted, but it cannot connect unrelated terms.
        let terms_end = |start: usize| -> Option<(usize, usize)> {
            let mut end = start;
            while let Some(&line) = lines.get(end) {
                if !can_continue(source, line)
                    || marker(line).is_some()
                    || setext(source, line).is_some()
                {
                    break;
                }
                end += 1;
            }
            if end == start {
                return None;
            }
            let definition = end + lines[end..].iter().take_while(|l| l.blank(source)).count();
            marker(*lines.get(definition)?)?;
            Some((end, definition))
        };
        let mut groups = Vec::new();
        let mut count = 0;
        let mut group = terms_end(0)?;
        loop {
            let (term_end, definition) = group;
            let terms = lines[count..term_end]
                .iter()
                .map(|line| {
                    let line = std::slice::from_ref(line);
                    Block {
                        span: extent(line),
                        lines: line.to_vec(),
                        kind: BlockKind::Paragraph(paragraph_inline(
                            source,
                            line,
                            self.inline_options(),
                        )),
                    }
                })
                .collect();
            count = definition;
            let mut descriptions = Vec::new();
            while let Some(start) = lines.get(count).and_then(|&line| marker(line)) {
                let colon = Span::new(start.content.start, start.content.start + 1);
                let first = start.marker(1);
                let pad = first.indent(source).clamp(1, 4);
                let mut body = vec![first.strip(source, pad)];
                let mut body_indent = pad + 1;
                let begin = count;
                count += 1;
                for (i, &next) in lines.iter().enumerate().skip(count) {
                    if next.blank(source) {
                        continue;
                    }
                    // Blank lines belong to the definition only when the next
                    // block starts at least four columns in.
                    let after_blank = i > count;
                    let required = if after_blank { 4 } else { 2 };
                    if next.indent(source) < required {
                        break;
                    }
                    if after_blank {
                        body_indent = 4;
                    }
                    body.extend(
                        lines[count..=i]
                            .iter()
                            .map(|line| line.strip(source, body_indent)),
                    );
                    count = i + 1;
                }
                descriptions.push(Description {
                    marker: colon,
                    span: extent(&lines[begin..count]),
                    blocks: self.blocks(body.into(), depth + 1, false),
                });
            }
            groups.push(DescriptionEntry {
                terms,
                descriptions,
            });
            let next = count
                + lines[count..]
                    .iter()
                    .take_while(|l| l.blank(source))
                    .count();
            let Some(next_group) = terms_end(next) else {
                break;
            };
            group = next_group;
            count = next;
        }
        Some((
            count,
            BlockKind::Extension(BlockExtension::DescriptionList { entries: groups }),
        ))
    }
}
