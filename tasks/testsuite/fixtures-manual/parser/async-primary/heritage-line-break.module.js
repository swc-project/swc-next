/*---
description: Async primary expression - heritage-line-break
flags: [module]
negative:
  phase: parse
  type: SyntaxError
---*/

class C extends async
function(){} {}
