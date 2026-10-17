/*---
description: Ordinary function duplicate exceptions do not exempt non-ambient class conflicts
negative:
  phase: early
  type: SyntaxError
---*/
{
  class f {}
  function f() {}
  function f() {}
}
