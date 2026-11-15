declare function f(...a: number[]): { bar: 1 };
f(1, 2 // @ts-ignore
, /* d */ ).foo;
