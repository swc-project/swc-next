/*---
description: Decimal backreference indices do not wrap around the capture count
negative:
  phase: parse
  type: SyntaxError
---*/
const re = /()\4294967297/u;
