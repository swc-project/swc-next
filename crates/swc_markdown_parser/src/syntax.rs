//! A source-order, read-only view for tools that consume punctuation, fields or syntax ranges.
use crate::{
    Block, BlockExtension, BlockKind, Document, Inline, InlineExtension, InlineKind, LinkTarget,
    RecoveryReason, ReferenceKind, SourceLine, Span, TableCell,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyntaxKind {
    Document,
    Blank,
    Paragraph,
    Heading {
        level: u8,
        setext: bool,
    },
    Quote,
    ThematicBreak,
    List {
        ordered: bool,
    },
    ListItem,
    Code {
        fenced: bool,
    },
    LinkDefinition,
    Table,
    Row {
        header: bool,
    },
    Cell,
    FrontMatter,
    Html,
    Ignore,
    Raw,
    Text,
    /// A backslash and the ASCII punctuation it escapes, including inside link fields.
    Escape,
    CodeSpan,
    Emphasis,
    Strong,
    Delete,
    Link {
        image: bool,
        reference: Option<ReferenceKind>,
    },
    Autolink,
    InlineHtml,
    HardBreak,
    SoftBreak,
    Alert,
    FootnoteDefinition,
    FootnoteReference,
    DescriptionList,
    Term,
    Description,
    Math {
        display: bool,
    },
    Entity,
    Recovery(RecoveryReason),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyntaxRole {
    Marker,
    Separator,
    Label,
    Reference,
    Destination,
    Title,
    Info,
    Language,
    Metadata,
    Task { checked: bool },
    AlertLabel,
    FootnoteLabel,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SyntaxPart {
    pub role: SyntaxRole,
    pub span: Span,
}

/// Children are ordered and disjoint. Their ranges and the ranges yielded by
/// `gaps` partition the exact original source covered by this node.
#[derive(Debug)]
pub struct SyntaxNode<'a> {
    pub kind: SyntaxKind,
    pub span: Span,
    pub children: Vec<SyntaxNode<'a>>,
    pub parts: Vec<SyntaxPart>,
    /// Source lines for this node. Code nodes expose the logical guest body.
    pub lines: &'a [SourceLine],
    surface_lines: &'a [SourceLine],
}

impl<'a> SyntaxNode<'a> {
    fn new(kind: SyntaxKind, span: Span, lines: &'a [SourceLine]) -> Self {
        Self {
            kind,
            span,
            children: Vec::new(),
            parts: Vec::new(),
            surface_lines: lines,
            lines,
        }
    }
    fn part(&mut self, role: SyntaxRole, span: Span) {
        self.parts.push(SyntaxPart { role, span });
    }
    fn optional(&mut self, role: SyntaxRole, span: Option<Span>) {
        if let Some(span) = span {
            self.part(role, span);
        }
    }
    /// Lines inside a quote, alert or list item after consuming its own prefix.
    /// Pass the original document source. Blank lines and fence delimiters are
    /// included. Task markers, alert labels and deeper container prefixes remain
    /// in the content. A lazy continuation retains its unconsumed indentation.
    pub fn container_lines<'s>(
        &'s self,
        source: &'s str,
    ) -> Option<impl Iterator<Item = SourceLine> + 's> {
        let item = match self.kind {
            SyntaxKind::ListItem => Some(crate::block::list_marker(source, *self.lines.first()?)?),
            SyntaxKind::Quote | SyntaxKind::Alert => None,
            _ => return None,
        };
        Some(
            self.lines
                .iter()
                .copied()
                .enumerate()
                .map(move |(i, line)| {
                    let body = match &item {
                        Some(marker) if i == 0 => Some(marker.body),
                        Some(marker) => marker.continuation(source, line),
                        None => crate::block::quote(source, line),
                    };
                    body.unwrap_or(SourceLine { lazy: true, ..line })
                }),
        )
    }
    /// Logical source fragments excluding surrounding container prefixes.
    pub fn fragments(&self) -> impl Iterator<Item = Span> + '_ {
        self.fragments_in(self.span)
    }
    /// Physical spans contributing to a lexical field, with surrounding container prefixes removed.
    /// An empty source field retains its exact insertion position.
    pub fn fragments_in(&self, span: Span) -> impl Iterator<Item = Span> + '_ {
        span.is_empty().then_some(span).into_iter().chain(
            covering_lines(self.surface_lines, span)
                .iter()
                .filter_map(move |l| l.clipped(span))
                .flat_map(|l| {
                    [l.raw_content(), l.ending]
                        .into_iter()
                        .filter(|s| !s.is_empty())
                }),
        )
    }
    /// Raw physical byte gaps before, between and after immediate children.
    pub fn gaps(&self) -> impl Iterator<Item = Span> + '_ {
        let mut previous = self.span.start;
        self.children
            .iter()
            .map(|c| c.span)
            .chain(std::iter::once(Span::new(self.span.end, self.span.end)))
            .filter_map(move |child| {
                let gap = Span::new(previous, child.start);
                previous = child.end;
                (!gap.is_empty()).then_some(gap)
            })
    }
    /// Visit the node and every descendant in source order.
    pub fn walk(&self) -> impl Iterator<Item = &SyntaxNode<'a>> {
        let mut pending = vec![self];
        std::iter::from_fn(move || {
            let next = pending.pop()?;
            pending.extend(next.children.iter().rev());
            Some(next)
        })
    }
}

