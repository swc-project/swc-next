declare const fn: number;
// @ts-expect-error
const value: string = (
// @ts-expect-error
[fn, unknown]
);
