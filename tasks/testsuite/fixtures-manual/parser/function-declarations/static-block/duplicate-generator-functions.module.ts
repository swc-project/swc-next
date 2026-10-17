/*---
description: Static block hoist scopes allow duplicate generator function implementations
flags: [module]
---*/
class C {
  static {
    function* f() {}
    function* f() {}
  }
}
