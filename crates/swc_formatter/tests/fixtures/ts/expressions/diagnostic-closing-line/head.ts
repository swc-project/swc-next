declare const a: number;
declare const foo: object;
const value = (
/* @ts-expect-error */ a
);
foo.unknownMethod();
