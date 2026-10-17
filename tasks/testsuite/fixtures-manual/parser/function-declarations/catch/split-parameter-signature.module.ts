/*---
description: Bodyless functions retain conflicts with split catch parameters
info: |
  Signature bindings retain the parameter conflict check used by implementations.
  This check is not deferred until an implementation appears.
flags: [module]
negative:
  phase: early
  type: SyntaxError
---*/
try {} catch ([f = 1]) {
  function f(): void;
}
