---
title: Semantic analysis
description: Resolve declarations and references across scopes.
---

# Semantic analysis

Semantic analysis tracks scopes, declarations, and references, so tools can determine which binding a name refers to. It also reports early errors, but does not perform TypeScript type checking.

## Example

```js title="Input"
const value = 1;
function read(value) {
  return value;
}
console.log(value);
```

The two uses of `value` resolve to different declarations:

| Reference | Declaration |
| --- | --- |
| `value` in `return value` | The parameter of `read`. |
| `value` in `console.log(value)` | The outer `const value = 1`. |
