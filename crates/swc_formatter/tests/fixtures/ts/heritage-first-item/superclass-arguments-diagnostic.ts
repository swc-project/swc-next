class Base<T> {}
class Subject extends Base<number> implements
// @ts-ignore
MissingFirst,
MissingSecond {}
const outsideSentinel: number = "outside";
