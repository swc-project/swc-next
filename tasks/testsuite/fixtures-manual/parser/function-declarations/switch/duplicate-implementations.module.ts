/*---
description: Module lexical scopes reject duplicate function implementations
flags: [module]
negative:
  phase: early
  type: SyntaxError
---*/
switch (0) {
  case 0:
    function f() {}
    function f() {}
}
