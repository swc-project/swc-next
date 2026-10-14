/*---
description: Computed parameter keys in signatures retain expression string checks
flags: [module]
negative:
  phase: parse
  type: SyntaxError
---*/
type F = ({ ["\1"]: value }: object) => typeof value;
