/*---
description: Named imports reject a high surrogate followed by a continuation and a non-surrogate.
esid: sec-module-semantics-static-semantics-early-errors
flags: [module]
negative:
  phase: early
  type: SyntaxError
---*/

import { "\uD800\
\u0041" as x } from "mod";
