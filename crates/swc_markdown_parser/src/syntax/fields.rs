//! Lexical kinds, source fields and the children embedded in link fields.
use super::{
    Block, BlockExtension, BlockKind, Inline, InlineExtension, InlineKind, LinkTarget, SourceLine,
    Span, SyntaxKind as K, SyntaxPart, SyntaxRole as R, between, covering_lines,
};
use crate::{Definition, Document, Link, ListItem};

pub(super) fn block_kind(block: &Block) -> K {
    match &block.kind {
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
        BlockKind::Extension(ext) => match ext {
            BlockExtension::Alert { .. } => K::Alert,
            BlockExtension::Footnote { .. } => K::FootnoteDefinition,
            BlockExtension::DescriptionList { .. } => K::DescriptionList,
            BlockExtension::Math { .. } => K::Math { display: true },
            BlockExtension::Liquid { .. } => K::Raw,
        },
    }
}

pub(super) fn inline_kind(document: &Document<'_>, inline: &Inline) -> K {
    match &inline.kind {
        InlineKind::Text => K::Text,
        InlineKind::Escape => K::Escape,
        InlineKind::CodeSpan { .. } => K::CodeSpan,
        InlineKind::Emphasis(_) => K::Emphasis,
        InlineKind::Strong(_) => K::Strong,
        InlineKind::Delete(_) => K::Delete,
        InlineKind::Image(l) | InlineKind::Link(l) => K::Link {
            image: matches!(inline.kind, InlineKind::Image(_)),
            reference: match document.link(*l).target {
                LinkTarget::Reference { kind, .. } => Some(kind),
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
        InlineKind::Extension(InlineExtension::Math { display }) => K::Math { display: *display },
        InlineKind::Recovery(why) => K::Recovery(*why),
    }
}
fn field_contents(source: &str, span: Span, role: R) -> (Span, bool) {
    match (role, span.text(source).as_bytes().first().copied()) {
        (R::Destination, Some(b'<')) => between(source, span, b'<', b'>'),
        (R::Title, Some(b @ (b'"' | b'\''))) => between(source, span, b, b),
        (R::Title, Some(b'(')) => between(source, span, b'(', b')'),
        _ => (span, false),
    }
}

struct Fields<'a>(&'a mut Vec<SyntaxPart>);
impl Fields<'_> {
    fn push(&mut self, role: R, span: Span) {
        self.0.push(SyntaxPart { role, span });
    }
    fn optional(&mut self, role: R, span: Option<Span>) {
        if let Some(s) = span {
            self.push(role, s);
        }
    }
    fn delimiters(&mut self, s: Span, width: usize) {
        if s.len() >= 2 * width {
            self.push(R::Marker, Span::new(s.start, s.start + width));
            self.push(R::Marker, Span::new(s.end - width, s.end));
        }
    }
    fn field(&mut self, source: &str, span: Span, role: R) {
        let (content, paired) = field_contents(source, span, role);
        if paired {
            self.push(R::Marker, Span::new(span.start, content.start));
        }
        self.push(role, content);
        if paired {
            self.push(R::Marker, Span::new(content.end, span.end));
        }
    }
}

