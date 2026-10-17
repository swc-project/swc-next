/*---
description: Functions conflict with preceding var declarations in block scopes
flags: [module]
negative:
  phase: early
  type: SyntaxError
---*/
{
  var f;
  function f() {}
}
