/*---
description: Parenthesized operands end a type operator before a left shift.
---*/
(value as keyof (A | typeof f)) << other;
