/*---
description: Import type qualifiers private name
flags: [module]
negative:
  phase: parse
  type: SyntaxError
---*/

type A = import("x").#a;
