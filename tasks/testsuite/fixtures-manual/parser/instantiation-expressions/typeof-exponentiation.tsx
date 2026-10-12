/*---
description: An instantiation inside a grouped typeof expression round-trips before exponentiation
flags: [module]
---*/
(typeof (f<T>)) ** x;
