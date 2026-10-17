/*---
description: An overload signature does not permit multiple module lexical implementations
flags: [module]
negative:
  phase: early
  type: SyntaxError
---*/
{
  function f(): void;
  function f() {}
  function f() {}
}