impl Document<'_> {
    /// Build a lexical view on demand. The formatter's typed AST remains independently usable.
    pub fn syntax(&self) -> SyntaxNode<'_> {
        let mut root = SyntaxNode::new(
            SyntaxKind::Document,
            Span::new(0, self.source.len()),
            &self.lines,
        );
        root.children = self.blocks.iter().map(|b| block(self.source, b)).collect();
        root
    }
}

fn between(source: &str, span: Span, prefix: u8, suffix: u8) -> (Span, bool) {
    let b = span.text(source).as_bytes();
    let paired = b.len() >= 2 && b[0] == prefix && b[b.len() - 1] == suffix;
    (
        if paired {
            Span::new(span.start + 1, span.end - 1)
        } else {
            span
        },
        paired,
    )
}

fn covering_lines(lines: &[SourceLine], span: Span) -> &[SourceLine] {
    let first = lines.partition_point(|line| line.ending.end <= span.start);
    let count = lines[first..].partition_point(|line| line.raw.start < span.end);
    &lines[first..first + count]
}

fn escape(span: Span, lines: &[SourceLine]) -> SyntaxNode<'_> {
    let mut node = SyntaxNode::new(SyntaxKind::Escape, span, lines);
    node.part(SyntaxRole::Marker, Span::new(span.start, span.start + 1));
    node
}

