/*---
description: Intersection types ending in a type query leave left shifts after satisfies.
---*/
value satisfies A & typeof f << other;
