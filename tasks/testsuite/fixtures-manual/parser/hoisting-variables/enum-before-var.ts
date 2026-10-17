/*---
description: Enums conflict with following hoisted vars
negative:
  phase: early
  type: SyntaxError
---*/
{
  enum E {}
  var E;
}
