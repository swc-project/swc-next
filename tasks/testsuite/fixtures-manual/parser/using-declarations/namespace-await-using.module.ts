/*---
description: Namespaces in modules cannot directly contain await using declarations
flags: [module]
negative:
  phase: parse
  type: SyntaxError
---*/

namespace N { await using x = null; }
