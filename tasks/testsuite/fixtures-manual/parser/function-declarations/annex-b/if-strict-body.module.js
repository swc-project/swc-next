/*---
description: Module code forbids if function declarations
flags: [module]
negative:
  phase: early
  type: SyntaxError
---*/
if (true) function f() { "use strict"; }
