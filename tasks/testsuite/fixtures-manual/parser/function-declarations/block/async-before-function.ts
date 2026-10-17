/*---
description: Annex B does not merge async or generator implementations with ordinary functions
negative:
  phase: early
  type: SyntaxError
---*/
{
  async function f() {}
  function f() {}
}
