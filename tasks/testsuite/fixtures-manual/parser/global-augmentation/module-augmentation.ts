/*---
description: Module augmentations cannot directly contain global augmentations (TS2669)
flags: [module]
negative:
  phase: parse
  type: SyntaxError
---*/
declare module "pkg" { global {} }
