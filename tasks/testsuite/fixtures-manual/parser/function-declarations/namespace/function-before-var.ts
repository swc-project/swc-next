/*---
description: Exported vars conflict with functions in earlier namespace bodies
negative:
  phase: early
  type: SyntaxError
---*/
namespace N {
  export function f() {}
}
namespace N {
  export var f: unknown;
}
