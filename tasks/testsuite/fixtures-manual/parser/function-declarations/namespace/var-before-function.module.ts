/*---
description: Exported functions conflict with vars in earlier namespace bodies
flags: [module]
negative:
  phase: early
  type: SyntaxError
---*/
namespace N {
  export var f: unknown;
}
namespace N {
  export function f() {}
}
