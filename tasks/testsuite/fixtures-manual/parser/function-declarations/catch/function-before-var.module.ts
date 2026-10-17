/*---
description: Var declarations conflict with preceding functions in catch scopes
flags: [module]
negative:
  phase: early
  type: SyntaxError
---*/
try {} catch {
  function f() {}
  var f;
}
