declare function f(a: () => void, b: number): { bar: 1 };
f(() => {}, 2 // @ts-ignore
, /* d */ ).foo;
