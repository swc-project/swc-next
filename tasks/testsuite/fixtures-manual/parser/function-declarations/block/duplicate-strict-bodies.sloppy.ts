/*---
description: Strict function bodies do not disable sloppy block ordinary function duplicates
flags: [noStrict]
---*/
{
  function f() { "use strict"; }
  function f() { "use strict"; }
}
