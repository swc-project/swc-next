# swc_formatter

Source formatting, configuration, and document transport for SWC Next. Provides shared print options and JavaScript, TypeScript and Markdown formatting APIs.

**Experimental and unstable.** APIs, binary formats, and formatting output may change. Use matching SWC Next wrappers and bindings.

Format JavaScript and TypeScript source or an existing program AST through `js::format` and `js::format_ast`:

```rust
let output = swc_next_formatter::js::format(
    "const answer=42",
    Default::default(),
    Default::default(),
)?;
```

Convert `PrintConfig` into `swc_next_formatter_core::PrintOptions`, or convert `js::JsFormatterConfig` into `swc_next_formatter_js::FormatOptions`. Enable `serde` for configuration deserialization and `bridge` for internal binary document transport. See the [crate API](src/lib.rs) for options and defaults.

Use `markdown::format` for Markdown source and `markdown::format_ast` for a
document produced by `swc_next_markdown_parser::parse`. JavaScript, JSX,
TypeScript, TSX and Markdown code fences use the built-in formatters. Unknown
languages and incomplete code examples retain their contents.

Use `markdown::format_with_dispatcher` or `markdown::format_ast_with_dispatcher`
to supply a host for code fences. `BuiltinFormatters` accepts syntax options
for each built-in language. A custom `EmbeddedDispatcher` can supply others.

```rust
let output = swc_next_formatter::markdown::format(
    "```ts\nconst n:number=1\n```\n",
    Default::default(),
)?;
```

## Command-line example

The `format` example infers the parser from each path and prints formatted
source to stdout in input order. Use `--check` to report files that would change
or `--write` to update them in place.

```sh
cargo run -p swc_next_formatter --example format -- index.ts README.md
cargo run -p swc_next_formatter --example format -- --check src/*.js
cargo run -p swc_next_formatter --example format -- --write src/*.tsx README.md
printf 'const answer:number=42\n' | cargo run -p swc_next_formatter --example format -- --stdin-filepath input.ts
printf '#  Title\n' | cargo run -p swc_next_formatter --example format -- --parser markdown
```

The exit status is `0` on success, `1` when `--check` finds a change, and `2`
for errors.

## Checking directive targets

Compare a file with a candidate output, or omit the candidate to check the
first and second formatter passes. Both files use the input's dialect.

```sh
cargo run -p swc_next_formatter --example directive_targets -- before.js after.js --show-targets
cargo run -p swc_next_formatter --example directive_targets -- input.ts --no-semi --width 40
```

The check aligns SWC AST nodes and source tokens. It checks a standalone
`/*#__PURE__*/` or `/*@__PURE__*/` against the call or constructor beginning
at the next token. In JS and JSX, a single-tag `/** @type {T} */` cast must
still enclose the same expression in explicit parentheses. Identical calls or
identifiers at different positions have distinct targets.
`--show-targets` prints the byte span and a source excerpt for each resolved
target in each file.

Program and function directive prologues are compared in their own scopes and
in source order. The raw values between the quotes distinguish `"use strict"`
from `"\x75se strict"`, and both activation and loss count as changes.

For `@ts-ignore` and `@ts-expect-error`, the check compares the code tokens on
the covered line, including lost or added coverage across a line break. Blank
lines and standalone line comments can intervene. This is structural coverage.
Actual diagnostics and inferred JSDoc types require a type checker.

The exit status is `0` for preserved targets, `1` for a confirmed structural
change, and `2` for a parse error or an unverified comparison. A confirmed
change takes precedence when both occur. Newly active markers are allowed on
the first pass. The second pass must also reproduce the first output bytes.
Ambiguous call chains, mixed markers, and syntax or diagnostic-line
punctuation that cannot be aligned produce an unverified result.
Moving only delimiters such as `]` or `;` across a diagnostic line returns
`2`, as does gaining code from a source line that contained only delimiters.
Moving previously covered code off the line returns `1`, even if a delimiter
stays on that line.

## Testing

Unit tests cover binary print options and document printing. The [API and fixture tests](tests) cover options, source and AST entry points, document serialization, and idempotency.

```sh
cargo test -p swc_next_formatter --all-features --locked
```
