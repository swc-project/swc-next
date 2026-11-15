const chosen = 1 as
// @ts-expect-error
Record<MissingOne,
MissingTwo>;
const outsideSentinel: number = "x";
