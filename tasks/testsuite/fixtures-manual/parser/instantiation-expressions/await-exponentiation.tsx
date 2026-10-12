/*---
description: An instantiation inside a grouped await expression round-trips before exponentiation
flags: [module]
---*/
(await (f<T>)) ** x;
