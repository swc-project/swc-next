---
title: Isolated declarations
description: Generate TypeScript declaration files.
---

# Isolated declarations

Isolated declaration generation produces `.d.ts` output describing a module's public types. It retains declaration signatures and removes implementations, without full TypeScript type checking.

## Example

```ts title="Input"
export function greet(name: string): string {
  return "Hello, " + name;
}
```

```ts title="Output (.d.ts)"
export declare function greet(name: string): string;
```
