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

`source` is the complete source text. Omitting `options` or passing `null` uses the defaults below. Parsing is synchronous and can return a recovered AST with diagnostics.

## ParserOptions

| Option | Type | Default | Behavior |
| --- | --- | --- | --- |
| `lang` | `"js" \| "ts" \| "jsx" \| "tsx" \| "dts"` | `"js"` | Select the input syntax. Parsing preserves TypeScript and JSX syntax. |
| `sourceType` | `"module" \| "script" \| "commonjs" \| "unambiguous"` | `"module"` | Select the source type, or infer it with `unambiguous`. |
| `preserveParens` | `boolean` | `true` | Preserve parenthesized expressions and types as explicit nodes. |
| `comments` | `"none" \| "flat"` | `"flat"` | Disable comment collection or return a source-ordered comment list. |
| `semanticErrors` | `boolean` | `false` | Run an additional AST pass for semantic early errors, including after syntax recovery. |

`semanticErrors` adds diagnostics; it does not expose the full [semantic model](/api/components/semantic) or perform TypeScript type checking.

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

The program, comments, and diagnostics are decoded lazily when accessed. See [AST types](/api/javascript/types) for node declarations.

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
