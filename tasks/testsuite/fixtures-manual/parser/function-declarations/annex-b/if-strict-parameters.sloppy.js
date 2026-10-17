/*---
description: An Annex B if function still rejects strict directives with non-simple parameters
flags: [noStrict]
negative:
  phase: early
  type: SyntaxError
---*/
if (true) function f(value = 1) { "use strict"; }
