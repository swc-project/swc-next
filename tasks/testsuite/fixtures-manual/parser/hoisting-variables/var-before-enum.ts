/*---
description: Enums conflict with preceding hoisted vars
negative:
  phase: early
  type: SyntaxError
---*/
{
  var E;
  enum E {}
}
