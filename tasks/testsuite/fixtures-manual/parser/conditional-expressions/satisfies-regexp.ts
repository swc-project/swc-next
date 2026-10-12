/*---
description: Satisfies expressions in conditional tests keep parentheses before regexp consequents
flags: [module]
---*/
(x satisfies T) ? /a/ : false;
