/*---
description: Import type qualifiers template name
flags: [module]
negative:
  phase: parse
  type: SyntaxError
---*/

type A = import("x").`a`;
