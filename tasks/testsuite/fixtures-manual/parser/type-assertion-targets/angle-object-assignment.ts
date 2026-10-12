/*---
description: Object destructuring retains angle assertion targets
flags: [module]
---*/
({ value: (<number>a) } = obj);
