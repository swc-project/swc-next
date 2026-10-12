/*---
description: Logical AND conditional tests retain trailing type assertion parentheses before regexp consequents
flags: [module]
---*/
x && (y as T) ? /a/ : false;
