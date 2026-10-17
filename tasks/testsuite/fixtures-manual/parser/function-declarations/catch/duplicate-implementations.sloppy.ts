/*---
description: Sloppy blocks retain Annex B ordinary function duplicates in TypeScript
info: |
  Strictness follows the source mode and directives, without tsc's alwaysStrict option.
flags: [noStrict]
---*/
try {} catch {
  function f() {}
  function f() {}
}
