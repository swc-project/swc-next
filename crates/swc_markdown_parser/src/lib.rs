//! Parse Markdown into ordered blocks, inline nodes, and ranges into the original UTF-8 source.

mod block;
mod inline;
mod inline_store;
pub use inline_store::{InlineNodes, InlineText, LinkId};
mod reference;
mod source;
pub use block::{
    can_continue_link_definition, is_front_matter_closing_line, is_table_delimiter_row,
    liquid_block_delimiter, starts_display_math, starts_footnote_definition,
};
pub use inline::{delimiter_flanking, emphasis_runs_match};
pub use reference::character_reference;
mod syntax;
pub use syntax::{SyntaxCursor, SyntaxKind, SyntaxPart, SyntaxRole, SyntaxView};

use std::{collections::HashMap, sync::Arc};

pub use source::{SourceLine, SourceLines, Span};

/// Parse a complete document. Malformed and unsupported constructs retain their source text.
pub fn parse(source: &str) -> Document<'_> {
    parse_with_options(source, ParseOptions::default())
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Dialect {
    #[default]
    Markdown,
    /// Includes alerts, description lists, footnote references, inline math and single-tilde strike.
    Extended,
}

#[derive(Clone, Copy, Debug)]
pub struct ParseOptions {
    pub dialect: Dialect,
    /// Maximum nested block or inline levels, up to the library's stack-safe ceiling.
    pub max_nesting: usize,
    /// Limit lookahead per inline paragraph independently of its input-dependent default.
    pub max_inline_steps: Option<usize>,
    /// Return one recoverable source region for documents exceeding this size in bytes.
    pub max_input_bytes: Option<usize>,
}
impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            dialect: Dialect::Markdown,
            max_nesting: MAX_DEPTH,
            max_inline_steps: None,
            max_input_bytes: None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecoveryReason {
    Nesting,
    InlineWork,
    InputSize,
}

pub fn parse_with_options(source: &str, mut options: ParseOptions) -> Document<'_> {
    options.max_nesting = options.max_nesting.min(MAX_DEPTH);
    let lines = SourceLines::from(source::lines(source));
    let mut inline_store = inline_store::InlineStore::default();
    let (mut blocks, definitions, first_definitions, needs_lookup) = {
        let mut parser = block::Parser::new(source, options, &mut inline_store);
        let blocks = if options
            .max_input_bytes
            .is_some_and(|limit| source.len() > limit)
        {
            vec![Block {
                span: Span::new(0, source.len()),
                lines: lines.clone(),
                kind: BlockKind::Recovery(RecoveryReason::InputSize),
            }]
        } else {
            parser.blocks(lines.clone(), 0, true)
        };
        (
            blocks,
            parser.definitions,
            parser.first_definitions,
            parser.needs_reference_lookup.get(),
        )
    };
    if needs_lookup {
        // Forward definitions can change the winner between a link and an emphasis opener.
        drop(blocks);
        inline_store = inline_store::InlineStore::default();
        blocks = block::Parser::new(source, options, &mut inline_store)
            .with_reference_definitions(&first_definitions)
            .blocks(lines.clone(), 0, true);
    }
    Document {
        source,
        dialect: options.dialect,
        lines,
        blocks,
        definitions,
        first_definitions,
        inline_store,
    }
}

#[derive(Debug)]
pub struct Document<'a> {
    pub source: &'a str,
    pub lines: SourceLines,
    pub blocks: Vec<Block>,
    inline_store: inline_store::InlineStore,
    /// Every occurrence in document order, including duplicate definitions.
    pub definitions: Vec<DefinitionEntry>,
    first_definitions: HashMap<String, usize>,
    dialect: Dialect,
}

impl<'a> Document<'a> {
    pub fn source(&self) -> &'a str {
        self.source
    }
    /// The dialect used to parse this document.
    pub fn dialect(&self) -> Dialect {
        self.dialect
    }
    pub fn text(&self, span: Span) -> &'a str {
        &self.source[span.start..span.end]
    }
    /// Resolve a reference using its label or its normalized identifier. The first definition wins.
    pub fn definition(&self, label: &str) -> Option<&DefinitionEntry> {
        self.first_definitions
            .get(&identifier(label))
            .map(|&index| &self.definitions[index])
    }
}

fn identifier(label: &str) -> String {
    let mut normalized = String::with_capacity(label.len());
    append_identifier(&mut normalized, label);
    normalized
}

fn append_identifier(normalized: &mut String, label: &str) {
    let start = normalized.len();
    if label.bytes().all(|byte| matches!(byte, b'!'..=b'~')) {
        normalized.push_str(label);
        normalized[start..].make_ascii_lowercase();
        return;
    }
    for word in label.split_whitespace() {
        if normalized.len() != start {
            normalized.push(' ');
        }
        normalized.push_str(word);
    }
    if normalized[start..].is_ascii() {
        normalized[start..].make_ascii_lowercase();
    } else {
        // TODO: Use `str::to_casefold_unnormalized` once stable (rust-lang/rust#157000).
        let folded = unicase::UniCase::unicode(&normalized[start..]).to_folded_case();
        normalized.truncate(start);
        normalized.push_str(&folded);
    }
}

