# @swc-next/formatter

Internal, version-matched native/Wasm document transport for
`@swc-next/prettier-plugin-swcfmt`. Uses the shared SWC bridge and the Rust formatter;
it does not depend on `@swc-next/core`.

`formatDocBufferSync` returns an unprinted binary document or structured
formatting diagnostic. Its option records and document protocol are private ABI,
not a public source-formatting API. Use the Prettier plugin for formatting.

Build native: `pnpm --filter @swc-next/formatter build`.
Build Wasm: `pnpm --filter @swc-next/formatter build:wasm`.

Plugin tests build a separate native addon with the `build:test` script. Its
`test-native` feature exposes `formatNativeSync`, returning the
native formatter's string directly with independently decoded fixture options.
The addon is loaded directly from `test/binding` even when testing the Wasm
plugin. This test feature and its generated binding are not part of the published API.
