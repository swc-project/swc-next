/*---
description: Template literal types support escaped heads inside namespaces
flags: [module]
---*/

namespace N { type T = `\n${string | number}\u0061`; }
