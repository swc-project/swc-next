// Correct
// ParenthesizedExpression
declare const n = 0;
declare const s = "";
declare const t = "";
declare const b = true;
// UnaryExpression
declare let unaryA: number;
declare const unaryB = -12n;
// Incorrect
// UnaryExpression
declare const unaryC: unknown;
declare const unaryD: unknown;
declare const unaryE: {};
declare const unaryF: readonly [];
