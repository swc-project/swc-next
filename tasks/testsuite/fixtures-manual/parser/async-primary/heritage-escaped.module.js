/*---
description: Async primary expression - heritage-escaped
flags: [module]
negative:
  phase: parse
  type: SyntaxError
---*/

class C extends as\u0079nc function(){} {}
