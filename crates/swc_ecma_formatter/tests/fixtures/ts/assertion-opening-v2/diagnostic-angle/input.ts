declare const y: unknown;
const x = <
// @ts-expect-error
Missing | string>y;
