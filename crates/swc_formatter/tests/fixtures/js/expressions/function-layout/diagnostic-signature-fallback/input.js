consume("diagnostic", (value =
// @ts-expect-error
missing()) => { return value; }); second();
