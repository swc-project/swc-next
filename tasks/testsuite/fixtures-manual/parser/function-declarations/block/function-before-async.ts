/*---
description: Annex B does not merge ordinary functions with async or generator implementations
negative:
  phase: early
  type: SyntaxError
---*/
{
  function f() {}
  async function f() {}
}
