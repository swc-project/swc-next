/*---
description: Type assertions keep parentheses before regexp member call consequents
flags: [module]
---*/
(x as T) ? /a/.test(s) : false;
