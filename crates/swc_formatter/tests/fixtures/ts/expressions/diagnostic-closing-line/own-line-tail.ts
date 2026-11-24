declare const a: number;
declare const foo: object;
(a
// @ts-ignore
);
foo.unknownMethod();
