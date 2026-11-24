//! Project physical lines into quotes, list items and footnotes, sharing the
//! open-container and leaf state needed to recognize unmarked paragraph continuations.

use std::ops::Range;

use super::{
    FenceMark, HtmlEnd, Marker, ParagraphContinuation, Parser, can_continue, complete_html_tag,
    extent, fence_close, fence_open, html_mode, lazy_candidate, list_marker,
    paragraph_continuation, paragraph_inline, quote, table_row, thematic, visible,
};
use crate::{
    Block, BlockExtension, BlockKind, Dialect, List, ListItem, MAX_DEPTH, SourceLine, Span, Task,
    inline,
};

impl Parser<'_> {
    pub(super) fn quote(
        &mut self,
        lines: &[SourceLine],
        depth: usize,
        first: SourceLine,
        parent: Option<ContainerCursor<'_>>,
    ) -> (usize, BlockKind) {
        let label = if self.options.dialect == Dialect::Extended {
            alert_label(self.source, first)
        } else {
            None
        };
        // An alert removes its label before determining whether its body has an open paragraph.
        let (first, parent) = match label {
            Some(label) => (first.marker(label.len() + 3), None),
            None => (first, parent),
        };
        let body = self.container_body(lines, depth, ContainerPrefix::Quote, first, parent);
        let kind = match label {
            Some(label) => BlockKind::Extension(BlockExtension::Alert {
                label,
                blocks: body.blocks,
            }),
            None => BlockKind::BlockQuote(body.blocks),
        };
        (body.consumed, kind)
    }

    pub(super) fn list(
        &mut self,
        lines: &[SourceLine],
        depth: usize,
        first: Marker,
        mut parent: Option<ContainerCursor<'_>>,
    ) -> (usize, BlockKind) {
        let source = self.source;
        let mut items = Vec::new();
        let mut p = 0;
        let mut mark = first;
        let mut consumed;
        loop {
            let mut body = self.container_body(
                &lines[p..],
                depth,
                ContainerPrefix::ListItem(mark.width),
                mark.body,
                parent.as_mut().map(|scan| scan.at(p)),
            );
            consumed = p + body.consumed;
            let task = body
                .blocks
                .first_mut()
                .and_then(|block| task(source, block, self.inline_options()));
            items.push(ListItem {
                span: extent(&lines[p..consumed]),
                marker: mark.token,
                task,
                blocks: body.blocks,
            });
            p += body.scanned;
            let Some(&line) = lines.get(p) else { break };
            if thematic(source, line) {
                break;
            }
            let Some(next) = list_marker(source, line) else {
                break;
            };
            if next.delimiter != first.delimiter {
                break;
            }
            mark = next;
        }
        (
            consumed,
            BlockKind::List(List {
                ordered: first.number.is_some(),
                start: first.number,
                items,
            }),
        )
    }

    #[inline(always)]
    fn container_body(
        &mut self,
        lines: &[SourceLine],
        depth: usize,
        prefix: ContainerPrefix,
        first: SourceLine,
        parent: Option<ContainerCursor<'_>>,
    ) -> ContainerBody {
        let source = self.source;
        let mut scan = ContainerScan::new(depth, prefix, first);
        let mut local = ContainerCursor {
            scan: &mut scan,
            lines,
            row: 0,
        };
        let mut parent = parent;
        if parent.as_mut().is_some_and(|cursor| !cursor.opened(depth)) {
            parent = None;
        }
        let body = &mut self.projected_lines;
        body.clear();
        body.push(first);
        let mut empty_item = matches!(prefix, ContainerPrefix::ListItem(_)) && first.blank(source);
        for (i, &line) in lines.iter().enumerate().skip(1) {
            if empty_item && i > 1 && !line.blank(source) {
                break;
            }
            let child = if let Some(child) = prefix.strip(source, line) {
                // A lazy row can still supply indentation to an inner list. Its
                // stripped remainder can open a different leaf than the ancestor saw.
                if line.lazy
                    && parent
                        .as_mut()
                        .is_some_and(|cursor| cursor.needs_local_view(source, i, depth))
                {
                    parent = None;
                }
                child
            } else if lazy_candidate(source, line) && local.accepts(source, i, &mut parent) {
                SourceLine { lazy: true, ..line }
            } else {
                break;
            };
            empty_item = empty_item && child.blank(source);
            body.push(child);
        }
        let scanned = body.len();
        if !matches!(prefix, ContainerPrefix::Quote) {
            body.truncate(
                body.iter()
                    .rposition(|line| !line.blank(source))
                    .map_or(1, |i| i + 1),
            );
        }
        let cursor = parent.unwrap_or(local);
        let shared =
            (!cursor.scan.records.is_empty() && !cursor.scan.reached_limit).then_some(cursor);
        let body = body.to_vec();
        let consumed = body.len();
        let blocks = if matches!(prefix, ContainerPrefix::Quote) && shared.is_none() {
            self.quoted_body(body, depth + 1)
        } else {
            self.blocks_in(body.into(), depth + 1, false, shared)
        };
        ContainerBody {
            scanned,
            consumed,
            blocks,
        }
    }

    /// Explicit quote markers shared by every row form a chain of single-child
    /// blocks. Materialize each level's source lines without rescanning its body.
    fn quoted_body(&mut self, mut lines: Vec<SourceLine>, depth: usize) -> Vec<Block> {
        let ends = self.shared_quote_markers(&lines, depth);
        let mut layers = Vec::with_capacity(ends.len());
        let mut previous = 0;
        for &end in &ends {
            let inside = lines
                .iter()
                .map(|line| line.marker(end - previous))
                .collect();
            layers.push(lines);
            lines = inside;
            previous = end;
        }
        let mut blocks = self.blocks(lines.into(), depth + ends.len(), false);
        for lines in layers.into_iter().rev() {
            blocks = vec![Block {
                span: extent(&lines),
                lines,
                kind: BlockKind::BlockQuote(blocks),
            }];
        }
        blocks
    }

    fn shared_quote_markers(&self, lines: &[SourceLine], depth: usize) -> Vec<usize> {
        let source = self.source;
        let first = lines[0];
        let mut ends = Vec::new();
        if first.lazy || first.virtual_spaces != 0 || first.first_content_byte(source) != Some(b'>')
        {
            return ends;
        }
        let spelling = first.content.bytes(source);
        let mut line = first;
        while depth + ends.len() < self.options.max_nesting {
            let Some(inner) = quote(source, line) else {
                break;
            };
            // A tab's width depends on the original column. An alert consumes its label.
            if source.as_bytes()[line.content.start..inner.content.start].contains(&b'\t')
                || self.options.dialect == Dialect::Extended && alert_label(source, inner).is_some()
            {
                break;
            }
            ends.push(inner.content.start - first.content.start);
            line = inner;
        }
        for &row in &lines[1..] {
            let Some(&end) = ends.last() else { break };
            if row.lazy || row.virtual_spaces != 0 {
                ends.clear();
                break;
            }
            let text = row.content.bytes(source);
            if !text.starts_with(&spelling[..end]) {
                let common = text
                    .iter()
                    .zip(&spelling[..end])
                    .take_while(|(a, b)| a == b)
                    .count();
                ends.truncate(ends.partition_point(|&end| end <= common));
            }
            // A bare final marker may consume an optional space or tab on this row.
            if let Some(&end) = ends.last()
                && spelling[end - 1] == b'>'
                && matches!(text.get(end), Some(b' ' | b'\t'))
            {
                ends.pop();
            }
        }
        ends
    }

    pub(super) fn footnote(
        &mut self,
        lines: &[SourceLine],
        after: SourceLine,
        label: Span,
        depth: usize,
    ) -> (usize, BlockKind) {
        let indent = after.indent(self.source);
        let first_body = after.strip(
            self.source,
            if self.options.dialect == Dialect::Extended {
                indent.min(4)
            } else {
                indent
            },
        );
        let body = self.container_body(lines, depth, ContainerPrefix::Footnote, first_body, None);
        (
            body.consumed,
            BlockKind::Extension(BlockExtension::Footnote {
                label,
                blocks: body.blocks,
            }),
        )
    }
}

