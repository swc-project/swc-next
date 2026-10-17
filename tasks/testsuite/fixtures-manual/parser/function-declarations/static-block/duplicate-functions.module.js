/*---
description: JavaScript static blocks permit duplicate function declarations
flags: [module]
---*/
class C {
  static {
    function f() {}
    function f() {}
  }
}
