/*---
description: Const enums conflict with following hoisted vars
flags: [module]
negative:
  phase: early
  type: SyntaxError
---*/
{
  const enum E {}
  var E;
}
