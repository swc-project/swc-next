//! Contiguous sibling groups and their variable-size payloads belong to one document.

use std::ops::Range;

use crate::{DefinitionEntry, Document, Inline, Link};

/// Ordered siblings in the document that parsed them. Resolve with [`Document::inlines`].
#[derive(Debug, Clone, Copy, Default)]
pub struct InlineNodes {
    start: usize,
    end: usize,
}

/// Link or image payload in its originating document. Resolve with [`Document::link`].
#[derive(Debug, Clone, Copy)]
pub struct LinkId(usize);

/// Parsed inline value in its originating document. Resolve with [`Document::inline_text`].
#[derive(Debug, Clone, Copy)]
pub struct InlineText(usize);

#[derive(Debug, Default)]
pub(crate) struct InlineStore {
    nodes: smallvec::SmallVec<[Inline; 1]>,
    links: Vec<Link>,
    text: String,
    values: Vec<Range<usize>>,
}

#[derive(Clone, Copy)]
pub(super) struct Checkpoint {
    nodes: usize,
    links: usize,
    values: usize,
}

impl InlineStore {
    pub(super) fn append(&mut self, nodes: &[Inline]) -> InlineNodes {
        let start = self.nodes.len();
        if let [node] = nodes {
            self.nodes.push(*node);
        } else {
            self.nodes.extend_from_slice(nodes);
        }
        InlineNodes {
            start,
            end: self.nodes.len(),
        }
    }

    pub(super) fn push_link(&mut self, link: Link) -> LinkId {
        let id = LinkId(self.links.len());
        self.links.push(link);
        id
    }

    pub(super) fn push_text(&mut self, value: &str) -> InlineText {
        let start = self.text.len();
        self.text.push_str(value);
        self.finish_value(start)
    }

    pub(super) fn push_identifier(&mut self, label: &str) -> InlineText {
        let start = self.text.len();
        crate::append_identifier(&mut self.text, label);
        self.finish_value(start)
    }

    fn finish_value(&mut self, start: usize) -> InlineText {
        let id = InlineText(self.values.len());
        self.values.push(start..self.text.len());
        id
    }

    pub(super) fn nodes(&self, range: InlineNodes) -> &[Inline] {
        &self.nodes[range.start..range.end]
    }

    pub(super) fn link(&self, id: LinkId) -> &Link {
        &self.links[id.0]
    }

    pub(super) fn text(&self, id: InlineText) -> &str {
        &self.text[self.values[id.0].clone()]
    }

    pub(super) fn checkpoint(&self) -> Checkpoint {
        Checkpoint {
            nodes: self.nodes.len(),
            links: self.links.len(),
            values: self.values.len(),
        }
    }

    // Nested descendants are appended before their parent. Dropping a tentative
    // link label or replacing an over-depth subtree can discard its whole suffix.
    pub(super) fn rewind(&mut self, point: Checkpoint) {
        self.nodes.truncate(point.nodes);
        self.links.truncate(point.links);
        self.values.truncate(point.values);
        self.text
            .truncate(self.values.last().map_or(0, |span| span.end));
    }
}

impl Document<'_> {
    /// Borrow an ordered group of inline children from this document.
    ///
    /// ```
    /// use swc_next_markdown_parser::{parse, BlockKind, InlineKind};
    ///
    /// let doc = parse("*hello*");
    /// let BlockKind::Paragraph(group) = doc.blocks[0].kind else { panic!() };
    /// let InlineKind::Emphasis(children) = doc.inlines(group)[0].kind else { panic!() };
    /// assert_eq!(doc.text(doc.inlines(children)[0].span), "hello");
    /// ```
    pub fn inlines(&self, group: InlineNodes) -> &[Inline] {
        self.inline_store.nodes(group)
    }

    /// Borrow the payload of a link or image from this document.
    pub fn link(&self, id: LinkId) -> &Link {
        self.inline_store.link(id)
    }

    /// Read the normalized code span value or reference identifier from this document.
    pub fn inline_text(&self, value: InlineText) -> &str {
        self.inline_store.text(value)
    }

    /// Resolve the normalized identifier stored on a reference link or image.
    pub fn reference_definition(&self, identifier: InlineText) -> Option<&DefinitionEntry> {
        self.first_definitions
            .get(self.inline_text(identifier))
            .map(|&index| &self.definitions[index])
    }
}
