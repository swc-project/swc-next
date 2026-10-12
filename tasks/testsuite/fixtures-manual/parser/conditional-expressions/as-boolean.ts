/*---
description: Type assertions before boolean consequents do not add parentheses
flags: [module]
---*/
x as T ? true : false;
