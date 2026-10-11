/*---
description: Ambient namespaces cannot directly contain global augmentations (TS2669)
negative:
  phase: parse
  type: SyntaxError
---*/
declare namespace N { global {} }
