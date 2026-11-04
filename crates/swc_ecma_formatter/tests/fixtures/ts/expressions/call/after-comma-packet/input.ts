declare function f(a: number, b: number): void;
declare const first: number;
f(first, /*#__PURE__*/ // @ts-expect-error
missing());
