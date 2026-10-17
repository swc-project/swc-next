/*---
description: Enums conflict with preceding hoisted vars
flags: [module]
negative:
  phase: early
  type: SyntaxError
---*/
{
  var E;
  enum E {}
}
