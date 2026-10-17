/*---
description: Strict lexical scopes reject duplicate function implementations
flags: [onlyStrict]
negative:
  phase: early
  type: SyntaxError
---*/
switch (0) {
  case 0:
    function f() {}
    function f() {}
}
