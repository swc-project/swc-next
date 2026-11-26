# swc_formatter_markdown

Markdown formatting for SWC Next. Produces a shared layout document from Markdown source or a parsed Markdown document. Supports prose wrapping, headings, lists, block quotes, reference definitions, tables, and embedded code fences.

**Experimental and unstable.** APIs and formatting output may change.

```rust
use swc_next_formatter_host::FormatSession;
use swc_next_formatter_markdown::{MarkdownDocumentProducer, SyntaxOptions};

let session = FormatSession::default();
let document = session.produce(
    &MarkdownDocumentProducer,
    "#  Example\n",
    &SyntaxOptions::default(),
)?;
let formatted = document.print(session.print_options());
```

Use `ast_to_doc` with a parsed Markdown document. Configure layout through the formatting session and Markdown syntax through `SyntaxOptions`.

SWC intentionally preserves literal `*` and `_` where Prettier may add escapes. It changes the surrounding delimiters only when the emphasis spans and their nesting stay the same. For example, `*foo**bar*` formats as `_foo**bar_`, and `x*a**b*` keeps its original spelling.

With `ProseWrap::Always`, SWC may exceed the print width to keep a literal backslash visible or a link destination such as `#` beside its definition label. It also keeps paragraph text together where a line break would introduce a GFM table, footnote definition, or display math block. Indented code at the start of a footnote stays on its own line. These line breaks can differ from Prettier.

Markdown prints the delimiters and indentation for code fences and front matter. The host installs an `EmbeddedDispatcher` to select a formatter and supply the child Doc. It receives the decoded fence language and metadata. Markdown keeps their source spelling when printing the decoded text would change their meaning. When embedding is disabled or the host declines a language, Markdown preserves the body and reapplies the surrounding indentation. Ignored blocks and opaque syntax retain their source content.

## Testing

```sh
cargo test -p swc_next_formatter_markdown --locked
```
