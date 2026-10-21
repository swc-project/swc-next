---
title: Codegen
description: printSync AST inputs, metadata, formatting options, and output.
---

# Codegen

`printSync` generates source text from a program AST. It is exported by `@swc-next/core`, `@swc-next/codegen`, and the initialized `@swc-next/core-browser` package.

## Signature and inputs

```ts
declare function printSync(
  program: Program,
  options?: CodegenOptions | null,
  meta?: Partial<AstMeta>,
): CodegenOutput;
```

| Input | Meaning |
| --- | --- |
| `program` | A valid program AST, usually returned by `parseSync` and optionally edited with `walk`. |
| `options` | Formatting options. Omit or pass `null` for defaults. |
| `meta` | Optional language and source type metadata from the parser. |

Input nodes follow the [AST declarations](/api/javascript/types) shared with the parser.

## CodegenOptions

| Option | Type | Default | Behavior |
| --- | --- | --- | --- |
| `asciiOnly` | `boolean` | `false` | Escape non-ASCII characters where supported by the printer. |
| `inlineScript` | `boolean` | `false` | Escape closing script tags; tagged template raw text is preserved. |
| `minify` | `boolean` | `false` | Generate compact output with reduced whitespace. |

`minify` does not run compression or name mangling. Those capabilities belong to the separate [Minifier component](/api/components/minifier).

## Metadata

```ts
interface AstMeta {
  readonly lang: "js" | "jsx" | "ts" | "tsx" | "dts";
  readonly sourceType: "module" | "script" | "commonjs" | "unambiguous";
}
```

Pass `parseSync(...).meta` to preserve the input language and source type. Explicit metadata takes precedence; when omitted, source type comes from the program and language is inferred from the tree. Printing does not remove TypeScript types or lower JSX.

## CodegenOutput

```ts
interface CodegenOutput {
  code: string;
}
```

The JavaScript interface returns generated code only. It does not expose source maps or comment emission options; the parser's separate comment list is not passed to the printer.
