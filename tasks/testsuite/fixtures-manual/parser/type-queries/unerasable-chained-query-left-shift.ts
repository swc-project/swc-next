/*---
description: Chained assertions preserve the erasure restriction before left shifts.
negative:
  phase: parse
  type: SyntaxError
---*/
one < two as unknown as typeof f << other;
