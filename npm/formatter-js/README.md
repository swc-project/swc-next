# @swc-next/formatter-js

**Experimental and unstable.** APIs, document formats, and formatting output may
change without notice.

Internal, version-matched ECMAScript document producer for
`@swc-next/formatter-prettier-plugin`, with native and Wasm bindings.

`formatDocBufferSync` parses JavaScript, JSX, TypeScript, or TSX and returns an
unprinted SWCD document or a structured formatting diagnostic. Its option
records and document protocol are private ABI. Use the Prettier plugin for
source formatting, or `@swc-next/formatter` to print a document through Rust.

Build native: `pnpm --filter @swc-next/formatter-js build`.
Build Wasm: `pnpm --filter @swc-next/formatter-js build:wasm`.

Then run `pnpm --filter @swc-next/formatter-js build:js` to build the JavaScript
entry point in `dist`. Sources live in `src-js`; native and Wasm artifacts
also go to `dist`.
