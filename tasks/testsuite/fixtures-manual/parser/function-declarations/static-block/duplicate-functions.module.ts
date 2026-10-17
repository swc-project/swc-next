/*---
description: TypeScript static blocks retain JavaScript function implementation merge rules
flags: [module]
---*/
class C {
  static {
    function f() {}
    function f() {}
  }
}
