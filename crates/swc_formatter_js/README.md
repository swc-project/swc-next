# swc_formatter_js

JavaScript, JSX, TypeScript, and TSX formatting for SWC Next. Formats source text or an existing program AST and produces documents for shared formatting sessions.

**Experimental and unstable.** Syntax support is incomplete, and output can differ from Prettier. Unsupported syntax returns an error.

## Usage

```rust
let output = swc_next_formatter_js::format(
    "const answer=42",
    Default::default(),
    Default::default(),
)?;
```

See the [crate API](src/lib.rs) for formatting options and AST input requirements.

## Testing

API and fixture tests cover options, source and AST entry points, document serialization, and idempotency.

```sh
cargo test -p swc_next_formatter_js --locked
```

See the [test documentation](tests) for fixtures and Prettier conformance coverage.
