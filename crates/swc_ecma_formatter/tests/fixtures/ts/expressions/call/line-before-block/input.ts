declare function f(a: number, b: number): void;
declare const first: number, second: number;
// @ts-ignore
missing;
f(first // @ts-ignore
, /* c
 d */ second);
