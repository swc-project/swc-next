/*---
description: Quoted modules cannot be nested inside an external module augmentation (TS2435)
flags: [module]
negative:
  phase: parse
  type: SyntaxError
---*/
declare module "outer" { module "inner" {} }
