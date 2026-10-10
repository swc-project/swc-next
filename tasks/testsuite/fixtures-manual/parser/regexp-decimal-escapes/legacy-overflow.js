/*---
description: Oversized decimal escapes fall back to legacy octal escapes without Unicode flags
---*/
const re = /()\18446744073709551616/;
