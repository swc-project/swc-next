# SWC Next JavaScript Formatter

**Experimental and unstable.** APIs, document formats, and formatting output may
change without notice.

Source-aware JavaScript, JSX, TypeScript, and TSX formatting for the SWC Next AST.
Formats source text or an existing program AST and produces layout documents for
shared formatter sessions. Syntax support is incomplete; unsupported programs
return an error. See the crate documentation for formatting options and AST
parsing prerequisites.

```rust
let output = swc_next_formatter_js::format(
    "const answer=42",
    Default::default(),
    Default::default(),
)?;
```

Run the crate tests with `cargo test -p swc_next_formatter_js`.
