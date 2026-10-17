/*---
description: Async functions do not receive the Annex B duplicate function exception
flags: [module]
negative:
  phase: early
  type: SyntaxError
---*/
{
  async function f() {}
  async function f() {}
}
