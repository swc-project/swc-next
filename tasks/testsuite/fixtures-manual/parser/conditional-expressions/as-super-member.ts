/*---
description: Type assertions keep parentheses before super member call consequents
flags: [module]
---*/
class C extends B { m() { return (x as T) ? super.m() : false; } }
