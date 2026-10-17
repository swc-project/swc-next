/*---
description: An Annex B if function name still follows its own strict body rules
flags: [noStrict]
negative:
  phase: early
  type: SyntaxError
---*/
if (true) function eval() { "use strict"; }
