//! Borrow the typed AST while visiting lexical nodes in source order.

use super::fields::{
    block_kind, block_parts, definition_leaves, escape_marker, inline_kind, inline_parts,
    item_parts, link_leaves, row_parts,
};
use super::{
    Block, BlockExtension, BlockKind, Document, Inline, InlineKind, SourceLine, Span,
    SyntaxKind as K, SyntaxPart, SyntaxRole as R, TableCell, container_lines, covering_lines,
    fragments_in,
};
use crate::{Description, ListItem};
use std::{fmt, ops::Deref};

#[derive(Clone, Copy)]
enum Origin<'a> {
    Document(&'a [Block]),
    Block(&'a Block),
    Inline(&'a Inline),
    Item(&'a ListItem),
    Row(&'a [TableCell]),
    Cell(&'a TableCell),
    Description(&'a Description),
    FieldLeaf,
}

/// A position in the lexical view of a parsed document.
/// Children and fields are derived from the borrowed AST as they are requested.
#[derive(Clone, Copy)]
pub struct SyntaxCursor<'a> {
    pub kind: K,
    pub span: Span,
    lines: &'a [SourceLine],
    source: &'a str,
    origin: Origin<'a>,
}

impl fmt::Debug for SyntaxCursor<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SyntaxCursor")
            .field("kind", &self.kind)
            .field("span", &self.span)
            .finish_non_exhaustive()
    }
}

/// Fields of one visited node, valid until the caller reuses its scratch buffer.
#[derive(Debug)]
pub struct SyntaxView<'a, 'fields> {
    cursor: SyntaxCursor<'a>,
    pub parts: &'fields [SyntaxPart],
}

impl<'a> Deref for SyntaxView<'a, '_> {
    type Target = SyntaxCursor<'a>;
    fn deref(&self) -> &Self::Target {
        &self.cursor
    }
}

impl Document<'_> {
    /// Borrow the root of the lexical tree. Children and fields are computed on demand.
    pub fn syntax(&self) -> SyntaxCursor<'_> {
        SyntaxCursor::new(
            self.source,
            Origin::Document(&self.blocks),
            K::Document,
            Span::new(0, self.source.len()),
            &self.lines,
        )
    }
}

impl<'a> SyntaxCursor<'a> {
    #[inline]
    fn new(
        source: &'a str,
        origin: Origin<'a>,
        kind: K,
        span: Span,
        lines: &'a [SourceLine],
    ) -> Self {
        Self {
            source,
            origin,
            kind,
            span,
            lines,
        }
    }

    fn block(source: &'a str, block: &'a Block) -> Self {
        let kind = block_kind(block);
        let mut cursor = Self::new(source, Origin::Block(block), kind, block.span, &block.lines);
        match &block.kind {
            BlockKind::Fence(fence) => cursor.lines = &fence.code.lines,
            BlockKind::IndentedCode(code) => cursor.lines = &code.lines,
            _ => {}
        }
        cursor
    }

    fn inline(source: &'a str, lines: &'a [SourceLine], inline: &'a Inline) -> Self {
        let kind = inline_kind(inline);
        Self::new(source, Origin::Inline(inline), kind, inline.span, lines)
    }

