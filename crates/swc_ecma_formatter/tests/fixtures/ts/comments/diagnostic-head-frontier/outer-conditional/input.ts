declare const test: boolean, other: number, fn: number;
// @ts-expect-error
test ? (
fn
)() : other;
