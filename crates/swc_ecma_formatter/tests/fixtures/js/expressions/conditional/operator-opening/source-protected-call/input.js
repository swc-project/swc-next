// @ts-check
function take() {}
a ? (// @ts-expect-error
take(1))
// prettier-ignore
: b  +c;