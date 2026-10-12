/*---
description: Satisfies expressions keep parentheses before delete consequents
flags: [module]
---*/
(x satisfies T) ? delete a.b : false;
