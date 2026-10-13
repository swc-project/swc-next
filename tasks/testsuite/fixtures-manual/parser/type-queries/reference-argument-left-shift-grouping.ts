/*---
description: Queries inside ordinary type arguments preserve cast grouping before left shifts.
---*/
(value as Wrapper<typeof f>) << other;
