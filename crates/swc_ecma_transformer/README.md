# swc_ecma_transformer

AST transformations for SWC Next, including TypeScript lowering and class
static blocks. See the [transformer API](src/lib.rs) for options.

## Usage

Transform and print a file from the workspace root:

```sh
cargo run -p swc_next_ecma_transformer --example transformer -- input.ts
```

The example lowers TypeScript and class static blocks while preserving JSX.

## Testing

```sh
cargo test -p swc_next_ecma_transformer --locked
```

For Babel, TypeScript, and SWC fixture coverage, see [conformance testing](../../tasks/testsuite/README.md).
