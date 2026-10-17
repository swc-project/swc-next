/*---
description: Named imports reject a lone high surrogate.
esid: sec-module-semantics-static-semantics-early-errors
flags: [module]
negative:
  phase: early
  type: SyntaxError
---*/

import { "\uD800" as x } from "mod";
