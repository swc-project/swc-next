/*---
description: Var declarations conflict with preceding functions in catch scopes
negative:
  phase: early
  type: SyntaxError
---*/
try {} catch {
  function f() {}
  var f;
}
