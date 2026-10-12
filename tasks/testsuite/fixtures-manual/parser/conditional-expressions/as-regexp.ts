/*---
description: Type assertions in conditional tests keep parentheses before regexp consequents
flags: [module]
---*/
(x as T) ? /a/ : false;
