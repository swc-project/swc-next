/*---
description: Nested exported namespaces in scripts can directly contain using declarations
---*/

namespace N { export namespace M { using x = null; } }
