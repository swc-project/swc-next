/*---
description: Enums conflict with following hoisted vars
flags: [module]
negative:
  phase: early
  type: SyntaxError
---*/
{
  enum E {}
  var E;
}
