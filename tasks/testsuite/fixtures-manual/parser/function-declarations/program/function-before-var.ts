/*---
description: TypeScript programs reject var and function merges
negative:
  phase: early
  type: SyntaxError
---*/
function f() {}
var f;
