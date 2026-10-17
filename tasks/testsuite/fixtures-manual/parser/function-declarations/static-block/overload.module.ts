/*---
description: TypeScript static blocks permit function overloads
flags: [module]
---*/
class C {
  static {
    function f(): void;
    function f() {}
  }
}
