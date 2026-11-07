## Prettier Plugin

The main idea is to build formatting documents in Rust and let Prettier render
them. Rust sends a compact binary document to JavaScript, where the plugin
reconstructs a Prettier Doc.

The integration uses Prettier's [plugin API](https://prettier.io/docs/plugins).

- **Pros:** Reuses the SWC formatter without transferring the AST to JavaScript.
- **Cons:** Requires Doc conversion and separate support for features that depend
  on an AST, such as range formatting and embedded languages.

### Detailed Design

There are three parts: the Rust formatter, a binary transport, and the Prettier
plugin. The document model and native printer remain in the formatter crate.

#### Plugin API

`@swc-next/prettier-plugin-swcfmt` formats source into a Doc. It exports
`languages`, `parsers`, `printers`, and `options` through an ESM entry point.

```js
import * as prettier from "prettier";
import * as swc from "@swc-next/prettier-plugin-swcfmt";

const code = await prettier.format(source, {
  plugins: [swc],
  parser: "swc-ts",
  printWidth: 100,
});
```

| Parser    | Language         | File extensions                                    |
| --------- | ---------------- | -------------------------------------------------- |
| `swc`     | JavaScript / JSX | `.js`, `.jsx`, `.mjs`, `.cjs`                      |
| `swc-ts`  | TypeScript       | `.ts`, `.mts`, `.cts`, including declaration files |
| `swc-tsx` | TSX              | `.tsx`                                             |

The plugin can infer the language from `filepath`. Explicit parser names select
the grammar and do not override Prettier's built-in parsers. Module extensions
(`.mjs`, `.mts`, including `.d.mts`) select module mode; `.cjs` and `.cts`
select CommonJS mode. Other extensions use unambiguous parsing.

`@swc-next/prettier-plugin-swc-parser` is reserved for a possible future plugin
that supplies an AST to Prettier's AST printer. It is not implemented.

#### Formatting Pipeline

The parser calls Rust through `@swc-next/formatter`, an independent binding with
native and Wasm support. Neither package depends on `@swc-next/core`.

```text
Source → Rust parser → SWC formatter → binary Doc
       → JavaScript decoder → Prettier Doc → formatted text
```

The plugin uses `SWCFMT` as both its `astFormat` and printer key. Its parser
returns a small root containing the decoded Doc and the source range:

```js
{ type: "SWCFMT", start: 0, end: source.length, doc }
```

The printer simply returns `path.node.doc`. Rust handles comments, ignore
directives, and parentheses, so the root has no comments or visitor keys.
Positions on the root use UTF-16 offsets.

Prettier handles the final layout, line endings, and BOM. Rust also receives
`printWidth`, `tabWidth`, and the resolved `endOfLine` because these options
affect document construction, including JSX attribute entity encoding.

#### Binary Transport

The transport follows the same approach as the ESTree bridge: numeric tags,
string pools, and JavaScript object reconstruction from a single buffer.
Compression here means compact encoding and deduplication, without JSON or a
separate compression codec.

A successful response has the following layout:

```text
"SWCD"
version: u32 = 0
fragmentCount: u32
stringBytes: u32
fragments: [instructionBytes: u32, instructions] × fragmentCount
UTF-8 string pool
```

Integers are little endian. Each instruction has a one-byte opcode followed by
its operands. Text uses an offset and length in the string pool; shared fragments
use numeric IDs. Named group IDs receive stable Symbols within each decode call.
Scope endings
identify the corresponding scope; normal, absolute, and conditional indents
share one ending instruction.

Each arena fragment is encoded once, and strings are deduplicated by content.
Rust returns owned bytes, so no source references, arena lifetimes, or native
pointers cross the bridge. The decoder checks bounds, versions, opcodes, scope
pairs, and reference cycles.

Errors use `SWCE`, followed by the version, UTF-8 byte offsets for the start and
end, and the message. JavaScript converts the offsets into UTF-16 line and column
positions for a `SyntaxError` with `loc`. Invalid call arguments use the bridge's
normal exception mechanism.

Formatter options use a separate 15-byte record (version 0), including a resolved
line-ending tag: 0 for LF, 1 for CRLF, and 2 for CR. Parser options reuse the
existing packed representation. `formatDocBufferSync` and the document protocol
are internal interfaces whose versions must match the plugin.