fn link_field(node: &mut SyntaxNode<'_>, source: &str, span: Span, role: SyntaxRole) {
    let b = span.text(source).as_bytes().first().copied();
    let (contents, paired) = match (role, b) {
        (SyntaxRole::Destination, Some(b'<')) => between(source, span, b'<', b'>'),
        (SyntaxRole::Title, Some(b @ (b'"' | b'\''))) => between(source, span, b, b),
        (SyntaxRole::Title, Some(b'(')) => between(source, span, b'(', b')'),
        _ => (span, false),
    };
    if paired {
        node.part(SyntaxRole::Marker, Span::new(span.start, contents.start));
    }
    node.part(role, contents);
    if paired {
        node.part(SyntaxRole::Marker, Span::new(contents.end, span.end));
    }

    let bytes = source.as_bytes();
    for line in covering_lines(node.surface_lines, contents) {
        let Some(field) = line.clipped(contents) else {
            continue;
        };
        let mut offset = field.content.start;
        while offset < field.content.end {
            if bytes[offset] == b'\\'
                && offset + 1 < field.content.end
                && bytes[offset + 1].is_ascii_punctuation()
            {
                node.children.push(escape(
                    Span::new(offset, offset + 2),
                    std::slice::from_ref(line),
                ));
                offset += 2;
            } else if bytes[offset] == b'&'
                && matches!(role, SyntaxRole::Destination | SyntaxRole::Title)
                && let Some(len) = crate::reference::length(&source[offset..field.content.end])
            {
                node.children.push(SyntaxNode::new(
                    SyntaxKind::Entity,
                    Span::new(offset, offset + len),
                    std::slice::from_ref(line),
                ));
                offset += len;
            } else {
                offset += 1;
            }
        }
    }
}
fn delimiters(node: &mut SyntaxNode<'_>, n: usize) {
    let s = node.span;
    if s.len() >= 2 * n {
        node.part(SyntaxRole::Marker, Span::new(s.start, s.start + n));
        node.part(SyntaxRole::Marker, Span::new(s.end - n, s.end));
    }
}
fn inline<'a>(source: &str, lines: &'a [SourceLine], i: &Inline) -> SyntaxNode<'a> {
    use SyntaxKind as K;
    use SyntaxRole as R;
    let lines = covering_lines(lines, i.span);
    let kind = match &i.kind {
        InlineKind::Text => K::Text,
        InlineKind::Escape => return escape(i.span, lines),
        InlineKind::CodeSpan { .. } => K::CodeSpan,
        InlineKind::Emphasis(_) => K::Emphasis,
        InlineKind::Strong(_) => K::Strong,
        InlineKind::Delete(_) => K::Delete,
        InlineKind::Image(l) | InlineKind::Link(l) => K::Link {
            image: matches!(&i.kind, InlineKind::Image(_)),
            reference: match &l.target {
                LinkTarget::Reference { kind, .. } => Some(*kind),
                _ => None,
            },
        },
        InlineKind::Autolink => K::Autolink,
        InlineKind::Html => K::InlineHtml,
        InlineKind::HardBreak => K::HardBreak,
        InlineKind::SoftBreak => K::SoftBreak,
        InlineKind::Raw
        | InlineKind::Extension(InlineExtension::WikiLink | InlineExtension::Liquid) => K::Raw,
        InlineKind::Extension(InlineExtension::Entity) => K::Entity,
        InlineKind::Extension(InlineExtension::Footnote { .. }) => K::FootnoteReference,
        InlineKind::Extension(InlineExtension::Math { opening, .. }) => K::Math {
            display: opening.len() == 2,
        },
        InlineKind::Recovery(why) => K::Recovery(*why),
    };
    let mut node = SyntaxNode::new(kind, i.span, lines);
    match &i.kind {
        InlineKind::CodeSpan {
            opening, closing, ..
        }
        | InlineKind::Extension(InlineExtension::Math { opening, closing }) => {
            node.part(R::Marker, *opening);
            node.part(R::Marker, *closing);
        }
        InlineKind::Emphasis(children)
        | InlineKind::Strong(children)
        | InlineKind::Delete(children) => {
            let width = match kind {
                K::Strong => 2,
                K::Delete if i.span.text(source).starts_with("~~") => 2,
                _ => 1,
            };
            delimiters(&mut node, width);
            node.children = children.iter().map(|c| inline(source, lines, c)).collect();
        }
        InlineKind::Link(link) | InlineKind::Image(link) => {
            if matches!(i.kind, InlineKind::Image(_)) {
                node.part(R::Marker, Span::new(i.span.start, i.span.start + 1));
            }
            for marker in link.brackets {
                node.part(R::Marker, marker);
            }
            if let Some(markers) = link.target_delimiters {
                for marker in markers {
                    node.part(R::Marker, marker);
                }
            }
            node.part(R::Label, link.label);
            node.children = link
                .children
                .iter()
                .map(|c| inline(source, lines, c))
                .collect();
            match &link.target {
                LinkTarget::Reference { label, .. } => {
                    if let Some(label) = label {
                        link_field(&mut node, source, *label, R::Reference);
                    }
                }
                LinkTarget::Inline { destination, title } => {
                    link_field(&mut node, source, *destination, R::Destination);
                    if let Some(title) = title {
                        link_field(&mut node, source, *title, R::Title);
                    }
                }
            }
        }
        InlineKind::Extension(InlineExtension::Liquid) => delimiters(&mut node, 2),
        InlineKind::Extension(InlineExtension::WikiLink) => {
            let contents = Span::new(i.span.start + 2, i.span.end - 2);
            node.part(R::Marker, Span::new(i.span.start, contents.start));
            node.part(R::Label, contents);
            node.part(R::Marker, Span::new(contents.end, i.span.end));
        }
        InlineKind::Extension(InlineExtension::Footnote { label }) => {
            node.part(R::Marker, Span::new(i.span.start, label.start));
            node.part(R::FootnoteLabel, *label);
            node.part(R::Marker, Span::new(label.end, i.span.end));
        }
        InlineKind::Autolink => {
            let (destination, paired) = between(source, i.span, b'<', b'>');
            if paired {
                delimiters(&mut node, 1);
            }
            node.part(R::Destination, destination);
        }
        _ => {}
    }
    node.parts.sort_by_key(|p| (p.span.start, p.span.end));
    node
}

