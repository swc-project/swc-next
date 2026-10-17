/*---
description: Generators do not receive the Annex B duplicate function exception
flags: [module]
negative:
  phase: early
  type: SyntaxError
---*/
{
  function* f() {}
  function* f() {}
}
