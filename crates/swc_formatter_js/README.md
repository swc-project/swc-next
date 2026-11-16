# swc_formatter_js

JavaScript, JSX, TypeScript, and TSX document production for SWC Next. Converts source text or an existing program AST into document IR for shared formatting sessions. Source-to-string and AST-to-string formatting live in [`swc_formatter`](../swc_formatter).

**Experimental and unstable.** Syntax support is incomplete, and output can differ from Prettier. Unsupported syntax returns an error.

## Usage

```rust
use swc_next_formatter_host::FormatSession;
use swc_next_formatter_js::{EcmaDocumentProducer, SyntaxOptions};

let session = FormatSession::default();
let producer = EcmaDocumentProducer::default();
let document = session.produce(&producer, "const answer=42", &SyntaxOptions::default())?;
let encoded = document.encode();
```

See the [crate API](src/lib.rs) for formatting options and AST input requirements.

## Testing

Module unit tests cover internal document construction. API and fixture tests live in [`swc_formatter`](../swc_formatter/tests).

```sh
cargo test -p swc_next_formatter_js --locked
```

See the [test documentation](../swc_formatter/tests) for fixtures and Prettier conformance coverage.
