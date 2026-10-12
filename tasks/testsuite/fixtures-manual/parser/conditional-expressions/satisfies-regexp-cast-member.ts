/*---
description: Satisfies expressions keep parentheses before member calls on asserted regexps
flags: [module]
---*/
(x satisfies T) ? (/a/ as R).test(s) : false;
