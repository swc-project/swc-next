/*---
description: Async primary expression - heritage-arrow-identifier
flags: [module]
negative:
  phase: parse
  type: SyntaxError
---*/

class C extends async x => x {}