fn alert_label(source: &str, line: SourceLine) -> Option<Span> {
    if line.virtual_spaces != 0 {
        return None;
    }
    let (name, tail) = line
        .content
        .text(source)
        .strip_prefix("[!")?
        .split_once(']')?;
    (matches!(name, "NOTE" | "TIP" | "CAUTION" | "WARNING" | "IMPORTANT")
        && tail
            .as_bytes()
            .first()
            .is_none_or(|b| matches!(b, b' ' | b'\t')))
    .then(|| Span::new(line.content.start + 2, line.content.start + 2 + name.len()))
}

impl Marker {
    pub(crate) fn continuation(&self, source: &str, line: SourceLine) -> Option<SourceLine> {
        ContainerPrefix::ListItem(self.width).strip(source, line)
    }
}

fn task(source: &str, block: &mut Block, options: inline::Options<'_>) -> Option<Task> {
    if !matches!(block.kind, BlockKind::Paragraph(_)) {
        return None;
    }
    let mut line = *block.lines.first()?;
    line = line.strip(source, line.indent(source));
    let bytes = line.content.text(source).as_bytes();
    let checked = match bytes.get(..3)? {
        b"[ ]" => false,
        b"[x]" | b"[X]" => true,
        _ => return None,
    };
    if bytes.get(3).is_some_and(|b| !matches!(b, b' ' | b'\t')) {
        return None;
    }
    let marker = Span::new(line.content.start, line.content.start + 3);
    let remainder = line.marker(3);
    let remainder = remainder.strip(source, remainder.indent(source));
    if remainder.blank(source) {
        if block.lines.len() == 1 {
            return None;
        }
        block.lines.remove(0);
    } else {
        block.lines[0] = remainder;
    }
    block.span = extent(&block.lines);
    block.kind = BlockKind::Paragraph(paragraph_inline(source, &block.lines, options));
    Some(Task { marker, checked })
}

