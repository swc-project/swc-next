interface First { value: string; }
class Subject implements
// @ts-ignore
First, MissingFirst,
MissingSecond {}
const outsideSentinel: number = "outside";
