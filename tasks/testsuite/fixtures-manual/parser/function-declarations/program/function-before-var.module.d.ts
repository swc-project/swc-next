/*---
description: Ambient functions conflict with following vars
flags: [module]
negative:
  phase: early
  type: SyntaxError
---*/
declare function f(): void;
declare var f: unknown;
