/*---
description: A type binding does not hide a hoisted var from block function conflicts
flags: [module]
negative:
  phase: early
  type: SyntaxError
---*/
{
  var f;
  interface f {}
  function f() {}
}
