/*---
description: Type assertions in TSX before boolean consequents do not add parentheses
flags: [module]
---*/
x as T ? true : false;
