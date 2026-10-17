/*---
description: A strict function body keeps an Annex B if binding separate from outer lexicals
flags: [noStrict]
---*/
let f = 1;
if (true) function f() { "use strict"; }
