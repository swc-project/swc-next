/*---
description: Const enums conflict with preceding hoisted vars
negative:
  phase: early
  type: SyntaxError
---*/
{
  var E;
  const enum E {}
}
