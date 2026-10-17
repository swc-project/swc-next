/*---
description: Annex B does not merge ordinary functions with async or generator implementations
flags: [module]
negative:
  phase: early
  type: SyntaxError
---*/
{
  function f() {}
  function* f() {}
}
