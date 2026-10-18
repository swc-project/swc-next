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
use numeric IDs. Group IDs become Symbols within each decode call. Scope endings
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

Most instructions map directly to Prettier's public Doc builders:

| Rust document                    | Prettier document                                              |
| -------------------------------- | -------------------------------------------------------------- |
| Text                             | Strings, with raw CR/LF converted to literal lines             |
| Space                            | Deferred whitespace, emitted as a string                       |
| Line / soft line                 | `line` / `softline`                                            |
| Hard / literal line              | `hardlineWithoutBreakParent` / `literallineWithoutBreakParent` |
| Group                            | `group`, preserving the computed expansion state               |
| Indent, absolute indent, align   | `indent`, root-relative alignment, `align`                     |
| Conditional content / indent     | `ifBreak` / `indentIfBreak`                                    |
| Display / canonical alternatives | `conditionalGroup` (see limitations below)                     |
| Fill entries and separators      | Alternating `fill` parts                                       |
| Line suffix scope                | `lineSuffix(contents)`, preserving its complete Doc            |
| Line suffix boundary             | `lineSuffixBoundary`                                           |
| Shared / flat fragment           | Cached normal or flattened Doc references                      |

Line suffixes add no whitespace or parent break of their own. Producers include
the desired spaces and `breakParent` explicitly in the Doc. Suffixes retain their
insertion mode and indentation when flushed, including when they contain nested
layout instructions. A suffix boundary flushes pending suffixes with a hard line;
without pending suffixes it leaves the layout unchanged. Fit measurement rejects
a flat layout that would flush pending suffixes at a boundary.

Space needs special handling because the native printer defers it until content
is written. The decoder keeps a pending-space state to
merge consecutive spaces and discard spaces before hard lines. Structural scopes
carry pending spaces through to their next text. Conditional branches preserve
the incoming gap when they do not print. Conditional gaps remain pending across
soft lines when their logical group stays flat, so fitting does not count a
space that a broken branch will discard. Unconditional gaps retain their fit
width because the transport currently uses the same instruction for deferred
and reserved spaces.

Conditions on the nearest logical group use Prettier's current mode while that
mode still belongs to the group. Fill entries and alternative candidates can
choose their own mode, so conditions within them explicitly reference the
logical group. Deferred gaps also retain that binding because they can be
consumed after leaving the group. This lets Prettier evaluate current-group
conditions during fitting, before a named group has been printed.

Group expansion distinguishes a source preference from a required break.
Canonical alternatives use a cached flat projection for fitting, removing only
source preferences, then render the original candidate with its source layout.
Required breaks remain active in both views. The projection uses an explicit
work stack and shares completed subdocuments with the decoded document graph.

Shared fragments receive the incoming whitespace state and return the outgoing
state. Anonymous groups keep the identity of their encoded scope. A linear
analysis of the fragment graph finds free nearest-group dependencies, so only
fragments that observe their caller's group include it in their cache key. The
remaining key records the incoming gap, raw trailing whitespace, and whether the
current mode belongs to the group. This preserves graph sharing across layout
alternatives while retaining the context needed by conditions. Explicit work
stacks handle dependency analysis and deeply nested references.

Line Docs absorb a preceding pending gap. Flat fragments use the native IR's
flattening rules before encoding. Both encoders and decoders use explicit work
lists for fragment references.

The adapter emits only public Prettier Doc nodes. The native IR's canonical fit
measurement, flat-width bounds, and forced-flat named groups are not all directly
expressible with those builders. The adapter uses Prettier's measurement and
group selection, so exact width boundaries can differ from the native printer.
Conditions on unprinted named groups have no host mode, and repeated shared
canonical choices can retain a mode from an earlier occurrence.
Prettier also normalizes input line endings before calling the plugin. Raw
trailing spaces, including Markdown hard breaks in comments, use literal line Docs with
the active indentation marked as their root. Differences are retained in
validation reports. The plugin does not promise
byte-for-byte native output parity.

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
