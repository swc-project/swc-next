/*---
description: Functions conflict with preceding var declarations in block scopes
negative:
  phase: early
  type: SyntaxError
---*/
{
  var f;
  function f() {}
}
