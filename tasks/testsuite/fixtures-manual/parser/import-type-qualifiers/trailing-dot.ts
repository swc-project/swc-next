/*---
description: Import type qualifiers trailing dot
flags: [module]
negative:
  phase: parse
  type: SyntaxError
---*/

type A = import("x").A.;
