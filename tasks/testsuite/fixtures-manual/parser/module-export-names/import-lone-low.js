/*---
description: Named imports reject a lone low surrogate.
esid: sec-module-semantics-static-semantics-early-errors
flags: [module]
negative:
  phase: early
  type: SyntaxError
---*/

import { "\uDC00" as x } from "mod";
