/*---
description: Overload signatures merge with their implementation in switch scopes
flags: [module]
---*/
switch (0) {
  case 0:
    function f(): void;
    function f() {}
}
