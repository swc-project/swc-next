/*---
description: Const enums conflict with following hoisted vars
negative:
  phase: early
  type: SyntaxError
---*/
{
  const enum E {}
  var E;
}
