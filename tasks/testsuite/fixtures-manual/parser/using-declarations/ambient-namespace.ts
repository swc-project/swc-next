/*---
description: Ambient namespaces in scripts cannot contain using declarations
negative:
  phase: parse
  type: SyntaxError
---*/

declare namespace N { using x: null; }
