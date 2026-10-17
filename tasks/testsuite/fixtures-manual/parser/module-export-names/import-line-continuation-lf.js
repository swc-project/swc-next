/*---
description: Named imports pair surrogate escapes across LF line continuations.
esid: sec-module-semantics-static-semantics-early-errors
flags: [module]
---*/

import { "\uD800\
\uDC00" as x } from "mod";
