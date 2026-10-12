/*---
description: Satisfies expressions keep parentheses before class expression consequents
flags: [module]
---*/
(x satisfies T) ? class {} : false;
