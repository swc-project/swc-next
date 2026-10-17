/*---
description: Const enums conflict with preceding hoisted vars
flags: [module]
negative:
  phase: early
  type: SyntaxError
---*/
{
  var E;
  const enum E {}
}
