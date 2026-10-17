/*---
description: TypeScript static blocks permit function overloads
---*/
class C {
  static {
    function f(): void;
    function f() {}
  }
}
