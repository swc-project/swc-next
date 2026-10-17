/*---
description: JavaScript static blocks permit var and function merges
flags: [module]
---*/
class C {
  static {
    var f;
    function f() {}
  }
}
