/*---
description: JavaScript static blocks permit var and function merges
---*/
class C {
  static {
    function f() {}
    var f;
  }
}
