/*---
description: TypeScript static blocks reject var and function merges
negative:
  phase: early
  type: SyntaxError
---*/
class C {
  static {
    function f() {}
    var f;
  }
}
