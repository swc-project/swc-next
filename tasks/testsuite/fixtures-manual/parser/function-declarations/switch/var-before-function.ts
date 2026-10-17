/*---
description: Functions conflict with preceding var declarations in switch scopes
negative:
  phase: early
  type: SyntaxError
---*/
switch (0) {
  case 0:
    var f;
    function f() {}
}
