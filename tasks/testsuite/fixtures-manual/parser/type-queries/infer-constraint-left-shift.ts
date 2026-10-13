/*---
description: An infer constraint ending in a type query leaves left shifts in expression parsing.
---*/
value as infer T extends typeof f << other;
