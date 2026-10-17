/*---
description: Overload signatures merge with their implementation in catch scopes
---*/
try {} catch {
  function f(): void;
  function f() {}
}
