/*---
description: Template literal types reject invalid escapes in the tail
flags: [module]
negative:
  phase: parse
  type: SyntaxError
---*/

type T = `\n${string}\8`;
