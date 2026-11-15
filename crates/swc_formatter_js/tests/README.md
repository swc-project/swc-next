# Formatter tests

`api.rs` covers public options, parser configuration, and errors. Local fixtures live under `fixtures/<language>/<syntax>/`, with expected output in adjacent `.snap` files. The nearest `options.json` supplies formatting option sets.

Fixtures check source and AST entry-point consistency, document encoding and decoding, output parsing, comment counts, and second-pass idempotency. Parsing and comment counts alone do not establish semantic equivalence.

```sh
cargo test -p swc_next_formatter_js --locked
```

Upstream Prettier compatibility is covered by the [testsuite conformance runner](../../../tasks/testsuite).
