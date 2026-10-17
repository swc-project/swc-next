/*---
description: Ambient functions conflict with preceding vars
flags: [noStrict]
negative:
  phase: early
  type: SyntaxError
---*/
declare var f: unknown;
declare function f(): void;
