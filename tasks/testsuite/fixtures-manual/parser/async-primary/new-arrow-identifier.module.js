/*---
description: Async primary expression - new-arrow-identifier
flags: [module]
negative:
  phase: parse
  type: SyntaxError
---*/

new async x => x;
