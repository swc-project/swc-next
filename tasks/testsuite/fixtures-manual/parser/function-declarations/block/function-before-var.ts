/*---
description: Var declarations conflict with preceding functions in block scopes
negative:
  phase: early
  type: SyntaxError
---*/
{
  function f() {}
  var f;
}
