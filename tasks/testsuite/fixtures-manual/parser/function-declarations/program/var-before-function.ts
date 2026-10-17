/*---
description: TypeScript programs reject var and function merges
negative:
  phase: early
  type: SyntaxError
---*/
var f;
function f() {}
