/*---
description: Unicode decimal backreferences reject oversized indices
negative:
  phase: parse
  type: SyntaxError
---*/
const re = /()\18446744073709551616/u;
