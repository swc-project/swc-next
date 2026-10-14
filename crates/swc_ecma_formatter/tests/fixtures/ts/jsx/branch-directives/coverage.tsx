declare const flag: boolean;
declare const React: any;
const covered = flag ? // @ts-expect-error
<Missing><Child /><Other /></Missing> : null;
const uncovered = flag ? <Missing /> : null;
const logical = flag && // @ts-expect-error
<Missing><Child /><Other /></Missing>;
const logicalUncovered = flag && <Missing />;
const sourcePair = flag ? (// @ts-expect-error
<Missing><Child /><Other /></Missing>) : null;
const ignored = flag ? // prettier-ignore
<Missing   value = {  1  } ><Child/><Other/></Missing> : null;
