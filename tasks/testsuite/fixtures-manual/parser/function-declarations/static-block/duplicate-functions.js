/*---
description: JavaScript static blocks permit duplicate function declarations
---*/
class C {
  static {
    function f() {}
    function f() {}
  }
}
