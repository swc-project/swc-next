/*---
description: TypeScript static blocks retain JavaScript function implementation merge rules
---*/
class C {
  static {
    function f() {}
    function f() {}
  }
}
