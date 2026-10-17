/*---
description: Functions conflict with parameters in separate catch scopes
negative:
  phase: early
  type: SyntaxError
---*/
try {} catch ([f = 1]) {
  function f() {}
}
