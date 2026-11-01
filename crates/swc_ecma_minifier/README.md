# swc_ecma_minifier

JavaScript AST optimization for SWC Next. Compression folds constant
expressions, shortens booleans and unshadowed global constants, simplifies safe
local `typeof` comparisons, and removes unused local variables.

Use `Minifier::minify(&mut ast, options)` on a valid JavaScript Program AST.
Parsing, TypeScript/JSX lowering, and printing are separate operations. Print
with the code generator's `minify` option for compact output, and recompute
semantic analysis before querying the optimized AST.

See [compression options](src/options.rs) for configuration. Name mangling is
not implemented.

## Testing

Local [fixtures](tests/fixtures) check expected output and minification
stability; they do not compare runtime behavior or require external suites.
Each case has `input.js`, a manually maintained `output.js`, and optional
`options.json` for source type and minifier options.

Run from the workspace root:

```sh
cargo test -p swc_next_ecma_minifier --locked
cargo test -p swc_next_ecma_minifier --test fixtures -- preserve/script
```
