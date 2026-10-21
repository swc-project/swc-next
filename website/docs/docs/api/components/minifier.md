---
title: Minifier
description: Compress JavaScript expressions and remove unused local variables.
---

# Minifier

The minifier shortens expressions and removes unused local variables. Compression can be combined with compact printing; variable name mangling is not yet implemented.

## Example

```js title="Input"
function choose(value) {
  return value ? [true] : { flag: false };
}
```

```js title="Output"
function choose(value){return value?[!0]:{flag:!1}}
```
