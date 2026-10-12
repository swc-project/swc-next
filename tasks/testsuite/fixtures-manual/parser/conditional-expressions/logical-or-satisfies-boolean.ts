/*---
description: Logical conditions ending in satisfies expressions do not add parentheses before booleans
flags: [module]
---*/
x || y satisfies T ? true : false;
