/*---
description: Type assertions keep parentheses before prefix update consequents
flags: [module]
---*/
(x as T) ? ++a : false;
