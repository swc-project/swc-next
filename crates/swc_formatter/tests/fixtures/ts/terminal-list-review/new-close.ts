declare function F(...a: number[]): { bar: 1 };
new F(1, 2 // @ts-ignore
, /* d */ ).foo;