fn block<'a>(source: &str, b: &'a Block) -> SyntaxNode<'a> {
    use SyntaxKind as K;
    use SyntaxRole as R;
    let kind = match &b.kind {
        BlockKind::Blank => K::Blank,
        BlockKind::Paragraph(_) => K::Paragraph,
        BlockKind::Heading(h) => K::Heading {
            level: h.level,
            setext: h.opening.is_none(),
        },
        BlockKind::BlockQuote(_) => K::Quote,
        BlockKind::ThematicBreak => K::ThematicBreak,
        BlockKind::List(l) => K::List { ordered: l.ordered },
        BlockKind::Fence(_) => K::Code { fenced: true },
        BlockKind::IndentedCode(_) => K::Code { fenced: false },
        BlockKind::Definition(_) => K::LinkDefinition,
        BlockKind::Table(_) => K::Table,
        BlockKind::FrontMatter(_) => K::FrontMatter,
        BlockKind::Html => K::Html,
        BlockKind::Ignore(_) => K::Ignore,
        BlockKind::Raw => K::Raw,
        BlockKind::Recovery(why) => K::Recovery(*why),
        BlockKind::Extension(e) => match e {
            BlockExtension::Alert { .. } => K::Alert,
            BlockExtension::Footnote { .. } => K::FootnoteDefinition,
            BlockExtension::DescriptionList { .. } => K::DescriptionList,
            BlockExtension::Math { .. } => K::Math { display: true },
            BlockExtension::Liquid { .. } => K::Raw,
        },
    };
    let mut node = SyntaxNode::new(kind, b.span, &b.lines);
    match &b.kind {
        BlockKind::Paragraph(inlines) => {
            node.children = inlines
                .iter()
                .map(|i| inline(source, &b.lines, i))
                .collect()
        }
        BlockKind::Heading(h) => {
            node.optional(R::Marker, h.opening);
            node.optional(R::Marker, h.closing);
            node.children = h
                .children
                .iter()
                .map(|i| inline(source, &b.lines, i))
                .collect();
        }
        BlockKind::BlockQuote(blocks)
        | BlockKind::Extension(BlockExtension::Alert { blocks, .. }) => {
            for line in &b.lines {
                let raw = line.content.text(source);
                let content = raw.trim_start_matches([' ', '\t']);
                if content.starts_with('>') && !line.lazy {
                    let start = line.content.end - content.len();
                    node.part(R::Marker, Span::new(start, start + 1));
                }
            }
            if let BlockKind::Extension(BlockExtension::Alert { label, .. }) = &b.kind {
                node.part(R::Marker, Span::new(label.start - 2, label.start));
                node.part(R::AlertLabel, *label);
                node.part(R::Marker, Span::new(label.end, label.end + 1));
            }
            node.children = blocks.iter().map(|b| block(source, b)).collect();
        }
        BlockKind::ThematicBreak => node.part(R::Marker, b.span),
        BlockKind::List(list) => {
            for item in &list.items {
                let first = b.lines.partition_point(|l| l.ending.end <= item.span.start);
                let n = b.lines[first..].partition_point(|l| l.raw.start < item.span.end);
                let mut child = SyntaxNode::new(K::ListItem, item.span, &b.lines[first..first + n]);
                child.part(R::Marker, item.marker);
                if let Some(task) = item.task {
                    child.part(
                        R::Task {
                            checked: task.checked,
                        },
                        task.marker,
                    );
                }
                child.children = item.blocks.iter().map(|b| block(source, b)).collect();
                node.children.push(child);
            }
        }
        BlockKind::Fence(f) => {
            node.lines = &f.code.lines;
            node.part(R::Marker, f.opening);
            node.optional(R::Info, f.info);
            node.optional(R::Language, f.lang);
            node.optional(R::Metadata, f.meta);
            node.optional(R::Marker, f.closing);
        }
        BlockKind::IndentedCode(code) => node.lines = &code.lines,
        BlockKind::Definition(d) => {
            for marker in d.brackets {
                node.part(R::Marker, marker);
            }
            link_field(&mut node, source, d.label, R::Label);
            node.part(R::Separator, d.separator);
            link_field(&mut node, source, d.destination, R::Destination);
            if let Some(title) = d.title {
                link_field(&mut node, source, title, R::Title);
            }
        }
        BlockKind::FrontMatter(f) => {
            node.part(R::Marker, f.opening);
            node.part(R::Marker, f.closing);
        }
        BlockKind::Table(t) => {
            node.part(R::Separator, t.delimiter);
            node.children
                .push(row(source, &b.lines[..1], &t.header, true));
            for (line, cells) in b.lines[2..].iter().zip(&t.rows) {
                node.children
                    .push(row(source, std::slice::from_ref(line), cells, false));
            }
        }
        BlockKind::Extension(BlockExtension::Liquid { opening, closing }) => {
            node.part(R::Marker, *opening);
            node.part(R::Marker, *closing);
        }
        BlockKind::Extension(BlockExtension::Footnote { label, blocks }) => {
            node.part(R::Marker, Span::new(label.start - 2, label.start));
            node.part(R::FootnoteLabel, *label);
            node.part(R::Marker, Span::new(label.end, label.end + 2));
            node.children = blocks.iter().map(|b| block(source, b)).collect();
        }
        BlockKind::Extension(BlockExtension::Math {
            opening,
            closing,
            meta,
            ..
        }) => {
            node.part(R::Marker, *opening);
            node.optional(R::Metadata, *meta);
            node.optional(R::Marker, *closing);
        }
        BlockKind::Extension(BlockExtension::DescriptionList { entries }) => {
            for entry in entries {
                for term in &entry.terms {
                    let mut term = block(source, term);
                    term.kind = K::Term;
                    node.children.push(term);
                }
                for d in &entry.descriptions {
                    let mut desc =
                        SyntaxNode::new(K::Description, d.span, covering_lines(&b.lines, d.span));
                    desc.part(R::Marker, d.marker);
                    desc.children = d.blocks.iter().map(|b| block(source, b)).collect();
                    node.children.push(desc);
                }
            }
        }
        _ => {}
    }
    node.parts.sort_by_key(|p| (p.span.start, p.span.end));
    node
}

fn row<'a>(
    source: &str,
    lines: &'a [SourceLine],
    cells: &[TableCell],
    header: bool,
) -> SyntaxNode<'a> {
    let mut row = SyntaxNode::new(SyntaxKind::Row { header }, lines[0].content, lines);
    let bytes = row.span.text(source).as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 2,
            b'|' => {
                let p = row.span.start + i;
                row.part(SyntaxRole::Marker, Span::new(p, p + 1));
                i += 1;
            }
            _ => i += 1,
        }
    }
    for cell in cells {
        let mut child = SyntaxNode::new(SyntaxKind::Cell, cell.span, lines);
        child.children = cell
            .children
            .iter()
            .map(|i| inline(source, lines, i))
            .collect();
        row.children.push(child);
    }
    row
}
