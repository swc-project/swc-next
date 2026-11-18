const object: { known: number } = {
// @ts-expect-error
extra /* a */
// B
: missingValue };
