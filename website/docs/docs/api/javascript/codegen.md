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
| [`asciiOnly`](#asciionly) | `boolean` | `false` | Escape non-ASCII characters where supported by the printer. |
| [`inlineScript`](#inlinescript) | `boolean` | `false` | Escape closing script tags in supported contexts. |
| [`minify`](#minify) | `boolean` | `false` | Generate compact output with reduced whitespace. |

Omitted fields use their defaults. Non-boolean option values, such as `minify: 1`, throw a `TypeError`.

### asciiOnly

Type: `boolean`

Default: `false`

Use escape sequences for non-ASCII characters in identifiers, strings, and supported template text. With `false`, the printer can emit Unicode characters directly.

```js
import { parseSync, printSync } from "@swc-next/core";

const { program } = parseSync('const greeting = "你好";');
printSync(program, { asciiOnly: false }).code;
// 'const greeting = "你好";\n'

printSync(program, { asciiOnly: true }).code;
// 'const greeting = "\\u4F60\\u597D";\n'
```

Escaping is context-dependent. Regular expression bodies and tagged template raw text are preserved, so this option does not guarantee that the entire output contains only ASCII. It does not change string values at runtime.

### inlineScript

Type: `boolean`

Default: `false`

Escape case-insensitive `</script` sequences in strings and untagged templates, and insert separators when neighboring tokens would form such a sequence. This helps prevent generated JavaScript text from closing an HTML `<script>` element early.

```js
import { parseSync, printSync } from "@swc-next/core";

const { program } = parseSync('const text = "</script>";');
printSync(program, { inlineScript: false }).code;
// 'const text = "</script>";\n'

printSync(program, { inlineScript: true }).code;
// 'const text = "<\\/script>";\n'
```

The escape preserves the string's runtime value. Tagged template raw text is preserved because changing it would be observable by the tag function. This option therefore does not guarantee that arbitrary generated output can be embedded directly in HTML.

### minify

Type: `boolean`

Default: `false`

Use compact printing: remove optional whitespace, indentation, and line breaks, and choose shorter spellings where supported. Required separators remain so that adjacent tokens are parsed correctly. With `false`, print readable, indented code.

```js
import { parseSync, printSync } from "@swc-next/core";

const { program } = parseSync("const answer = 1 + 2;");
printSync(program, { minify: false }).code;
// 'const answer = 1 + 2;\n'

printSync(program, { minify: true }).code;
// 'const answer=1+2;'
```

`minify` does not evaluate `1 + 2`, remove unused declarations, or shorten variable names. Compression and name mangling belong to the separate [Minifier component](/api/components/minifier). Printing preserves TypeScript and JSX syntax with either setting.

## Metadata

```ts
interface AstMeta {
  readonly lang: "js" | "jsx" | "ts" | "tsx" | "dts";
  readonly sourceType: "module" | "script" | "commonjs" | "unambiguous";
}
```

Pass `parseSync(...).meta` to preserve the input language and source type. Explicit metadata takes precedence; when omitted, source type comes from the program and language is inferred from the tree. Printing does not remove TypeScript types or lower JSX.

| Field | Meaning |
| --- | --- |
| `lang` | Select the AST language variant used for encoding and printing. Pass the parser's value to retain its JavaScript, JSX, TypeScript, TSX, or declaration-file mode. |
| `sourceType` | Select the program's source type for printing. Pass the parser's resolved value to retain module, script, or CommonJS mode. |

```js
import { parseSync, printSync } from "@swc-next/core";

const result = parseSync("export const answer: number = 42;", { lang: "ts" });
const { code } = printSync(result.program, { minify: true }, result.meta);
```

## CodegenOutput

```ts
interface CodegenOutput {
  code: string;
}
```

The JavaScript interface returns generated code only. It does not expose source maps or comment emission options; the parser's separate comment list is not passed to the printer.
