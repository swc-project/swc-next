/*---
description: Using declarations are forbidden at the top level of TypeScript scripts
negative:
  phase: parse
  type: SyntaxError
---*/

using x = null;
