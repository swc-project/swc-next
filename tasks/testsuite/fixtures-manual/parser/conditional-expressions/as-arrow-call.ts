/*---
description: Regexps inside arrow calls do not add parentheses to conditional assertions
flags: [module]
---*/
x as T ? (() => /a/)() : false;