pub(super) fn block_parts(source: &str, b: &Block, output: &mut Vec<SyntaxPart>) {
    let mut fields = Fields(output);
    let s = b.span;
    match &b.kind {
        BlockKind::Heading(h) => {
            fields.optional(R::Marker, h.opening);
            fields.optional(R::Marker, h.closing);
        }
        BlockKind::BlockQuote(_) | BlockKind::Extension(BlockExtension::Alert { .. }) => {
            for l in &b.lines {
                let raw = l.content.text(source);
                let content = raw.trim_start_matches([' ', '\t']);
                if content.starts_with('>') && !l.lazy {
                    let start = l.content.end - content.len();
                    fields.push(R::Marker, Span::new(start, start + 1));
                }
            }
            if let BlockKind::Extension(BlockExtension::Alert { label, .. }) = b.kind {
                fields.push(R::Marker, Span::new(label.start - 2, label.start));
                fields.push(R::AlertLabel, label);
                fields.push(R::Marker, Span::new(label.end, label.end + 1));
                fields.0.sort_by_key(|p| (p.span.start, p.span.end));
            }
        }
        BlockKind::ThematicBreak => fields.push(R::Marker, s),
        BlockKind::Fence(f) => {
            fields.push(R::Marker, f.opening);
            fields.optional(R::Info, f.info);
            fields.optional(R::Language, f.lang);
            fields.optional(R::Metadata, f.meta);
            fields.optional(R::Marker, f.closing);
            if f.meta.is_some() {
                fields.0.sort_by_key(|p| (p.span.start, p.span.end));
            }
        }
        BlockKind::Definition(d) => {
            fields.push(R::Marker, d.brackets[0]);
            fields.push(R::Label, d.label);
            fields.push(R::Marker, d.brackets[1]);
            fields.push(R::Separator, d.separator);
            fields.field(source, d.destination, R::Destination);
            if let Some(t) = d.title {
                fields.field(source, t, R::Title);
            }
        }
        BlockKind::FrontMatter(f) => {
            fields.push(R::Marker, f.opening);
            fields.push(R::Marker, f.closing);
        }
        BlockKind::Table(t) => fields.push(R::Separator, t.delimiter),
        BlockKind::Extension(BlockExtension::Liquid { opening, closing }) => {
            fields.push(R::Marker, *opening);
            fields.push(R::Marker, *closing);
        }
        BlockKind::Extension(BlockExtension::Footnote { label, .. }) => {
            fields.push(R::Marker, Span::new(label.start - 2, label.start));
            fields.push(R::FootnoteLabel, *label);
            fields.push(R::Marker, Span::new(label.end, label.end + 2));
        }
        BlockKind::Extension(BlockExtension::Math {
            opening,
            closing,
            meta,
            ..
        }) => {
            fields.push(R::Marker, *opening);
            fields.optional(R::Metadata, *meta);
            fields.optional(R::Marker, *closing);
        }
        _ => {}
    }
}

pub(super) fn inline_parts(
    document: &Document<'_>,
    i: &Inline,
    kind: K,
    output: &mut Vec<SyntaxPart>,
) {
    let mut fields = Fields(output);
    let source = document.source;
    let s = i.span;
    match &i.kind {
        InlineKind::Escape => fields.0.push(escape_marker(s)),
        InlineKind::CodeSpan { width, .. } => fields.delimiters(s, *width),
        InlineKind::Extension(InlineExtension::Math { display }) => {
            fields.delimiters(s, 1 + usize::from(*display));
        }
        InlineKind::Emphasis(_) | InlineKind::Strong(_) | InlineKind::Delete(_) => {
            let width = match kind {
                K::Strong => 2,
                K::Delete if s.text(source).starts_with("~~") => 2,
                _ => 1,
            };
            fields.delimiters(s, width);
        }
        InlineKind::Link(id) | InlineKind::Image(id) => {
            let l = document.link(*id);
            if matches!(i.kind, InlineKind::Image(_)) {
                fields.push(R::Marker, Span::new(s.start, s.start + 1));
            }
            fields.push(R::Marker, l.brackets[0]);
            fields.push(R::Label, l.label);
            fields.push(R::Marker, l.brackets[1]);
            fields.optional(R::Marker, l.target_delimiters.map(|ms| ms[0]));
            match &l.target {
                LinkTarget::Reference { label: Some(s), .. } => {
                    fields.field(source, *s, R::Reference)
                }
                LinkTarget::Inline { destination, title } => {
                    fields.field(source, *destination, R::Destination);
                    if let Some(t) = title {
                        fields.field(source, *t, R::Title);
                    }
                }
                _ => {}
            }
            fields.optional(R::Marker, l.target_delimiters.map(|ms| ms[1]));
        }
        InlineKind::Extension(InlineExtension::Liquid) => fields.delimiters(s, 2),
        InlineKind::Extension(InlineExtension::WikiLink) => {
            let content = Span::new(s.start + 2, s.end - 2);
            fields.push(R::Marker, Span::new(s.start, content.start));
            fields.push(R::Label, content);
            fields.push(R::Marker, Span::new(content.end, s.end));
        }
        InlineKind::Extension(InlineExtension::Footnote { label }) => {
            fields.push(R::Marker, Span::new(s.start, label.start));
            fields.push(R::FootnoteLabel, *label);
            fields.push(R::Marker, Span::new(label.end, s.end));
        }
        InlineKind::Autolink => {
            fields.field(source, s, R::Destination);
        }
        _ => {}
    }
}

