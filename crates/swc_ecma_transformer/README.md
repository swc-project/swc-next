# swc_ecma_transformer

AST transformations for SWC Next, including TypeScript lowering and class
static blocks. See the [transformer API](src/lib.rs) for options.

## Usage

Transform and print a file from the workspace root:

```sh
cargo run -p swc_next_ecma_transformer --example transformer -- input.ts
```

The example lowers TypeScript and class static blocks while preserving JSX.

Modules that need runtime helpers emit named imports from
[`@swc-next/helpers`](../../packages/helpers). Install that package when
running the generated modules. Scripts embed the helpers they use.

Set `TransformerOptions::helpers` to `HelperMode::Inline` to embed the required
helpers instead. Inline helpers are parsed from the shared package sources;
their bindings and scopes are registered in the transform's semantic model.
The example exposes this option as `--inline-helpers`:

```sh
cargo run -p swc_next_ecma_transformer --example transformer -- --inline-helpers input.ts
```

## Testing

```sh
cargo test -p swc_next_ecma_transformer --locked
```

For Babel, TypeScript, and SWC fixture coverage, see [conformance testing](../../tasks/testsuite/README.md).
