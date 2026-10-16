/*---
description: Template literal types reject invalid escapes in the head
flags: [module]
negative:
  phase: parse
  type: SyntaxError
---*/

type T = `\unicode${string}`;
