/*---
description: Satisfies expressions keep parentheses before consequents starting with regexp operands
flags: [module]
---*/
(x satisfies T) ? /a/ === r : false;
