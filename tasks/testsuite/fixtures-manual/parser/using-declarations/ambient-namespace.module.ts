/*---
description: Ambient namespaces in modules cannot contain using declarations
flags: [module]
negative:
  phase: parse
  type: SyntaxError
---*/

declare namespace N { using x: null; }
