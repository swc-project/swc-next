/*---
description: Nested block functions may shadow catch parameters
---*/
try {} catch ([f = 1]) {
  {
    function f() {}
  }
}
