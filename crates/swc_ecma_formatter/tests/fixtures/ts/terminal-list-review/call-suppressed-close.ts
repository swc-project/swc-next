declare function f(...a: number[]): { bar: 1 };
f(1, /* prettier-ignore */ 2 // @ts-ignore
, /* d */ ).foo;
