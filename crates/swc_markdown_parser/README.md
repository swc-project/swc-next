# Markdown parser

`swc_next_markdown_parser` parses Markdown into source-backed block and inline trees. It is experimental.

```rust
use swc_next_markdown_parser::{BlockKind, parse};

let document = parse("> hello\r\n> world");
let BlockKind::BlockQuote(blocks) = &document.blocks[0].kind else { unreachable!() };
assert_eq!(blocks[0].logical_text(document.source(), blocks[0].span), "hello\nworld");
```

The tree includes headings, lists and tasks, block quotes, links and reference definitions, tables, front matter, ignore markers, and code. It also recognizes footnote definitions, display math, wiki links, and Liquid tags and expressions. Code fences expose their original markers, info string, logical body, and a mapping back to UTF-8 byte offsets. Definitions remain in document order, and lookup selects the first matching definition. Standalone unresolved references retain their source nodes.

Use `Document::syntax()` to visit lexical nodes in source order and read their fields into a reusable buffer. The cursor exposes marker roles, field ranges, and gaps between children. Empty fields retain their insertion offsets. On a quote, alert, or list item, `container_lines` exposes the content and prefix of each physical line, including blank lines and lazy continuations. For guest ranges within code fences, `Code::source_fragments` maps cooked offsets through original line endings, indentation, and container prefixes.

`parse_with_options` accepts resource limits and an extended dialect with alerts, description lists, footnote references, inline math, character references, and single-tilde strikethrough. Recovery nodes identify the limit reached and retain the affected source. HTML and unsupported extension syntax can retain opaque source. Deeply nested or unusually ambiguous input can produce raw nodes. The parser does not validate or execute embedded languages. Full CommonMark and GFM conformance is under development.

Run `cargo test -p swc_next_markdown_parser` to test the crate.
