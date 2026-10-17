/*---
description: Nested block functions may shadow catch parameters
flags: [module]
---*/
try {} catch ([f = 1]) {
  {
    function f() {}
  }
}
