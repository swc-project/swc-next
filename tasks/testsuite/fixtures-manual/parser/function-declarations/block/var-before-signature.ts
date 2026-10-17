/*---
description: Bodyless functions retain conflicts with preceding hoisted vars
info: |
  Signature bindings use the same var exclusions as implementations.
  Semantic analysis reports this conflict at the signature, without waiting
  for an implementation.
negative:
  phase: early
  type: SyntaxError
---*/
{
  var f;
  function f(): void;
}
