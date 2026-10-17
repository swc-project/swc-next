/*---
description: A type binding does not hide a hoisted var from block function conflicts
negative:
  phase: early
  type: SyntaxError
---*/
{
  var f;
  interface f {}
  function f() {}
}
