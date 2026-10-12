/*---
description: TSX type assertions in conditional tests keep parentheses before regexp consequents
flags: [module]
---*/
(x as T) ? /a/ : false;
