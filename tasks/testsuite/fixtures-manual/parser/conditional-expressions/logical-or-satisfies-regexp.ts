/*---
description: Logical OR conditional tests retain trailing satisfies parentheses before regexp consequents
flags: [module]
---*/
x || (y satisfies T) ? /a/ : false;
