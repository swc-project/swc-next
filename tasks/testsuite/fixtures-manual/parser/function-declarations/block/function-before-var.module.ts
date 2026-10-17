/*---
description: Var declarations conflict with preceding functions in block scopes
flags: [module]
negative:
  phase: early
  type: SyntaxError
---*/
{
  function f() {}
  var f;
}
