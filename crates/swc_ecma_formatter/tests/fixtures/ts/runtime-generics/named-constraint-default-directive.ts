function identity<
// @ts-expect-error
T extends Missing = number
>(value: T): T { return value; }
