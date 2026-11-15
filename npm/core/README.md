# @swc-next/core

The Node.js API for SWC Next, a fast JavaScript and TypeScript toolchain written in Rust. Parse source, traverse and edit ESTree-based ASTs, and generate code.

## Documentation

See the [Core API documentation](https://next.swc.rs/api/javascript/core) for details.

## Building and testing

From the workspace root, run `pnpm build --release` for native bindings, then
`pnpm build:js` for the JavaScript entry points with Rslib. Use
`pnpm build:wasm --profile wasm-release` for WebAssembly bindings. Sources live
in `src-js`; generated JavaScript, copied loaders and declarations, and
native/Wasm artifacts go to `dist`. Internal
protocols are bundled into `dist/index.js`.

Run `pnpm test` for native API tests or `pnpm test:wasm` after building the
WebAssembly bindings. Both commands rebuild the JavaScript entry points.
