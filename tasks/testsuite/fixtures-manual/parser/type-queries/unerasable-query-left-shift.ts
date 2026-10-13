/*---
description: Type queries diagnose left shifts that change relational grouping after assertion erasure.
negative:
  phase: parse
  type: SyntaxError
---*/
one < two as typeof f << other;
