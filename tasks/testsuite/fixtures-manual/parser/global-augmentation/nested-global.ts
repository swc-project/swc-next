/*---
description: An external module's global augmentation cannot contain another global augmentation (TS2669)
flags: [module]
negative:
  phase: parse
  type: SyntaxError
---*/
declare global { global {} }