#[derive(Clone, Copy)]
enum ContainerPrefix {
    Quote,
    ListItem(usize),
    Footnote,
}
impl ContainerPrefix {
    fn strip(self, source: &str, line: SourceLine) -> Option<SourceLine> {
        match self {
            Self::Quote => quote(source, line),
            Self::Footnote if line.blank(source) => Some(line),
            Self::Footnote => (line.indent(source) >= 4).then(|| line.strip(source, 4)),
            Self::ListItem(width) => {
                let body = line.strip(source, width);
                (body.column - line.column == width || body.content.is_empty()).then_some(body)
            }
        }
    }
}

enum OpenLeaf {
    None,
    /// The last content line may become a table header.
    Paragraph(Span),
    Table,
    Fence(FenceMark),
    Html(HtmlEnd),
}

struct ContainerBody {
    /// Trailing blanks can separate sibling list items without belonging to the last item.
    scanned: usize,
    consumed: usize,
    blocks: Vec<Block>,
}

pub(super) struct ContainerCursor<'a> {
    scan: &'a mut ContainerScan,
    lines: &'a [SourceLine],
    row: usize,
}

impl ContainerCursor<'_> {
    pub(super) fn at(&mut self, offset: usize) -> ContainerCursor<'_> {
        ContainerCursor {
            scan: self.scan,
            lines: self.lines,
            row: self.row + offset,
        }
    }

    fn accepts(
        &mut self,
        source: &str,
        offset: usize,
        parent: &mut Option<ContainerCursor<'_>>,
    ) -> bool {
        let depth = self.scan.depth;
        if let Some(shared) = parent {
            let accepted = shared.lazy_at(source, offset, depth);
            if !shared.scan.reached_limit {
                return accepted;
            }
            // The child can see farther from its own root when its ancestor's
            // lookahead stops at the nesting limit.
            *parent = None;
        }
        self.lazy_at(source, offset, depth)
    }

    fn lazy_at(&mut self, source: &str, offset: usize, depth: usize) -> bool {
        let level = (depth - self.scan.depth) as u8;
        matches!(self.record(source, offset), ContainerLine::Lazy(range) if range.contains(&level))
    }

    // Share only a previously scanned opener. A fresh subtree can scan its own
    // projected lines if a leaf consumes different syntax (e.g. a reference definition).
    fn opened(&self, depth: usize) -> bool {
        let level = (depth - self.scan.depth) as u8;
        !self.scan.reached_limit
            && matches!(self.scan.records.get(self.row), Some(ContainerLine::Opened(range)) if range.contains(&level))
    }

    fn needs_local_view(&mut self, source: &str, offset: usize, depth: usize) -> bool {
        let level = (depth - self.scan.depth) as u8;
        matches!(self.record(source, offset), ContainerLine::Lazy(range) if range.start < level)
            || self.scan.reached_limit
    }

    fn record(&mut self, source: &str, offset: usize) -> &ContainerLine {
        let row = self.row + offset;
        let scan = &mut self.scan;
        while scan.records.len() <= row {
            let i = scan.records.len();
            let line = self.lines[i];
            let record = if i == 0 {
                scan.observe(source, scan.first);
                ContainerLine::Opened(0..(1 + scan.stack.len()) as u8)
            } else if let Some(child) = scan.prefix.strip(source, line) {
                scan.observe(source, child)
            } else if lazy_candidate(source, line) && scan.continue_paragraph(source, line) {
                ContainerLine::Lazy(0..(1 + scan.stack.len()) as u8)
            } else {
                ContainerLine::None
            };
            scan.records.push(record);
        }
        &scan.records[row]
    }
}

