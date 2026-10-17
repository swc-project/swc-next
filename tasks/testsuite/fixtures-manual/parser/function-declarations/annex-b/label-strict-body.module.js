/*---
description: Module code forbids label function declarations
flags: [module]
negative:
  phase: early
  type: SyntaxError
---*/
label: function f() { "use strict"; }
