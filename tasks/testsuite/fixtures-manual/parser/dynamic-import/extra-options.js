/*---
description: Phase import calls share the dynamic import argument grammar
flags: [module]
negative:
  phase: parse
  type: SyntaxError
---*/
import.source('m', {}, {});
