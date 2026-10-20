/*---
description: Import type qualifiers typeof missing name
flags: [module]
negative:
  phase: parse
  type: SyntaxError
---*/

type A = typeof import("x").
