/*---
description: Overloads merge in catch bodies with a separate parameter scope
flags: [module]
---*/
try {} catch ([error = 1]) {
  function f(): void;
  function f() {}
}
