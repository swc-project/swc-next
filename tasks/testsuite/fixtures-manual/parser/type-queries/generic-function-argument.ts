/*---
description: Type queries accept generic function type arguments after a separate opening angle
---*/
type Query = typeof f< <T>(value: T) => T>;
