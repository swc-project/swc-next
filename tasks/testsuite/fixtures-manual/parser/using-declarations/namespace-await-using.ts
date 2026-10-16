/*---
description: Namespaces in scripts cannot directly contain await using declarations
negative:
  phase: parse
  type: SyntaxError
---*/

namespace N { await using x = null; }
