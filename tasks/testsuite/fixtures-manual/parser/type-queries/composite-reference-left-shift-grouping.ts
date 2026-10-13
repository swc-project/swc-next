/*---
description: A type query before a trailing ordinary reference does not remove cast grouping.
---*/
(value as typeof f | Reference) << other;
