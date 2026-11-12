# Transformer compile tests

Local output regressions live under `compile/<feature>/<case>/`. Each case has
one `input.js`, `input.jsx`, `input.ts`, or `input.tsx` and an insta
`output.js.snap` containing the complete emitted code. Cases expecting diagnostics
also have `diagnostics.snap`, including severity, message, spans, help, and
labels. No file means no diagnostics are expected; remove the diagnostic snapshot
when a case no longer reports diagnostics. Inputs preserve their
original whitespace, including CRLF and Unicode line separators.

The nearest `options.json`, in the case directory or an ancestor within
`compile/`, contains `swc_next_core::js::JsTransformApiOptions`.
An options file replaces ancestor options; files are not merged. Set parser
language and source type explicitly when a case depends on them. Transforms use
the public `transform_js` pipeline, and their output must parse with the fixture's
parser configuration.

One integration test uses `insta::glob!` to visit inputs in path order. Run the
compile tests, optionally selecting inputs with `INSTA_GLOB_FILTER`:

```sh
cargo test -p swc_next_ecma_transformer --test compile --locked
INSTA_GLOB_FILTER='**/refresh-*/**' cargo test -p swc_next_ecma_transformer --test compile --locked
```

Update intentional changes, review the snapshot diff, then rerun without updates:

```sh
INSTA_UPDATE=always cargo test -p swc_next_ecma_transformer --test compile --locked
```

Binding resolution, AST invariants, and diagnostic spans remain unit tests.
