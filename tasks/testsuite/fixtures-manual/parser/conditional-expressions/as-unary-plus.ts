/*---
description: Type assertions keep parentheses before unary plus consequents
flags: [module]
---*/
(x as T) ? +a : false;
