/*---
description: Async primary expression - new-arrow-parameters
flags: [module]
negative:
  phase: parse
  type: SyntaxError
---*/

new async () => {};
