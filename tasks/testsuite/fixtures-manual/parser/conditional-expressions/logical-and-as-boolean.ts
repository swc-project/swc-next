/*---
description: Logical conditions ending in type assertions do not add parentheses before booleans
flags: [module]
---*/
x && y as T ? true : false;
