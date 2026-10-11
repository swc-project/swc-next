/*---
description: Non-ambient global augmentations require a declare modifier (TS2670)
flags: [module]
negative:
  phase: parse
  type: SyntaxError
---*/
global {}
