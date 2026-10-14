/*---
description: Computed parameter keys in signatures retain expression literal checks
flags: [module]
negative:
  phase: parse
  type: SyntaxError
---*/
type F = ({ [010]: value }: { 8: number }) => typeof value;
