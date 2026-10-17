/*---
description: Strict outer code forbids if function declarations
flags: [onlyStrict]
negative:
  phase: early
  type: SyntaxError
---*/
if (true) function f() { "use strict"; }
