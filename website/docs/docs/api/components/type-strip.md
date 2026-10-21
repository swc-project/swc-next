---
title: Type strip
description: Remove erasable TypeScript syntax while preserving source positions.
---

# Type strip

Type stripping replaces erasable TypeScript syntax with spaces, preserving UTF-16 offsets and line breaks. Syntax that needs runtime code, such as enums, requires the [transformer](/api/components/transformer).

## Example

```ts title="Input"
let value: number = 1;
```

```js title="Output"
let value         = 1;
```
