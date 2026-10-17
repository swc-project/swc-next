/*---
description: Static block hoist scopes allow duplicate generator function implementations
---*/
class C {
  static {
    function* f() {}
    function* f() {}
  }
}
