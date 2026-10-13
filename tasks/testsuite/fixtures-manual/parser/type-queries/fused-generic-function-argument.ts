/*---
description: Type queries require a separate opening angle before generic function type arguments
negative:
  phase: parse
  type: SyntaxError
---*/
type Query = typeof f<<T>(value: T) => T>;
