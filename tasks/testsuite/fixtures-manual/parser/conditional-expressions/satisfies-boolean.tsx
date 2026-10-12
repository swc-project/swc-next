/*---
description: Satisfies expressions in TSX before boolean consequents do not add parentheses
flags: [module]
---*/
x satisfies T ? true : false;