#[derive(Debug)]
pub struct Block {
    pub span: Span,
    /// Source lines in order, with enclosing list and quote prefixes removed.
    pub lines: SourceLines,
    pub kind: BlockKind,
}

impl Block {
    /// Clip the leaf's logical source lines to a physical byte range.
    pub fn fragments(&self, span: Span) -> impl Iterator<Item = SourceLine> + '_ {
        let start = self
            .lines
            .partition_point(|line| line.ending.end <= span.start);
        self.lines[start..]
            .iter()
            .take_while(move |line| line.raw.start < span.end)
            .filter_map(move |line| line.clipped(span))
    }
    /// Read logical contents across physical lines, with line endings normalized to LF.
    pub fn logical_text(&self, source: &str, span: Span) -> String {
        let mut text = String::new();
        for line in self.fragments(span) {
            line.append(source, &mut text);
            if !line.ending.is_empty() {
                text.push('\n');
            }
        }
        text
    }
}

#[derive(Debug)]
pub enum BlockKind {
    Paragraph(InlineNodes),
    Heading(Heading),
    ThematicBreak,
    BlockQuote(Vec<Block>),
    List(List),
    Fence(Box<Fence>),
    IndentedCode(Box<Code>),
    Definition(Arc<Definition>),
    Table(Table),
    FrontMatter(FrontMatter),
    Html,
    Ignore(Ignore),
    Blank,
    Raw,
    Extension(BlockExtension),
    Recovery(RecoveryReason),
}

#[derive(Debug)]
pub enum BlockExtension {
    Alert {
        label: Span,
        blocks: Vec<Block>,
    },
    Footnote {
        label: Span,
        blocks: Vec<Block>,
    },
    DescriptionList {
        entries: Vec<DescriptionEntry>,
    },
    Math {
        opening: Span,
        closing: Option<Span>,
        /// Text after the opening run, excluding surrounding horizontal whitespace.
        meta: Option<Span>,
        /// Content lines with container prefixes and opening indentation removed.
        body: Vec<SourceLine>,
    },
    /// A standalone template tag or output expression.
    Liquid {
        opening: Span,
        closing: Span,
    },
}
#[derive(Debug)]
pub struct DescriptionEntry {
    pub terms: Vec<Block>,
    pub descriptions: Vec<Description>,
}
#[derive(Debug)]
pub struct Description {
    pub marker: Span,
    pub span: Span,
    pub blocks: Vec<Block>,
}

#[derive(Debug)]
pub struct Heading {
    pub level: u8,
    /// ATX opening run; absent for a setext heading.
    pub opening: Option<Span>,
    /// Optional ATX closing run, or the setext underline.
    pub closing: Option<Span>,
    pub children: InlineNodes,
}

#[derive(Debug)]
pub struct List {
    pub ordered: bool,
    pub start: Option<u64>,
    pub items: Vec<ListItem>,
}

#[derive(Debug)]
pub struct ListItem {
    pub span: Span,
    pub marker: Span,
    pub task: Option<Task>,
    pub blocks: ItemBlocks,
}

/// A list item commonly contains a single paragraph.
pub type ItemBlocks = smallvec::SmallVec<[Block; 1]>;
#[derive(Debug, Clone, Copy)]
pub struct Task {
    pub marker: Span,
    pub checked: bool,
}

#[derive(Debug)]
pub struct Fence {
    /// Opening and closing marker runs, excluding indentation and trailing whitespace.
    pub opening: Span,
    pub closing: Option<Span>,
    pub info: Option<Span>,
    pub lang: Option<Span>,
    pub meta: Option<Span>,
    pub code: Code,
}

#[derive(Debug)]
pub struct Code {
    /// Logical lines joined by LF, excluding the last physical line terminator.
    pub value: String,
    pub lines: SourceLines,
    line_starts: smallvec::SmallVec<[usize; 4]>,
    empty_origin: usize,
}

