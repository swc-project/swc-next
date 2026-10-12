/*---
description: Typeof operands retain conditional type assertion parentheses before regexp consequents
flags: [module]
---*/
let a = typeof (("a" as unknown) ? /a/ : false);
