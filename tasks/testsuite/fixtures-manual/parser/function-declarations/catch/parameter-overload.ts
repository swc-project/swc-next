/*---
description: Overload implementations conflict with catch parameters
negative:
  phase: early
  type: SyntaxError
---*/
try {} catch (f) {
  function f(): void;
  function f() {}
}
