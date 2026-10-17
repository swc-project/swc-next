/*---
description: Nested if function placement uses the enclosing function strictness
flags: [noStrict]
---*/
function outer() {
  if (true) function f() { "use strict"; }
}
