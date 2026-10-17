/*---
description: Overload signatures merge with their implementation in switch scopes
---*/
switch (0) {
  case 0:
    function f(): void;
    function f() {}
}
