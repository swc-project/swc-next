/*---
description: Sloppy blocks retain Annex B ordinary function duplicates in TypeScript
info: |
  Strictness follows the source mode and directives, without tsc's alwaysStrict option.
flags: [noStrict]
---*/
switch (0) {
  case 0:
    function f() {}
    function f() {}
}
