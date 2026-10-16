# swc_ecma_minifier

`swc_next_ecma_minifier` optimizes an existing JavaScript Program AST through
`Minifier::minify(&mut ast, options)`. Parse and validate the program first, then
print it with `swc_next_ecma_codegen` and `minify: true`. Recompute semantic
analysis if it is needed after minification.

Compression uses one AST visitor for all four optimizations. Each option in
`CompressOptions` defaults to `true`:

- `booleans`: `true` becomes `!0` and `false` becomes `!1`.
- `global_defs`: unshadowed global reads of `undefined`, `Infinity`, and `NaN`
  become `void 0`, `1/0`, and `0/0`. Writes and operands within `delete` stay
  unchanged. Object shorthand keeps its original property name.
- `typeofs`: comparisons between `typeof x` and `"undefined"` become strict
  comparisons between `x` and `void 0`, in either operand order. This requires a
  single function-local `var` declaration, no writes, and an absent or primitive
  literal initializer in a standalone declaration. Unknown values stay unchanged
  because `document.all` also has a `typeof` result of `"undefined"`.
- `unused`: remove unreferenced, singly declared function-local `var`, `let`, and
  `const` identifiers with absent or primitive literal initializers. Preserve
  destructuring, loop headers, top-level declarations, and potentially effectful
  initializers. Empty statements preserve statement positions and prevent a
  following string from becoming a directive.

Semantic optimizations share one analysis before compression. Candidate lookup
uses semantic indexes, without another AST traversal. Forward `typeof` uses are
recorded during that same visitor and patched on program exit after initializer
facts are known; the recorded nodes are not visited again. New nodes are never
queried through the original semantic model. Any `eval` reference or `with` scope
disables all three semantic optimizations for the program; boolean shortening
remains enabled.

Setting all three semantic options to `false` skips analysis. `compress: None`
skips both analysis and compression. Parsing, TypeScript/JSX lowering, mangling,
and printing remain separate concerns. Callers must still validate the input
and recompute semantic analysis if they need it after minification.

## Local fixtures

From the workspace root:

```sh
cargo test -p swc_next_ecma_minifier --locked
cargo test -p swc_next_ecma_minifier --test fixtures -- preserve/script
cargo test -p swc_next_ecma_minifier --test fixtures -- --list
```

Add a directory under `tests/fixtures/<feature>/<case>/` containing `input.js`
and `output.js`. Cases are discovered automatically and run as individually
named tests in the existing workspace test CI. No fixture downloads or Node.js
installation are required.

An optional `options.json` selects the source type and AST optimization options:

```json
{
  "source_type": "script",
  "minify": {
    "compress": null
  }
}
```

Source type defaults to `module`; `script` is the other supported value.
Omitted minifier options use their Rust defaults. `compress: null` disables
compression. Unknown options, malformed JSON, missing inputs/outputs, parser
errors, and semantic errors fail the test.

To disable only boolean shortening, use
`{"minify":{"compress":{"booleans":false}}}`. An empty compression object
(`{"minify":{"compress":{}}}`) uses the default compression settings.

Every case parses and checks the input, minifies it, and prints compact output.
It then reparses and checks the output, minifies it again, and requires identical
output on the second pass. The first output must match `output.js`, normalizing
only CRLF to LF and trailing newlines. Other whitespace remains significant.
Expected files are edited manually and are never overwritten by the runner.
These checks do not compare runtime behavior.

## Adding an optimization

Add an option to `CompressOptions` and implement the transformation in the
internal compression visitor. Use the visitor's expression replacement return
value when changing expression kinds and preserve source spans. Add small
fixtures for the transformation, its disabled option, and relevant syntax or
precedence boundaries. Keep printing policy in the code generator.
