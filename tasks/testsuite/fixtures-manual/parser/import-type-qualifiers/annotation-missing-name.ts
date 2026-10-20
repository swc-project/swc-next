/*---
description: Import type qualifiers annotation missing name
flags: [module]
negative:
  phase: parse
  type: SyntaxError
---*/

let x: import("x").
