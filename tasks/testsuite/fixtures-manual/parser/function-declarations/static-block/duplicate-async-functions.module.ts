/*---
description: Static block hoist scopes allow duplicate async function implementations
flags: [module]
---*/
class C {
  static {
    async function f() {}
    async function f() {}
  }
}
