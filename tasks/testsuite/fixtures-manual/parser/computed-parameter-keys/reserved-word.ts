/*---
description: Computed parameter keys in signatures retain expression identifier checks
flags: [module]
negative:
  phase: parse
  type: SyntaxError
---*/
type F = ({ [yield]: value }: object) => typeof value;