    /// Logical lines covered by the node. Code nodes expose the guest body's lines.
    pub fn lines(&self) -> &'a [SourceLine] {
        if matches!(self.origin, Origin::Inline(_)) {
            covering_lines(self.lines, self.span)
        } else {
            self.lines
        }
    }

    /// Lines inside a quote, alert or list item after its own prefix is removed.
    /// Blank lines, fence delimiters and deeper container prefixes are preserved.
    pub fn container_lines(&self) -> Option<impl Iterator<Item = SourceLine> + 'a> {
        container_lines(self.kind, self.lines, self.source)
    }
    /// Source fragments covered by this node, excluding enclosing container prefixes.
    pub fn fragments(&self) -> impl Iterator<Item = Span> + 'a {
        self.fragments_in(self.span)
    }
    /// Physical fragments of a byte range with enclosing container prefixes removed.
    /// For fenced code, the range may include the opening and closing fences.
    /// Use [`Self::lines`] to access just the code body's logical lines.
    /// An empty range retains its insertion offset.
    pub fn fragments_in(&self, span: Span) -> impl Iterator<Item = Span> + 'a {
        let lines = if span.start >= self.span.start && span.end <= self.span.end {
            self.enclosing_lines()
        } else {
            self.surface_lines()
        };
        fragments_in(lines, span)
    }

    fn surface_lines(&self) -> &'a [SourceLine] {
        if matches!(self.origin, Origin::Inline(_)) {
            self.lines()
        } else {
            self.enclosing_lines()
        }
    }

    fn enclosing_lines(&self) -> &'a [SourceLine] {
        match (self.kind, self.origin) {
            (K::Code { .. }, Origin::Block(block)) => &block.lines,
            _ => self.lines,
        }
    }

    /// Range from the first to the last inline in a nonempty heading, excluding its markers.
    pub fn heading_content(&self) -> Option<Span> {
        let Origin::Block(Block {
            kind: BlockKind::Heading(h),
            ..
        }) = self.origin
        else {
            return None;
        };
        Some(Span::new(
            h.children.first()?.span.start,
            h.children.last()?.span.end,
        ))
    }

    /// Return the node's fields in source order using a caller-owned reusable buffer.
    pub fn read<'fields>(&self, scratch: &'fields mut Vec<SyntaxPart>) -> SyntaxView<'a, 'fields> {
        scratch.clear();
        self.write_parts(scratch);
        SyntaxView {
            cursor: *self,
            parts: scratch,
        }
    }

    /// Visit this node and its descendants in source order.
    pub fn walk(&self, visit: &mut impl FnMut(Self)) {
        visit(*self);
        if !self.is_plain_leaf() {
            self.for_each_child(|child| child.walk(visit));
        }
    }

    /// Visit the gaps before, between and after immediate children in source order.
    /// The gaps and child spans partition the node's original byte range.
    pub fn for_each_gap(&self, mut visit: impl FnMut(Span)) {
        let mut previous = self.span.start;
        self.for_each_child(|child| {
            if previous < child.span.start {
                visit(Span::new(previous, child.span.start));
            }
            previous = previous.max(child.span.end);
        });
        if previous < self.span.end {
            visit(Span::new(previous, self.span.end));
        }
    }

    /// Visit the immediate lexical children in source order.
    pub fn for_each_child<F: FnMut(Self)>(&self, mut visit: F) {
        let visit = &mut visit;
        let source = self.source;
        let lines = self.enclosing_lines();
        let blocks = |children: &'a [Block], emit: &mut F| {
            for child in children {
                emit(Self::block(source, child));
            }
        };
        let inlines = |children: &'a [Inline], emit: &mut F| {
            for child in children {
                emit(Self::inline(source, lines, child));
            }
        };
        match self.origin {
            Origin::Document(children) => blocks(children, visit),
            Origin::Block(b) => match &b.kind {
                BlockKind::Paragraph(v) => inlines(v, visit),
                BlockKind::Heading(h) => inlines(&h.children, visit),
                BlockKind::BlockQuote(v)
                | BlockKind::Extension(
                    BlockExtension::Alert { blocks: v, .. }
                    | BlockExtension::Footnote { blocks: v, .. },
                ) => blocks(v, visit),
                BlockKind::List(list) => {
                    for item in &list.items {
                        visit(Self::new(
                            source,
                            Origin::Item(item),
                            K::ListItem,
                            item.span,
                            covering_lines(lines, item.span),
                        ));
                    }
                }
                BlockKind::Definition(d) => {
                    definition_leaves(source, lines, d, |kind, span, row| {
                        visit(Self::new(source, Origin::FieldLeaf, kind, span, row));
                    })
                }
                BlockKind::Table(t) => {
                    let header = &lines[..1];
                    visit(Self::new(
                        source,
                        Origin::Row(&t.header),
                        K::Row { header: true },
                        header[0].content,
                        header,
                    ));
                    for (line, cells) in lines[2..].iter().zip(&t.rows) {
                        visit(Self::new(
                            source,
                            Origin::Row(cells),
                            K::Row { header: false },
                            line.content,
                            std::slice::from_ref(line),
                        ));
                    }
                }
                BlockKind::Extension(BlockExtension::DescriptionList { entries }) => {
                    for e in entries {
                        for term in &e.terms {
                            let mut item = Self::block(source, term);
                            item.kind = K::Term;
                            visit(item);
                        }
                        for d in &e.descriptions {
                            visit(Self::new(
                                source,
                                Origin::Description(d),
                                K::Description,
                                d.span,
                                covering_lines(lines, d.span),
                            ));
                        }
                    }
                }
                _ => {}
            },
            Origin::Inline(i) => match &i.kind {
                InlineKind::Emphasis(v) | InlineKind::Strong(v) | InlineKind::Delete(v) => {
                    inlines(v, visit)
                }
                InlineKind::Link(l) | InlineKind::Image(l) => {
                    inlines(&l.children, visit);
                    link_leaves(source, lines, l, |kind, span, row| {
                        visit(Self::new(source, Origin::FieldLeaf, kind, span, row));
                    });
                }
                _ => {}
            },
            Origin::Row(cells) => {
                for c in cells {
                    visit(Self::new(source, Origin::Cell(c), K::Cell, c.span, lines));
                }
            }
            Origin::Cell(c) => inlines(&c.children, visit),
            Origin::Item(i) => blocks(&i.blocks, visit),
            Origin::Description(d) => blocks(&d.blocks, visit),
            Origin::FieldLeaf => {}
        }
    }

    fn is_plain_leaf(&self) -> bool {
        matches!(
            self.kind,
            K::Text | K::Blank | K::SoftBreak | K::HardBreak | K::Entity | K::InlineHtml
        )
    }

    fn write_parts(&self, output: &mut Vec<SyntaxPart>) {
        match self.origin {
            Origin::Block(block) => block_parts(self.source, block, output),
            Origin::Inline(inline) => inline_parts(self.source, inline, self.kind, output),
            Origin::Item(item) => item_parts(item, output),
            Origin::Row(_) => row_parts(self.source, self.span, output),
            Origin::Description(d) => output.push(SyntaxPart {
                role: R::Marker,
                span: d.marker,
            }),
            Origin::FieldLeaf if self.kind == K::Escape => output.push(escape_marker(self.span)),
            _ => {}
        }
    }
}