#### Document Conversion

The instructions map to Prettier's public Doc builders:

| Rust document | Prettier document |
| --- | --- |
| Single-line Text | String |
| Line / soft line | `line` / `softline` |
| Hard / literal line | `hardline` / `literalline` |
| Literal line with active indentation | `markAsRoot(literalline)` |
| Break parent | `breakParent` |
| Group | `group`, preserving the expansion request and optional ID |
| Indent, absolute indent, align | `indent`, root-relative alignment, `align` |
| Conditional content / indent | `ifBreak` / `indentIfBreak` |
| Alternatives | `conditionalGroup` |
| Fill entries and separators | Alternating `fill` parts |
| Line suffix scope / boundary | `lineSuffix` / `lineSuffixBoundary` |
| Shared / flat fragment | Cached normal or statically projected Doc references |

Ordinary Text cannot contain CR or LF. Source text with physical lines is
constructed as explicit literal lines, which follow the resolved output EOL.
Spaces are actual text; consecutive spaces and flat `line` nodes remain distinct.
Ordinary broken lines trim trailing horizontal whitespace, while literal lines
preserve it. Line suffixes carry their complete Doc, including any separating
space and parent break, and flush according to standard Prettier semantics.

Conditions explicitly target either the current execution mode or a named
group. Both full conditional branches remain visible to break propagation.
Named IDs map directly to stable Symbols, including IDs referenced before their
group has executed; Prettier owns their print and fit behavior. A fragment that
defines a named group cannot be forced flat, so conversion requires no dynamic
binding emulation or extra width-dependent groups.

The encoder's flat projection removes eligible anonymous layout scopes and
projects their contents before transport. Named conditional indent scopes keep
their condition while projecting their body. The decoder builds each fragment
once in dependency order and reuses the resulting Doc. Both sides use explicit
work lists, preserving a shared DAG without recursion through references.

The protocol has no deferred whitespace, canonical fitting view, inherited
group, or forced-flat named binding operation. Version 1 rejects the old tags
instead of silently approximating them. Prettier handles all final layout
choices; language formatting policy remains in Rust's producers. Validation
compares native and Prettier output with identical normalized source and options.

### Current Scope

The plugin is experimental and currently pins Prettier to 3.9.9. It reuses
Prettier's JavaScript option declarations and defaults, with `printWidth` limited
to 0–65535 and `tabWidth` to 0–64.

The following options are not supported:

- Partial ranges and pragma options.
- `objectWrap: "collapse"`.
- `experimentalTernaries` and `experimentalOperatorPosition: "start"`.

`quoteProps: "consistent"` and `singleAttributePerLine` are supported by the
current native formatter and forwarded by the plugin.

Embedded languages retain their source text; there is no `embed` hook. Cursor
tracking uses Prettier's whole-document fallback. Plugins that expect an ESTree
AST cannot consume the `SWCFMT` root. Browser standalone support is not packaged.

Output follows the SWC formatter's existing differences and idempotency limits.
Using Prettier's Doc printer does not guarantee identical output to its built-in
JavaScript formatter. Performance claims require measurements of encoding,
decoding, allocation, and layout costs.

### Building and Testing

```sh
pnpm install
pnpm --filter @swc-next/formatter build
pnpm --filter @swc-next/prettier-plugin-swcfmt test
pnpm --filter @swc-next/formatter build:wasm
pnpm --filter @swc-next/prettier-plugin-swcfmt test:wasm
```

Tests live in `packages/prettier-plugin/test`. They cover formatting, options,
source locations, malformed buffers, and whitespace across shared fragments.
The same tests run against native and Wasm bindings.
They discover every local Rust formatter fixture and its nearest `options.json`,
normalize the input as the pinned Prettier host does, and compare exact strings
with a test-only N-API call to native `format()`. The oracle uses the same fixture
option decoder as the Rust tests, independent of the plugin's option and Doc
transport and returns formatted strings directly. No output EOL or EOF
normalization is applied, and presentation snapshots are not used as an oracle.
Tests also check the input-normalization boundary against Prettier itself.
The native oracle addon is built automatically with the `test-native` feature
into `npm/formatter/test/binding`. Tests load that `.node` file directly,
including when the plugin binding is forced to Wasm.
