---
title: Transformer
description: Lower supported TypeScript and JavaScript syntax.
---

# Transformer

The transformer rewrites supported syntax into JavaScript, including TypeScript constructs and class static blocks. Unlike type stripping, it can generate runtime code for TypeScript features such as parameter properties.

## Example

```ts title="Input"
class Point {
  constructor(public x: number) {}
}
```

```js title="Output"
class Point {
  x;
  constructor(x) {
    this.x = x;
  }
}
```
