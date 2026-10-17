# @swc-next/formatter

Internal, version-matched native/Wasm document transport for
`@swc-next/prettier-plugin-swcfmt`. Uses the shared SWC bridge and the Rust formatter;
it does not depend on `@swc-next/core`.

`formatDocBufferSync` returns an unprinted binary document or structured
formatting diagnostic. Its option records and document protocol are private ABI,
not a public source-formatting API. Use the Prettier plugin for formatting.

Build native: `pnpm --filter @swc-next/formatter build`.
Build Wasm: `pnpm --filter @swc-next/formatter build:wasm`.
