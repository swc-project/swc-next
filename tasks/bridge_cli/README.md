# Bridge CLI

Builds Wasm components and generates their JavaScript bindings.

For a custom output directory, run from the workspace root:

```sh
cargo run --locked -p bridge_cli -- build core --output-dir target/wasm-package/core --profile wasm-release
```

The `wasm32-unknown-unknown` Rust target is required. Replace `core` with
`parser`, `codegen`, or `playground` for another component. Trailing Cargo build
options, including `--profile` and `--target-dir`, are forwarded to the Wasm build.

This generates bindings and Wasm files; [assembling the browser package](../../npm/core-browser/README.md#building)
is a separate operation.
