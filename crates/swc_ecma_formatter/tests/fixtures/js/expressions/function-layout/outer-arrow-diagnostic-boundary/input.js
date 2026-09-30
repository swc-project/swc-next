consume("diagnostic",
// @ts-expect-error
(value) => { return missing(value); }); second();
