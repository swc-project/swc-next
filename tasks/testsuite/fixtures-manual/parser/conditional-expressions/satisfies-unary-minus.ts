/*---
description: Satisfies expressions keep parentheses before unary minus identifiers
flags: [module]
---*/
(x satisfies T) ? -a : false;
