---
title: Parser
description: parseSync inputs, parser options, results, and diagnostics.
---

# Parser

`parseSync` parses JavaScript, TypeScript, and JSX into an ESTree-based program. It is exported by `@swc-next/core`, `@swc-next/parser`, and the initialized `@swc-next/core-browser` package.

## Signature

```ts
declare function parseSync(
  source: string,
  options?: ParserOptions | null,
): ParseResult;
```

`source` is the complete source text. Omitting `options` or passing `null` uses the defaults below. Parsing is synchronous. By default, the first diagnostic is thrown; enable `errorRecovery` to return diagnostics and a recovered AST.

## ParserOptions

| Option | Type | Default | Behavior |
| --- | --- | --- | --- |
| [`lang`](#lang) | `"js" \| "ts" \| "jsx" \| "tsx" \| "dts"` | `"js"` | Select the input syntax. |
| [`sourceType`](#sourcetype) | `"module" \| "script" \| "commonjs" \| "unambiguous"` | `"module"` | Select module, script, or CommonJS parsing rules, or detect them from the source. |
| [`preserveParens`](#preserveparens) | `boolean` | `true` | Keep explicit nodes for parenthesized expressions and types. |
| [`comments`](#comments) | `"none" \| "flat"` | `"flat"` | Collect comments in a separate list, or skip collection. |
| [`errorRecovery`](#errorrecovery) | `boolean` | `false` | Return diagnostics and a recovered AST instead of throwing the first diagnostic. |
| [`semanticErrors`](#semanticerrors) | `boolean` | `false` | Also check semantic early errors that require an AST pass. |

Omitted fields use their defaults. Invalid option values, such as `lang: "javascript"` or `preserveParens: 1`, throw a `TypeError`.

### lang

Type: `"js" | "ts" | "jsx" | "tsx" | "dts"`

Default: `"js"`

Choose the syntax accepted by the parser:

| Value | Syntax |
| --- | --- |
| `"js"` | JavaScript without JSX or TypeScript. |
| `"ts"` | TypeScript without JSX. |
| `"jsx"` | JavaScript with JSX. |
| `"tsx"` | TypeScript with JSX. |
| `"dts"` | TypeScript declaration files, such as `.d.ts`, using ambient declaration rules. |

```js
import { parseSync } from "@swc-next/core";

const typed = parseSync("const answer: number = 42;", { lang: "ts" });
const element = parseSync("const view = <div />;", { lang: "jsx" });
const declarations = parseSync("export const answer: number;", { lang: "dts" });
```

TypeScript annotations and JSX remain in the AST. Selecting a language does not strip types, transform JSX, or type-check the source. Select `sourceType` separately; for file-based input, use [`langFromPath`](#path-helpers) to choose the language from the extension.

### sourceType

Type: `"module" | "script" | "commonjs" | "unambiguous"`

Default: `"module"`

Choose the rules for the file's top level:

| Value | Behavior |
| --- | --- |
| `"module"` | Parse an ES module with strict mode, static `import` / `export`, `import.meta`, and top-level `await`. |
| `"script"` | Parse a classic script. Strict mode applies when enabled by a `"use strict"` directive; static imports and exports are not allowed. |
| `"commonjs"` | Parse with CommonJS wrapper rules, allowing top-level `return` and `new.target`. Strict mode is not enabled automatically. |
| `"unambiguous"` | Detect module syntax first, then CommonJS wrapper syntax, and otherwise use script rules. The resolved value is returned in `program.sourceType` and `meta.sourceType`. |

Detection is based on syntax, rather than calls to `require` or assignments to `module.exports`. A dynamic `import()` alone also does not select module mode.

```js
import { parseSync } from "@swc-next/core";

parseSync("return 42;", { sourceType: "commonjs" }).diagnostics;
// []

parseSync("export const answer = 42;", { sourceType: "unambiguous" }).meta.sourceType;
// "module"

parseSync("return 42;", { sourceType: "unambiguous" }).meta.sourceType;
// "commonjs"

parseSync("const answer = 42;", { sourceType: "unambiguous" }).meta.sourceType;
// "script"
```

This option controls parsing only; it does not convert a module to CommonJS or resolve imports. Use [`sourceTypeFromPath`](#path-helpers) when a file extension specifies the source type.

### preserveParens

Type: `boolean`

Default: `true`

Keep parentheses as `ParenthesizedExpression` nodes and, for TypeScript types, `TSParenthesizedType` nodes. With `false`, keep only the inner expression or type, matching typescript-eslint's representation.

```js
import { parseSync } from "@swc-next/core";

const preserved = parseSync("(answer);", { preserveParens: true });
preserved.program.body[0].expression.type;
// "ParenthesizedExpression"
preserved.program.body[0].expression.expression.type;
// "Identifier"

const unwrapped = parseSync("(answer);", { preserveParens: false });
unwrapped.program.body[0].expression.type;
// "Identifier"
```

This changes the tree's shape, not expression precedence. The printer inserts parentheses required to preserve the AST's meaning even when explicit parenthesized nodes were omitted.

### comments

Type: `"none" | "flat"`

Default: `"flat"`

With `"flat"`, return every collected comment in source order in `result.comments`. With `"none"`, skip collection and return an empty array. Comments are separate from the program and are not attached to AST nodes.

```js
import { parseSync } from "@swc-next/core";

const source = "// header\nanswer; /* tail */";
parseSync(source, { comments: "flat" }).comments;
// [
//   { type: "Line", value: " header", start: 0, end: 9 },
//   { type: "Block", value: " tail ", start: 18, end: 28 },
// ]

parseSync(source, { comments: "none" }).comments;
// []
```

`value` excludes the comment delimiters, while `start` and `end` include them. Offsets use UTF-16 code units and the end is exclusive. A hashbang such as `#!/usr/bin/env node` is stored separately in `program.hashbang`.

Collecting comments does not make [`printSync`](/api/javascript/codegen) emit them: its JavaScript interface takes the program and metadata, without the flat comment list.

### errorRecovery

Type: `boolean`

Default: `false`

When parsing produces diagnostics, the default behavior throws the first `Diagnostic` object. Set `errorRecovery: true` to return the AST and diagnostics instead. The AST may be incomplete for invalid input. This also applies to semantic diagnostics when `semanticErrors` is enabled.

### semanticErrors

Type: `boolean`

Default: `false`

Run an additional AST pass and append semantic early errors to `result.diagnostics`. These include conflicting declarations in the same scope and exports of undeclared local names. Syntax diagnostics are reported regardless of this setting, and semantic checks also run on a recovered AST after syntax errors.

```js
import { parseSync } from "@swc-next/core";

const source = "let answer; let answer;";
parseSync(source, { semanticErrors: false }).diagnostics.length;
// 0

parseSync(source, { semanticErrors: true, errorRecovery: true }).diagnostics.length;
// 1
```

Enabling this option adds diagnostics without changing the AST. It does not expose the full [semantic model](/api/components/semantic) or perform TypeScript type checking, such as checking whether a value matches its annotated type.

## ParseResult

```ts
interface ParseResult {
  readonly program: Program;
  readonly meta: AstMeta;
  readonly comments: Comment[];
  readonly diagnostics: Diagnostic[];
}
```

| Field | Contents |
| --- | --- |
| `program` | The ESTree-based program. Its nodes can be inspected and edited. |
| `meta` | `{ lang, sourceType }`; keep it with the AST and pass it to [printSync](/api/javascript/codegen#metadata). |
| `comments` | Flat comments with `type: "Line" \| "Block"`, delimiter-free `value`, `start`, and `end`. Empty when collection is disabled. |
| `diagnostics` | Syntax diagnostics and, with `semanticErrors`, semantic early errors. Check these before using recovered output. |

The program and comments are decoded lazily when accessed. Diagnostics are decoded lazily when `errorRecovery` is enabled. See [AST types](/api/javascript/types) for node declarations.

## Diagnostics and positions

A `Diagnostic` contains:

| Field | Type | Meaning |
| --- | --- | --- |
| `message` | `string` | Description of the issue. |
| `severity` | `"error" \| "warning" \| "hint" \| "info"` | Severity of the diagnostic. |
| `start`, `end` | `number` | UTF-16 offsets in the input source. |
| `help` | `string`, optional | Additional guidance. |
| `labels` | `DiagnosticLabel[]` | Related spans, each with a `message`, `start`, and `end`. |

AST nodes and comments also use UTF-16 offsets. End offsets are exclusive, so `source.slice(start, end)` selects the corresponding span.

## Path helpers

```ts
declare function langFromPath(path: string): Lang;
declare function sourceTypeFromPath(path: string): SourceType;
```

These helpers are exported alongside `parseSync`; they do not read the file. Pass the returned values as parser options:

```js
import { langFromPath, parseSync, sourceTypeFromPath } from "@swc-next/core";

const path = "input.ts";
const { program } = parseSync("const answer: number = 42;", {
  lang: langFromPath(path),
  sourceType: sourceTypeFromPath(path),
});
```

Language and source type options accept strings; their TypeScript names are not runtime enums.

`langFromPath` checks the following suffixes in order:

| Path suffix | Language |
| --- | --- |
| `.d.ts`, `.d.mts`, `.d.cts` | `"dts"` |
| `.tsx` | `"tsx"` |
| `.ts`, `.mts`, `.cts` | `"ts"` |
| `.jsx` | `"jsx"` |
| Any other suffix | `"js"` |

`sourceTypeFromPath` returns `"module"` for `.mjs` and `.mts`, `"commonjs"` for `.cjs` and `.cts`, and `"unambiguous"` for all other paths. Both helpers use case-sensitive suffix matching and do not inspect `package.json`.