enum ContainerLine {
    None,
    Opened(Range<u8>),
    Lazy(Range<u8>),
}

/// One forward scan for a container and its nested quotes and list items. The
/// AST builder can revisit earlier rows at another depth without replaying their prefixes.
struct ContainerScan {
    depth: usize,
    prefix: ContainerPrefix,
    first: SourceLine,
    stack: Vec<ContainerPrefix>,
    leaf: OpenLeaf,
    records: Vec<ContainerLine>,
    reached_limit: bool,
}

impl ContainerScan {
    fn new(depth: usize, prefix: ContainerPrefix, first: SourceLine) -> Self {
        Self {
            depth,
            prefix,
            first,
            stack: Vec::new(),
            leaf: OpenLeaf::None,
            records: Vec::new(),
            reached_limit: false,
        }
    }

    fn continue_paragraph(&mut self, source: &str, line: SourceLine) -> bool {
        if !matches!(self.leaf, OpenLeaf::Paragraph(_)) {
            return false;
        }
        self.leaf = if complete_html_tag(source, line) {
            OpenLeaf::Html(HtmlEnd::CompleteTag)
        } else {
            OpenLeaf::Paragraph(line.content)
        };
        true
    }

    fn observe(&mut self, source: &str, mut line: SourceLine) -> ContainerLine {
        for i in 0..self.stack.len() {
            let Some(child) = self.stack[i].strip(source, line) else {
                if lazy_candidate(source, line) && self.continue_paragraph(source, line) {
                    return ContainerLine::Lazy((1 + i) as u8..(1 + self.stack.len()) as u8);
                }
                self.stack.truncate(i);
                self.leaf = OpenLeaf::None;
                break;
            };
            line = child;
        }
        // Inside an opaque leaf, only its enclosing containers and its own
        // terminator have meaning. A list or quote marker in its body is literal.
        match self.leaf {
            OpenLeaf::Html(mode) => {
                if mode.closes_on(source, line) {
                    self.leaf = OpenLeaf::None;
                }
                return ContainerLine::None;
            }
            OpenLeaf::Fence(mark) => {
                if fence_close(source, line, mark.byte, mark.run.len()).is_some() {
                    self.leaf = OpenLeaf::None;
                }
                return ContainerLine::None;
            }
            OpenLeaf::Table if table_row(source, line) => return ContainerLine::None,
            OpenLeaf::Paragraph(previous) => match paragraph_continuation(source, previous, line) {
                ParagraphContinuation::Text => {
                    self.leaf = OpenLeaf::Paragraph(line.content);
                    return ContainerLine::None;
                }
                ParagraphContinuation::Heading(..) => {
                    self.leaf = OpenLeaf::None;
                    return ContainerLine::None;
                }
                ParagraphContinuation::Table(..) => {
                    self.leaf = OpenLeaf::Table;
                    return ContainerLine::None;
                }
                ParagraphContinuation::End => {}
            },
            _ => {}
        }
        let matched = self.stack.len();
        while self.stack.len() < MAX_DEPTH {
            let (prefix, child) = if let Some(child) = quote(source, line) {
                (ContainerPrefix::Quote, child)
            } else if let Some(mark) = list_marker(source, line)
                && !thematic(source, line)
            {
                (ContainerPrefix::ListItem(mark.width), mark.body)
            } else {
                break;
            };
            self.stack.push(prefix);
            line = child;
        }
        self.leaf = if let Some(mark) = fence_open(source, line) {
            OpenLeaf::Fence(mark)
        } else if let Some(mode) =
            visible(source, line).and_then(|line| html_mode(line.content.text(source)))
        {
            if mode.closes_on(source, line) {
                OpenLeaf::None
            } else {
                OpenLeaf::Html(mode)
            }
        } else if line.indent(source) < 4 && can_continue(source, line) {
            OpenLeaf::Paragraph(line.content)
        } else {
            OpenLeaf::None
        };
        if self.stack.len() > matched {
            self.reached_limit |= self.stack.len() == MAX_DEPTH;
            ContainerLine::Opened((1 + matched) as u8..(1 + self.stack.len()) as u8)
        } else {
            ContainerLine::None
        }
    }
}
