/*---
description: Functions conflict with preceding var declarations in catch scopes
flags: [module]
negative:
  phase: early
  type: SyntaxError
---*/
try {} catch {
  var f;
  function f() {}
}
