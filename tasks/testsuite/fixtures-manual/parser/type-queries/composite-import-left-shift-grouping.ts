/*---
description: Composite types ending in import queries preserve cast grouping before left shifts.
---*/
(value as A | typeof import("m").f) << other;
