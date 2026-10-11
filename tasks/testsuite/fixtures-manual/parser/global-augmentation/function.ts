/*---
description: Global augmentations cannot be declared inside functions (TS1234)
negative:
  phase: parse
  type: SyntaxError
---*/
function f() { declare global {} }
