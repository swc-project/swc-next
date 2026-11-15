let Implementation: unknown;
Implementation = class extends
// @ts-expect-error
MissingBaseFactory(unknownValue)
{};
const outsideSentinel: number = "outside";
