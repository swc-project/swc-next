/*---
description: Functions conflict with preceding var declarations in catch scopes
negative:
  phase: early
  type: SyntaxError
---*/
try {} catch {
  var f;
  function f() {}
}
