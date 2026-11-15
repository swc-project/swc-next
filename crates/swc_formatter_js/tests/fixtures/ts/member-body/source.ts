type Ignored = {
// prettier-ignore
value  :  { inner :number }
next: string;
};
interface Coverage { // @ts-expect-error
value: Missing; sameLine: Other;
uncovered: Unknown;
}
// @ts-expect-error
interface Covered { /* leading */ value: Missing; sameLine: Other }
type EmptyCoverage = { /* @ts-expect-error */ };
