/*---
description: Ambient functions conflict with following vars
flags: [noStrict]
negative:
  phase: early
  type: SyntaxError
---*/
declare function f(): void;
declare var f: unknown;
