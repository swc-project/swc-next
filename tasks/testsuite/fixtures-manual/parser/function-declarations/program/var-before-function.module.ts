/*---
description: TypeScript programs reject var and function merges
flags: [module]
negative:
  phase: early
  type: SyntaxError
---*/
var f;
function f() {}
