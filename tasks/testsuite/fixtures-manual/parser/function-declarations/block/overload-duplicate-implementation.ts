/*---
description: An overload signature does not permit multiple strict lexical implementations
flags: [onlyStrict]
negative:
  phase: early
  type: SyntaxError
---*/
{
  function f(): void;
  function f() {}
  function f() {}
}
