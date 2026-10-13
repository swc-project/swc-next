/*---
description: Type predicates ending in a type query leave left shifts after function types.
---*/
value as (x: unknown) => x is typeof f << other;
