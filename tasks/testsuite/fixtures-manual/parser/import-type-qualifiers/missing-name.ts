/*---
description: Import type qualifiers missing name
flags: [module]
negative:
  phase: parse
  type: SyntaxError
---*/

type A = import("x").;
