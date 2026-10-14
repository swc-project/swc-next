/*---
description: Nested functions in computed parameter keys retain expression identifier checks
flags: [module]
negative:
  phase: parse
  type: SyntaxError
---*/
type F = ({ [(() => { const yield = "prop"; return yield; })()]: value }: { prop: number }) => typeof value;
