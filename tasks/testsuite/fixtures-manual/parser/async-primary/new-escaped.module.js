/*---
description: Async primary expression - new-escaped
flags: [module]
negative:
  phase: parse
  type: SyntaxError
---*/

new as\u0079nc function(){};
