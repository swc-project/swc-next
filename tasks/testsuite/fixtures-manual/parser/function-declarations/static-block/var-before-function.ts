/*---
description: TypeScript static blocks reject var and function merges
negative:
  phase: early
  type: SyntaxError
---*/
class C {
  static {
    var f;
    function f() {}
  }
}