pub(super) fn item_parts(i: &ListItem, output: &mut Vec<SyntaxPart>) {
    let mut fields = Fields(output);

    fields.push(R::Marker, i.marker);
    if let Some(t) = i.task {
        fields.push(R::Task { checked: t.checked }, t.marker);
    }
}

pub(super) fn row_parts(source: &str, s: Span, output: &mut Vec<SyntaxPart>) {
    let mut fields = Fields(output);

    let bytes = s.text(source).as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 2,
            b'|' => {
                fields.push(R::Marker, Span::new(s.start + i, s.start + i + 1));
                i += 1;
            }
            _ => i += 1,
        }
    }
}

fn field_leaves<'a>(
    source: &str,
    lines: &'a [SourceLine],
    span: Span,
    role: R,
    mut visit: impl FnMut(K, Span, &'a [SourceLine]),
) {
    let (contents, _) = field_contents(source, span, role);
    let bytes = source.as_bytes();
    let entities = matches!(role, R::Destination | R::Title);
    for line in covering_lines(lines, contents) {
        let Some(field) = line.clipped(contents) else {
            continue;
        };
        let mut pos = field.content.start;
        while let Some(offset) = if entities {
            memchr::memchr2(b'\\', b'&', &bytes[pos..field.content.end])
        } else {
            memchr::memchr(b'\\', &bytes[pos..field.content.end])
        } {
            pos += offset;
            let (kind, width) = if bytes[pos] == b'\\'
                && pos + 1 < field.content.end
                && bytes[pos + 1].is_ascii_punctuation()
            {
                (K::Escape, 2)
            } else if bytes[pos] == b'&'
                && let Some(len) = crate::reference::length(&source[pos..field.content.end])
            {
                (K::Entity, len)
            } else {
                pos += 1;
                continue;
            };
            visit(
                kind,
                Span::new(pos, pos + width),
                std::slice::from_ref(line),
            );
            pos += width;
        }
    }
}

pub(super) fn link_leaves<'a>(
    source: &str,
    lines: &'a [SourceLine],
    link: &Link,
    mut visit: impl FnMut(K, Span, &'a [SourceLine]),
) {
    match &link.target {
        LinkTarget::Reference {
            label: Some(span), ..
        } => field_leaves(source, lines, *span, R::Reference, visit),
        LinkTarget::Inline { destination, title } => {
            field_leaves(source, lines, *destination, R::Destination, &mut visit);
            if let Some(span) = title {
                field_leaves(source, lines, *span, R::Title, visit);
            }
        }
        _ => {}
    }
}
pub(super) fn definition_leaves<'a>(
    source: &str,
    lines: &'a [SourceLine],
    d: &Definition,
    mut visit: impl FnMut(K, Span, &'a [SourceLine]),
) {
    field_leaves(source, lines, d.label, R::Label, &mut visit);
    field_leaves(source, lines, d.destination, R::Destination, &mut visit);
    if let Some(span) = d.title {
        field_leaves(source, lines, span, R::Title, visit);
    }
}

pub(super) fn escape_marker(span: Span) -> SyntaxPart {
    SyntaxPart {
        role: R::Marker,
        span: Span::new(span.start, span.start + 1),
    }
}
