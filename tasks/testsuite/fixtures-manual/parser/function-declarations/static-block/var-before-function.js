/*---
description: JavaScript static blocks permit var and function merges
---*/
class C {
  static {
    var f;
    function f() {}
  }
}
