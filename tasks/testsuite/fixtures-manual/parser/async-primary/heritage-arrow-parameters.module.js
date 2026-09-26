/*---
description: Async primary expression - heritage-arrow-parameters
flags: [module]
negative:
  phase: parse
  type: SyntaxError
---*/

class C extends async () => {} {}