impl Code {
    fn from_lines(source: &str, lines: SourceLines, empty_origin: usize) -> Self {
        if let Some(span) = source::contiguous_content(source, &lines) {
            let line_starts = lines
                .iter()
                .map(|line| line.content.start - span.start)
                .collect();
            return Self {
                value: span.text(source).to_owned(),
                lines,
                line_starts,
                empty_origin,
            };
        }
        let mut value = String::new();
        let mut line_starts = smallvec::SmallVec::with_capacity(lines.len());
        for (index, line) in lines.iter().enumerate() {
            if index != 0 {
                value.push('\n');
            }
            line_starts.push(value.len());
            line.append(source, &mut value);
        }
        Self {
            value,
            lines,
            line_starts,
            empty_origin,
        }
    }
    /// Original contiguous code, including its physical final terminator, suitable for borrowing.
    /// Container-prefix gaps and partially consumed tabs require mapping instead.
    pub fn contiguous_range(&self) -> Option<Span> {
        let start = self.lines.first()?.content.start;
        let mut end = start;
        for line in &self.lines {
            if line.virtual_spaces != 0 || line.content.start != end {
                return None;
            }
            end = line.ending.end;
        }
        (start < end).then_some(Span::new(start, end))
    }
    /// Translate a cooked UTF-8 byte offset to the original file. Residual spaces map to their tab.
    pub fn source_offset(&self, offset: usize) -> Option<usize> {
        if !self.value.is_char_boundary(offset) || offset > self.value.len() {
            return None;
        }
        if self.lines.is_empty() {
            return Some(self.empty_origin);
        }
        let index = self.line_starts.partition_point(|&n| n <= offset) - 1;
        let within = offset - self.line_starts[index];
        let line = self.lines[index];
        let spaces = usize::from(line.virtual_spaces);
        Some(if within < spaces {
            line.content.start - 1
        } else {
            line.content.start + within - spaces
        })
    }

    /// Map a cooked range into physical fragments without selecting container prefixes.
    /// A logical LF selects the complete original line ending. Any residual spaces select their tab.
    pub fn source_fragments(&self, range: Span) -> Option<Vec<Span>> {
        if range.start > range.end
            || !self.value.is_char_boundary(range.end)
            || range.end > self.value.len()
        {
            return None;
        }
        let start = self.source_offset(range.start)?;
        if range.is_empty() {
            return Some(vec![Span::new(start, start)]);
        }
        let first = self.line_starts.partition_point(|&n| n <= range.start) - 1;
        let mut spans = Vec::new();
        for i in first..self.lines.len() {
            let base = self.line_starts[i];
            if base >= range.end {
                break;
            }
            let line = self.lines[i];
            let virtual_end = base + usize::from(line.virtual_spaces);
            let content_end = virtual_end + line.content.len();
            let from = if line.virtual_spaces != 0 && range.start < virtual_end {
                line.content.start - 1
            } else {
                line.content.start + range.start.saturating_sub(virtual_end)
            };
            let to = if range.end > content_end {
                line.ending.end
            } else {
                line.content.start + range.end.saturating_sub(virtual_end)
            };
            if from < to {
                spans.push(Span::new(from, to));
            }
        }
        Some(spans)
    }
}

#[derive(Debug, Clone)]
pub struct Definition {
    pub brackets: [Span; 2],
    pub separator: Span,
    pub label: Span,
    pub identifier: String,
    pub destination: Span,
    pub title: Option<Span>,
}
#[derive(Debug)]
pub struct DefinitionEntry {
    pub span: Span,
    pub definition: Arc<Definition>,
}

#[derive(Debug)]
pub struct Table {
    pub header: Vec<TableCell>,
    pub delimiter: Span,
    pub alignments: smallvec::SmallVec<[Alignment; 4]>,
    pub rows: Vec<Vec<TableCell>>,
}
#[derive(Debug)]
pub struct TableCell {
    pub span: Span,
    pub children: InlineNodes,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    None,
    Left,
    Right,
    Center,
}

