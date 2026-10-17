/*---
description: Strict outer code forbids label function declarations
flags: [onlyStrict]
negative:
  phase: early
  type: SyntaxError
---*/
label: function f() { "use strict"; }
