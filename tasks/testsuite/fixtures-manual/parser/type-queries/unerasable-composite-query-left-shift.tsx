/*---
description: Composite type queries diagnose left shifts that change satisfies grouping after erasure.
negative:
  phase: parse
  type: SyntaxError
---*/
one < two satisfies A | typeof f << other;
