/*---
description: Annex B does not merge async or generator implementations with ordinary functions
negative:
  phase: early
  type: SyntaxError
---*/
{
  function* f() {}
  function f() {}
}
