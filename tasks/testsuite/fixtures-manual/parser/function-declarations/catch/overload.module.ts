/*---
description: Overload signatures merge with their implementation in catch scopes
flags: [module]
---*/
try {} catch {
  function f(): void;
  function f() {}
}
