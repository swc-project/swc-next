/*---
description: Global augmentations cannot be declared inside ordinary blocks (TS1234)
negative:
  phase: parse
  type: SyntaxError
---*/
{ declare global {} }
