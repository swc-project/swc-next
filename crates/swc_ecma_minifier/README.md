# swc_ecma_minifier

`swc_next_ecma_minifier` optimizes an existing JavaScript Program AST through
`Minifier::minify(&mut ast, options)`. Parse and validate the program first, then
print it with `swc_next_ecma_codegen` and `minify: true`. Recompute semantic
analysis if it is needed after minification.

Compression currently shortens boolean expressions: `true` becomes `!0` and
`false` becomes `!1`. `MinifierOptions::default()` enables compression with
`CompressOptions { booleans: true }`; setting `booleans: false` disables this
optimization, while `compress: None` leaves the entire AST unchanged. Parsing,
TypeScript/JSX lowering, mangling, and printing are separate concerns.

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
