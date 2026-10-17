/*---
description: Var declarations conflict with preceding functions in switch scopes
flags: [module]
negative:
  phase: early
  type: SyntaxError
---*/
switch (0) {
  case 0:
    function f() {}
    var f;
}
