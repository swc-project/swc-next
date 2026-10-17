/*---
description: TypeScript static blocks reject var and function merges
flags: [module]
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