#[derive(Debug)]
pub struct FrontMatter {
    pub opening: Span,
    pub closing: Span,
    pub body: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ignore {
    Next,
    Start,
    End,
}

#[derive(Debug, Clone, Copy)]
pub struct Inline {
    pub span: Span,
    pub kind: InlineKind,
}
#[derive(Debug, Clone, Copy)]
pub enum InlineKind {
    Text,
    Escape,
    CodeSpan {
        /// Byte length of each bounding backtick run.
        width: usize,
        value: InlineText,
    },
    Emphasis(InlineNodes),
    Strong(InlineNodes),
    Delete(InlineNodes),
    Link(LinkId),
    Image(LinkId),
    Autolink,
    Html,
    SoftBreak,
    HardBreak,
    Raw,
    Extension(InlineExtension),
    Recovery(RecoveryReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InlineExtension {
    Entity,
    /// A double-bracket link on a single source line.
    WikiLink,
    /// An inline template tag or output expression.
    Liquid,
    Footnote {
        label: Span,
    },
    Math {
        display: bool,
    },
}

#[derive(Debug)]
pub struct Link {
    pub brackets: [Span; 2],
    pub target_delimiters: Option<[Span; 2]>,
    pub label: Span,
    pub children: InlineNodes,
    pub target: LinkTarget,
}
#[derive(Debug)]
pub enum LinkTarget {
    /// Raw destination and title include their angle or quote delimiters when present.
    Inline {
        destination: Span,
        title: Option<Span>,
    },
    Reference {
        label: Option<Span>,
        identifier: InlineText,
        kind: ReferenceKind,
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceKind {
    Full,
    Collapsed,
    Shortcut,
}

const MAX_DEPTH: usize = 48;

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_extended(source: &str) -> Document<'_> {
        parse_with_options(
            source,
            ParseOptions {
                dialect: Dialect::Extended,
                ..Default::default()
            },
        )
    }

    fn fence(block: &Block) -> &Fence {
        let BlockKind::Fence(value) = &block.kind else {
            panic!("expected fence")
        };
        value
    }

    fn paragraph<'a>(doc: &'a Document<'_>, block: &Block) -> &'a [Inline] {
        let BlockKind::Paragraph(nodes) = &block.kind else {
            panic!("expected paragraph")
        };
        doc.inlines(*nodes)
    }

    impl SyntaxCursor<'_> {
        fn children(self) -> Vec<Self> {
            let mut children = Vec::new();
            self.for_each_child(|node| children.push(node));
            children
        }

        fn descendants(self) -> impl Iterator<Item = Self> {
            let mut nodes = Vec::new();
            self.walk(&mut |node| nodes.push(node));
            nodes.into_iter()
        }
    }

    #[test]
    fn source_lines() {
        let mut document = parse("one\ntwo\n\nthree");
        document.blocks[0].lines[1].lazy = true;
        assert!(!document.lines[1].lazy);
        assert_eq!(document.blocks[0].lines.len(), 2);

        let doc = parse("\u{feff}λ\r\n> one\r> two\n");
        assert_eq!(doc.text(doc.lines[0].raw), "\u{feff}λ");
        assert_eq!(doc.text(doc.blocks[0].span), "λ");
        assert_eq!(doc.text(doc.lines[0].ending), "\r\n");
        assert_eq!(doc.text(doc.lines[1].ending), "\r");

        let BlockKind::BlockQuote(children) = &doc.blocks[1].kind else {
            panic!()
        };
        assert_eq!(
            children[0].logical_text(doc.source, children[0].span),
            "one\ntwo"
        );

        let source = format!("> [start\r\n{}> end](/guide)", "> *α*\r\n".repeat(300));
        let doc = parse(&source);
        let tree = doc.syntax();
        let link = tree
            .descendants()
            .find(|node| matches!(node.kind, SyntaxKind::Link { .. }))
            .unwrap();
        let emphasis: Vec<_> = link
            .descendants()
            .filter(|node| node.kind == SyntaxKind::Emphasis)
            .map(|node| node.span)
            .collect();
        let expected: Vec<_> = source
            .match_indices("*α*")
            .map(|(start, text)| Span::new(start, start + text.len()))
            .collect();
        assert_eq!(emphasis, expected);
    }

    #[test]
    fn containers_and_fences() {
        let doc = parse("> - [x] task\r\n>   ~~~ts linenums\r\n>   const π = 1;\r\n>   ~~~~\n");
        let BlockKind::BlockQuote(quote) = &doc.blocks[0].kind else {
            panic!()
        };
        let BlockKind::List(list) = &quote[0].kind else {
            panic!()
        };
        let item = &list.items[0];
        assert!(item.task.unwrap().checked);
        assert_eq!(doc.text(item.marker), "-");
        let fenced = fence(&item.blocks[1]);
        assert_eq!(doc.text(fenced.lang.unwrap()), "ts");
        assert_eq!(doc.text(fenced.meta.unwrap()), "linenums");
        assert_eq!(doc.text(fenced.closing.unwrap()), "~~~~");
        assert_eq!(fenced.code.value, "const π = 1;");
        assert_eq!(fenced.code.source_offset(6), doc.source.find('π'));
        assert_eq!(fenced.code.source_offset(7), None);

        let doc = parse("> \t~~~\r\n>\t\tλ\r\n");
        let BlockKind::BlockQuote(quote) = &doc.blocks[0].kind else {
            panic!()
        };
        let fenced = fence(&quote[0]);
        assert_eq!(fenced.closing, None);
        assert_eq!(fenced.code.value, "\tλ");

        let doc = parse("  ~~~\r\n\tλ\n  ~~~");
        let fenced = fence(&doc.blocks[0]);
        assert_eq!(fenced.code.value, "  λ");
        assert_eq!(fenced.code.source_offset(0), doc.source.find('\t'));
        assert_eq!(fenced.code.source_offset(2), doc.source.find('λ'));
    }

    #[test]
    fn container_lines() {
        fn body(source: &str, kind: SyntaxKind) -> Vec<(&str, bool, u8)> {
            let document = parse_extended(source);
            let syntax = document.syntax();
            let node = syntax.descendants().find(|node| node.kind == kind).unwrap();
            node.container_lines()
                .unwrap()
                .map(|line| (line.content.text(source), line.lazy, line.virtual_spaces))
                .collect()
        }

        let lines = body("> - a\n> b\n>\n> c", SyntaxKind::Quote);
        assert_eq!(
            lines,
            [
                ("- a", false, 0),
                ("b", false, 0),
                ("", false, 0),
                ("c", false, 0)
            ]
        );

        let lines = body("> - a\n> b", SyntaxKind::ListItem);
        assert_eq!(lines, [("a", false, 0), ("b", true, 0)]);

        let lines = body("> a\r\nb", SyntaxKind::Quote);
        assert_eq!(lines, [("a", false, 0), ("b", true, 0)]);

        let lines = body("- [x]\n  task\n  ~~~\n\tbody\n  ~~~", SyntaxKind::ListItem);
        assert_eq!(
            lines,
            [
                ("[x]", false, 0),
                ("task", false, 0),
                ("~~~", false, 0),
                ("body", false, 2),
                ("~~~", false, 0)
            ]
        );

        let lines = body("> [!NOTE]\n> text", SyntaxKind::Alert);
        assert_eq!(lines, [("[!NOTE]", false, 0), ("text", false, 0)]);
    }

    #[test]
    fn references() {
        let doc = parse(
            "[A b]: <somewhere> 'title'\n[a B]: other\n\n[hello][a b] and ![](link) and [missing]",
        );
        assert_eq!(doc.definitions.len(), 2);
        let first = &doc.definition("  a   B ").unwrap().definition;
        assert_eq!(doc.text(first.destination), "<somewhere>");
        assert_eq!(doc.text(first.title.unwrap()), "'title'");
        let nodes = paragraph(&doc, doc.blocks.last().unwrap());
        let InlineKind::Link(reference) = &nodes[0].kind else {
            panic!()
        };
        let LinkTarget::Reference {
            label: Some(label),
            kind,
            ..
        } = &doc.link(*reference).target
        else {
            panic!()
        };
        assert_eq!(*kind, ReferenceKind::Full);
        assert_eq!(doc.text(*label), "a b");
        let InlineKind::Image(image) = &nodes[2].kind else {
            panic!()
        };
        assert_eq!(doc.text(doc.link(*image).label), "");
        assert!(matches!(nodes[4].kind, InlineKind::Link(_)));
        assert!(doc.definition("missing").is_none());

        let doc = parse("[ẞ]\n\n[SS]: /first\n[ß]: /second");
        let first = doc.definition("ẞ").unwrap();
        assert_eq!(doc.text(first.definition.destination), "/first");

        let doc = parse("[I]: /url");
        assert!(doc.definition("ı").is_none());

        // Beria Erfe case pairs require Unicode 17 or later.
        let doc = parse("[\u{16ea0}]: /url");
        assert!(doc.definition("\u{16ebb}").is_some());

        let doc = parse("*text [label* tail]");
        let nodes = paragraph(&doc, &doc.blocks[0]);
        assert!(matches!(nodes[0].kind, InlineKind::Emphasis(_)));
        assert_eq!(doc.text(nodes[0].span), "*text [label*");
        assert_eq!(doc.text(nodes[1].span), " tail]");

        let doc = parse("*text [label* tail]\n\n[label* tail]: /url");
        let nodes = paragraph(&doc, &doc.blocks[0]);
        assert!(matches!(nodes[0].kind, InlineKind::Text));
        assert!(matches!(nodes[1].kind, InlineKind::Link(_)));
        assert_eq!(doc.text(nodes[1].span), "[label* tail]");

        let doc = parse("*outer [*label*] text*");
        let InlineKind::Emphasis(children) = &paragraph(&doc, &doc.blocks[0])[0].kind else {
            panic!()
        };
        assert!(matches!(
            doc.inlines(*children)[1].kind,
            InlineKind::Link(_)
        ));
    }

    #[test]
    fn inline_values() {
        let doc = parse("` first ` [x][ẞ] `second\nline`");
        let nodes = paragraph(&doc, &doc.blocks[0]);
        let InlineKind::CodeSpan { value: first, .. } = nodes[0].kind else {
            panic!()
        };
        let InlineKind::Link(link) = nodes[2].kind else {
            panic!()
        };
        let LinkTarget::Reference { identifier, .. } = doc.link(link).target else {
            panic!()
        };
        let InlineKind::CodeSpan { value: second, .. } = nodes[4].kind else {
            panic!()
        };
        assert_eq!(doc.inline_text(first), "first");
        assert_eq!(doc.inline_text(identifier), "ss");
        assert_eq!(doc.inline_text(second), "second line");
    }

    #[test]
    fn nested_links() {
        fn links(source: &str) -> Vec<&str> {
            let document = parse(source);
            let syntax = document.syntax();
            syntax
                .descendants()
                .filter(|node| matches!(node.kind, SyntaxKind::Link { .. }))
                .map(|node| node.span.text(source))
                .collect()
        }

        let spans = links("[outer *[inner](/i)*](/o)");
        assert_eq!(spans, ["[inner](/i)"]);

        let spans = links("[outer [inner][r]](/o)\n\n[r]: /i");
        assert_eq!(spans, ["[inner][r]"]);

        let spans = links("[outer ![alt](/img)](/o)");
        assert_eq!(spans, ["[outer ![alt](/img)](/o)", "![alt](/img)"]);

        let spans = links("![alt [inner](/i)](/img)");
        assert_eq!(spans, ["![alt [inner](/i)](/img)", "[inner](/i)"]);

        let spans = links("[outer [missing]](/o)");
        assert_eq!(spans.first(), Some(&"[outer [missing]](/o)"));

        let doc = parse("[a <i title=\"]\"> b](/target)");
        let InlineKind::Link(link) = &paragraph(&doc, &doc.blocks[0])[0].kind else {
            panic!()
        };
        let link = doc.link(*link);
        assert_eq!(doc.text(link.label), "a <i title=\"]\"> b");
        assert!(matches!(
            doc.inlines(link.children)[1].kind,
            InlineKind::Html
        ));

        let doc = parse("[x <https://example.org/?q=](x)>");
        let nodes = paragraph(&doc, &doc.blocks[0]);
        assert!(matches!(nodes[0].kind, InlineKind::Text));
        assert!(matches!(nodes[1].kind, InlineKind::Autolink));
        assert_eq!(doc.text(nodes[1].span), "<https://example.org/?q=](x)>");
    }

    #[test]
    fn inline_delimiters() {
        let doc = parse("a <!--> b <!---> c");
        let comments: Vec<_> = paragraph(&doc, &doc.blocks[0])
            .iter()
            .filter(|node| matches!(node.kind, InlineKind::Html))
            .map(|node| doc.text(node.span))
            .collect();
        assert_eq!(comments, ["<!-->", "<!--->"]);

        let doc = parse("*£*word");
        let nodes = paragraph(&doc, &doc.blocks[0]);
        assert_eq!(nodes.len(), 1);
        assert!(matches!(nodes[0].kind, InlineKind::Text));

        let doc = parse("*\u{20dd}*word");
        assert!(matches!(
            paragraph(&doc, &doc.blocks[0])[0].kind,
            InlineKind::Emphasis(_)
        ));
    }

    #[test]
    fn definition_paragraphs() {
        let doc = parse("[a]: /b\n    code");
        assert!(matches!(doc.blocks[1].kind, BlockKind::Paragraph(_)));

        let doc = parse("[a]: /b\n\n    code");
        assert!(matches!(doc.blocks[2].kind, BlockKind::IndentedCode(_)));

        let doc = parse("[a]: /b\n    [b]: /c");
        assert_eq!(
            doc.text(doc.definition("b").unwrap().definition.destination),
            "/c"
        );

        let doc = parse("[a\n> b]: /c");
        assert!(doc.definitions.is_empty());
        assert!(matches!(doc.blocks[1].kind, BlockKind::BlockQuote(_)));
    }

    #[test]
    fn footnote_paragraphs() {
        let doc = parse_extended("[^n]: first\ncontinued\n\noutside");
        let tree = doc.syntax();
        let note = tree.children()[0];
        assert_eq!(note.kind, SyntaxKind::FootnoteDefinition);
        assert_eq!(doc.text(note.children()[0].span), "first\ncontinued");
        assert_eq!(doc.text(tree.children()[2].span), "outside");

        let doc = parse_extended("[^n]: first\n\n    second\ncontinued");
        let tree = doc.syntax();
        assert_eq!(
            doc.text(tree.children()[0].children()[2].span),
            "second\ncontinued"
        );
    }

    #[test]
    fn description_paragraphs() {
        let doc = parse_extended("Term\n: first\n\n    second\n\n  outside");
        let tree = doc.syntax();
        let description = tree.children()[0].children()[1];
        assert_eq!(description.kind, SyntaxKind::Description);
        assert_eq!(description.children()[2].kind, SyntaxKind::Paragraph);
        assert_eq!(doc.text(description.children()[2].span), "second");
        assert_eq!(doc.text(tree.children()[2].span).trim_start(), "outside");
    }

    #[test]
    fn continuation_boundaries() {
        let doc = parse("[^a]: text\n\n    more\n\n[^b]: next");
        assert_eq!(doc.text(doc.blocks[0].span), "[^a]: text\n\n    more");

        let doc = parse("> - first\nlazy continuation");
        let BlockKind::BlockQuote(children) = &doc.blocks[0].kind else {
            panic!()
        };
        let BlockKind::List(list) = &children[0].kind else {
            panic!()
        };
        let body = &list.items[0].blocks[0];
        assert_eq!(
            body.logical_text(doc.source(), body.span),
            "first\nlazy continuation"
        );

        let doc = parse("> <div>\n> >\n> body\noutside");
        assert!(matches!(doc.blocks[1].kind, BlockKind::Paragraph(_)));
        assert_eq!(doc.text(doc.blocks[1].span), "outside");

        let doc = parse("- <!-- open\n  comment\noutside");
        assert_eq!(doc.text(doc.blocks[1].span), "outside");

        let doc = parse("> - a\n2. b");
        assert!(matches!(doc.blocks[1].kind, BlockKind::List(_)));

        let doc = parse("> - item\n\t~~~\noutside");
        let tree = doc.syntax();
        let quote = tree.children()[0];
        assert_eq!(doc.text(quote.children()[0].span), "- item\n\t~~~");
        assert_eq!(doc.text(quote.children()[1].span), "outside");

        let doc = parse("> [^n]: note\n> continued\n> 2. item\nnext");
        let tree = doc.syntax();
        let list = tree.children()[0].children()[1];
        assert_eq!(list.kind, SyntaxKind::List { ordered: true });
        assert_eq!(doc.text(list.span), "2. item\nnext");

        let doc = parse("> > a\n> +");
        let tree = doc.syntax();
        assert_eq!(
            tree.children()[0].children()[1].kind,
            SyntaxKind::List { ordered: false }
        );

        let doc = parse("> a\n> 2. text\ncontinued");
        let tree = doc.syntax();
        assert_eq!(tree.children()[0].children().len(), 1);
        assert_eq!(doc.text(tree.children()[0].span), doc.source());

        let doc = parse("> > <pre>\n> > code\n> paragraph\ncontinued");
        let BlockKind::BlockQuote(children) = &doc.blocks[0].kind else {
            panic!()
        };
        assert_eq!(
            children[1].logical_text(doc.source(), children[1].span),
            "paragraph\ncontinued"
        );

        let doc = parse("Read https://example.com/a_(b). Then a@b.example!");
        let nodes = paragraph(&doc, &doc.blocks[0]);
        assert_eq!(doc.text(nodes[1].span), "https://example.com/a_(b)");
        assert_eq!(doc.text(nodes[3].span), "a@b.example");
    }

    #[test]
    fn table_boundaries() {
        let doc = parse("> a | b\n> - | -\nc | d");
        let tree = doc.syntax();
        assert_eq!(
            tree.children()[0].children()[1].kind,
            SyntaxKind::List { ordered: false }
        );

        let doc = parse("- |a|\n  |-|\n  |b|\n|c|");
        assert_eq!(doc.text(doc.blocks[1].span), "|c|");
        let tree = doc.syntax();
        let table = tree
            .descendants()
            .find(|node| node.kind == SyntaxKind::Table)
            .unwrap();
        assert_eq!(table.children().len(), 2);

        let doc = parse("> a | b\n-|-\nc | d");
        assert_eq!(
            doc.syntax().children()[0].children()[0].kind,
            SyntaxKind::Paragraph
        );

        let doc = parse("first\n|a|\n|-|\n2. b");
        assert!(matches!(doc.blocks[0].kind, BlockKind::Paragraph(_)));
        assert!(matches!(doc.blocks[1].kind, BlockKind::Table(_)));
        assert!(matches!(doc.blocks[2].kind, BlockKind::List(_)));
    }

    #[test]
    fn lexical_fields() {
        let doc = parse("# *first* and last ##");
        let heading = doc.syntax().children()[0];
        assert_eq!(
            doc.text(heading.heading_content().unwrap()),
            "*first* and last"
        );

        let doc = parse("#");
        assert!(doc.syntax().children()[0].heading_content().is_none());

        let doc = parse("> [](< > \"\")\r\n> ![][id]");
        let view = doc.syntax();
        let links: Vec<_> = view
            .descendants()
            .filter(|n| matches!(n.kind, SyntaxKind::Link { .. }))
            .collect();
        assert_eq!(links.len(), 2);
        let mut fields = Vec::new();
        let empty = links[0]
            .read(&mut fields)
            .parts
            .iter()
            .find(|p| p.role == SyntaxRole::Title)
            .unwrap();
        assert_eq!(empty.span, Span::new(10, 10));
        assert_eq!(
            links[0].fragments_in(empty.span).collect::<Vec<_>>(),
            [empty.span]
        );
        view.walk(&mut |node| {
            let mut ranges = Vec::new();
            node.for_each_gap(|gap| ranges.push(gap));
            node.for_each_child(|child| ranges.push(child.span));
            ranges.sort_by_key(|s| (s.start, s.end));
            let combined: String = ranges.into_iter().map(|r| doc.text(r)).collect();
            assert_eq!(combined, doc.text(node.span));
        });

        let doc = parse("> ~~~text\r\n> 一\r\n> two\r\n> ~~~");
        let node = doc.syntax().children()[0].children()[0];
        assert_eq!(node.lines().len(), 2);
        let surface: String = node.fragments().map(|span| doc.text(span)).collect();
        assert_eq!(surface, "~~~text\r\n一\r\ntwo\r\n~~~");

        let BlockKind::BlockQuote(children) = &doc.blocks[0].kind else {
            panic!()
        };
        let code = &fence(&children[0]).code;
        let parts = code
            .source_fragments(Span::new(0, code.value.len()))
            .unwrap();
        assert_eq!(
            parts.into_iter().map(|s| doc.text(s)).collect::<Vec<_>>(),
            ["一\r\n", "two"]
        );
        assert!(code.contiguous_range().is_none());

        let doc = parse("> *one*\r\n> two");
        let emphasis = doc
            .syntax()
            .descendants()
            .find(|n| n.kind == SyntaxKind::Emphasis)
            .unwrap();
        let entire = Span::new(0, doc.source().len());
        let fragments: Vec<_> = emphasis.fragments_in(entire).map(|s| doc.text(s)).collect();
        assert_eq!(fragments, ["*one*", "\r\n"]);
    }

    #[test]
    fn link_field_escapes() {
        fn escapes(source: &str) -> Vec<Span> {
            let document = parse(source);
            let syntax = document.syntax();
            syntax
                .descendants()
                .filter(|node| node.kind == SyntaxKind::Escape)
                .map(|node| {
                    assert_eq!(node.fragments().collect::<Vec<_>>(), [node.span]);
                    assert_eq!(
                        node.read(&mut Vec::new()).parts,
                        [SyntaxPart {
                            role: SyntaxRole::Marker,
                            span: Span::new(node.span.start, node.span.start + 1),
                        }]
                    );
                    node.span
                })
                .collect()
        }

        let ranges = escapes(r#"[x\!](<π\*> "t\?")"#);
        assert_eq!(
            ranges,
            [Span::new(2, 4), Span::new(9, 11), Span::new(15, 17)]
        );

        let ranges = escapes(r"![x](/a\\\*\q)");
        assert_eq!(ranges, [Span::new(7, 9), Span::new(9, 11)]);

        let ranges = escapes("> [i\\*]: /π\\!\r\n>   \"a\r\n>    b\\?\"");
        assert_eq!(
            ranges,
            [Span::new(4, 6), Span::new(12, 14), Span::new(30, 32)]
        );

        let ranges = escapes(r"[x][id\!]");
        assert_eq!(ranges, [Span::new(6, 8)]);

        assert!(escapes(r"<https://example.com/a\*>").is_empty());
    }

    #[test]
    fn link_field_entities() {
        fn entities(source: &str) -> Vec<&str> {
            let document = parse(source);
            let syntax = document.syntax();
            syntax
                .descendants()
                .filter(|node| node.kind == SyntaxKind::Entity)
                .map(|node| node.span.text(source))
                .collect()
        }

        let fields = entities(r#"[x](<π&amp;> "&quot;\&amp;&#34;&unknown;")"#);
        assert_eq!(fields, ["&amp;", "&quot;", "&#34;"]);

        let fields = entities("> [a&amp;]: /&#x21;\r\n>   \"&quot;\"");
        assert_eq!(fields, ["&#x21;", "&quot;"]);

        assert!(entities("[x][a&amp;]").is_empty());
        assert!(entities("<https://example.com/?x&amp;y>").is_empty());
    }

    #[test]
    fn recovery() {
        let limited = ParseOptions {
            max_inline_steps: Some(22),
            ..Default::default()
        };

        let doc = parse_with_options("https://example.org/日本", limited);
        let nodes = paragraph(&doc, &doc.blocks[0]);
        assert!(matches!(nodes[0].kind, InlineKind::Text));
        assert!(matches!(
            nodes[1].kind,
            InlineKind::Recovery(RecoveryReason::InlineWork)
        ));

        let doc = parse_with_options("editor@markdown.example.org", limited);
        let nodes = paragraph(&doc, &doc.blocks[0]);
        assert!(matches!(
            nodes.last().unwrap().kind,
            InlineKind::Recovery(RecoveryReason::InlineWork)
        ));

        let doc = parse_with_options(
            "é一",
            ParseOptions {
                max_inline_steps: Some(1),
                ..Default::default()
            },
        );
        let root = doc.syntax();
        let recovered = root
            .descendants()
            .find(|n| n.kind == SyntaxKind::Recovery(RecoveryReason::InlineWork))
            .unwrap();
        assert_eq!(doc.text(recovered.span), "一");

        let doc = parse("~~~guest\n**untouched\n  [unterminated(");
        assert!(fence(&doc.blocks[0]).closing.is_none());
        assert_eq!(
            fence(&doc.blocks[0]).code.value,
            "**untouched\n  [unterminated("
        );

        let source = format!("{}x\n", "> ".repeat(4096));
        let doc = parse(&source);
        let mut block = &doc.blocks[0];
        while let BlockKind::BlockQuote(children) = &block.kind {
            block = &children[0];
        }
        assert!(matches!(
            block.kind,
            BlockKind::Recovery(RecoveryReason::Nesting)
        ));

        let source = format!("{}text\n> more\nlast", "> ".repeat(MAX_DEPTH + 2));
        let doc = parse(&source);
        let tree = doc.syntax();
        let recovered = tree
            .descendants()
            .find(|node| node.kind == SyntaxKind::Recovery(RecoveryReason::Nesting))
            .unwrap();
        assert_eq!(doc.text(recovered.span), "> > text\n> more\nlast");
    }
}
