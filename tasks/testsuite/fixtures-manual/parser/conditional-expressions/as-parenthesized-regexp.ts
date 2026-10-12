/*---
description: Type assertions keep parentheses before parenthesized regexp consequents
flags: [module]
---*/
(x as T) ? (/a/) : false;
