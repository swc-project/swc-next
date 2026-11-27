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

mod cursor;
pub use cursor::{SyntaxCursor, SyntaxView};

mod fields;

fn container_lines<'a>(
    kind: SyntaxKind,
    lines: &'a [SourceLine],
    source: &'a str,
) -> Option<impl Iterator<Item = SourceLine> + 'a> {
    let item = match kind {
        SyntaxKind::ListItem => Some(crate::block::list_marker(source, *lines.first()?)?),
        SyntaxKind::Quote | SyntaxKind::Alert => None,
        _ => return None,
    };
    Some(lines.iter().copied().enumerate().map(move |(i, line)| {
        let body = match &item {
            Some(marker) if i == 0 => Some(marker.body),
            Some(marker) => marker.continuation(source, line),
            None => crate::block::quote(source, line),
        };
        body.unwrap_or(SourceLine { lazy: true, ..line })
    }))
}

fn fragments_in(surface_lines: &[SourceLine], span: Span) -> impl Iterator<Item = Span> + '_ {
    span.is_empty().then_some(span).into_iter().chain(
        covering_lines(surface_lines, span)
            .iter()
            .filter_map(move |l| l.clipped(span))
            .flat_map(|l| {
                [l.raw_content(), l.ending]
                    .into_iter()
                    .filter(|s| !s.is_empty())
            }),
    )
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
    if let [line] = lines {
        return if line.ending.end > span.start && line.raw.start < span.end {
            lines
        } else {
            &[]
        };
    }
    let first = lines.partition_point(|line| line.ending.end <= span.start);
    let count = lines[first..].partition_point(|line| line.raw.start < span.end);
    &lines[first..first + count]
}
