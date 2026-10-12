/*---
description: Typeof operands retain conditional satisfies parentheses before regexp consequents
flags: [module]
---*/
let a = typeof (("a" satisfies foo) ? /a/ : false);
