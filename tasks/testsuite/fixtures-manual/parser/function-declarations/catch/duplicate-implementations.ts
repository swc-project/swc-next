/*---
description: Strict lexical scopes reject duplicate function implementations
flags: [onlyStrict]
negative:
  phase: early
  type: SyntaxError
---*/
try {} catch {
  function f() {}
  function f() {}
}
