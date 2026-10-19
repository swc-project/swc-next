/*---
description: Paragraph separators in leading block comments remain stable when printing yield expressions.
---*/
function* f() { yield (/* lead trail */ value); }
