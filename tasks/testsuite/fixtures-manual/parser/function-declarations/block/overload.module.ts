/*---
description: Overload signatures merge with their implementation in block scopes
flags: [module]
---*/
{
  function f(): void;
  function f() {}
}
