---
title: Transformer
description: Transform JavaScript, TypeScript, and JSX source synchronously.
---

# Transformer

`@swc-next/transformer` transforms source text and returns generated code,
diagnostics, and an optional source map. It lowers TypeScript and JSX using
the default transformation settings.

```ts
import { transformSync, type TransformApiOptions } from "@swc-next/transformer";

const options = {
  filename: "view.tsx",
  sourceMap: true,
} satisfies TransformApiOptions;

const result = transformSync("export const view = <div />;", options);
console.log(result.code);
```

`transformSync` and its option types are also exported by `@swc-next/core`.
In browsers, use `@swc-next/core-browser` after awaiting its default `init()` export.

The package uses ESM and is ready after import in Node.js. Native and Wasm bindings
provide the same API. Calls are synchronous.

## API

```ts
function transformSync(
  source: string,
  options?: TransformApiOptions | null,
): TransformOutput;
```

Options default to an empty object. Unknown options and invalid values throw an
error. Source diagnostics are returned in the result. Parser and semantic errors
stop processing and return an empty `code` string.

| Result field | Type | Description |
| --- | --- | --- |
| `code` | `string` | Generated source |
| `map` | `string`, optional | JSON source map, present when `sourceMap` is enabled |
| `diagnostics` | `Diagnostic[]` | Errors, warnings, hints, and informational messages |

Diagnostic `start`, `end`, and related label offsets use UTF-16 code units in the
original source. The fields match the [parser diagnostic shape](/api/javascript/parser#diagnostics-and-positions).

## Options

| Option | Description |
| --- | --- |
| `filename` | Infers language and source type; used for JSX metadata and source maps |
| `sourceMap` | Generate a source map; defaults to `false` |
| `comments` | `"all"` (default) or `"none"` |
| `parser` | [Parser options](/api/javascript/parser#parseapioptions), including explicit `lang` and `sourceType` overrides |
| `generator` | [Generator options](/api/javascript/generator#generateapioptions) |

The language defaults to `"js"`. A filename such as `input.ts`, `view.tsx`, or
`input.cjs` infers the corresponding language and source type. Explicit parser
options take precedence. `parser.comments` controls comment collection and
`comments` controls printing.

TypeScript lowering is enabled for `ts`, `tsx`, and `dts`. JSX lowering is enabled
for `jsx` and `tsx` and uses the automatic runtime by default. Transformation
settings are not configurable through the JavaScript API.

## Limitations

This API preserves module formats and does not load `.swcrc` files. It does not
provide target-based lowering for arbitrary ECMAScript features, minification
passes, or SWC plugin execution. `generator.minify` only prints compact code.
