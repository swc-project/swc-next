# swc_ecma_minifier

JavaScript AST optimization for SWC Next. Compression folds constant
expressions, removes branches that constant conditions never run, shortens
booleans and unshadowed global constants, simplifies safe local `typeof`
comparisons, and removes unused local variables. Mangling shortens the names
of local bindings; top-level names are kept.

Use `Minifier::minify(&mut ast, options)` on a valid JavaScript Program AST.
Parsing, TypeScript/JSX lowering, and printing are separate operations. Print
with the code generator's `minify` option for compact output, and recompute
semantic analysis before querying the optimized AST.

See [minifier options](src/options.rs) for configuration. A program that uses
`eval` or `with` is neither compressed with semantic optimizations nor mangled.

## Testing

Local [fixtures](tests/fixtures) check expected output and minification
stability; they do not compare runtime behavior or require external suites.
Each case has `input.js`, a manually maintained `output.js`, and optional
`options.json` for source type and minifier options. The stability check
minifies the output again without mangling, because names are chosen before
compression removes references.

Run from the workspace root:

```sh
cargo test -p swc_next_ecma_minifier --locked
cargo test -p swc_next_ecma_minifier --test fixtures -- preserve/script
```

## Output size

The [minify example](examples/minify.rs) minifies scripts with the default
options and checks that each output parses again:

```sh
cargo run --release -p swc_next_ecma_minifier --example minify -- target/minified tasks/benchmark/files/*.js
```

The [Minifier Output Size workflow](../../.github/workflows/minifier-size.yml)
compares the raw, gzip, and Brotli output sizes of the benchmark scripts between
the base and head revisions. It comments on a pull request or a pushed `main`
commit only when a size changed. When a later push removes the change, it updates
the existing pull request comment.
