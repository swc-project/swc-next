/*---
description: Left shifts preserve grouping around typeof import type assertions
---*/
(value as typeof import("m").f) << other;
