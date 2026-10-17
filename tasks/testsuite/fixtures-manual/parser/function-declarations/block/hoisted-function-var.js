/*---
description: An outer function does not exempt a hoisted var from block function conflicts
flags: [noStrict]
negative:
  phase: early
  type: SyntaxError
---*/
function f() {}
{
  var f;
  function f() {}
}
