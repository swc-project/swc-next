/*---
description: Object destructuring retains as assertion targets
flags: [module]
---*/
({ value: (a as number) } = obj);
