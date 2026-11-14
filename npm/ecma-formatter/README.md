# @swc-next/ecma-formatter

Internal, version-matched ECMAScript document producer for
`@swc-next/prettier-plugin-swcfmt`, with native and Wasm bindings.

`formatDocBufferSync` parses JavaScript, JSX, TypeScript, or TSX and returns an
unprinted SWCD document or a structured formatting diagnostic. Its option
records and document protocol are private ABI. Use the Prettier plugin for
source formatting, or `@swc-next/formatter` to print a document through Rust.

Build native: `pnpm --filter @swc-next/ecma-formatter build`.
Build Wasm: `pnpm --filter @swc-next/ecma-formatter build:wasm`.

Plugin tests use `build:test` to build the separate native
`formatNativeSync` oracle. That test-only entry is excluded from the published API.
