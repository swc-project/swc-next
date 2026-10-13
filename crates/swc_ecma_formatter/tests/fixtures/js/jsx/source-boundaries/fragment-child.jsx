// @ts-ignore
const value0=<>{coveredFirst}{coveredSecond}</>;

// @ts-ignore
const value1=<>
{uncoveredFirst}{uncoveredSecond}
</>;

// @ts-ignore
const value2=<MissingPanel><>{coveredFirst}{coveredSecond}</></MissingPanel>;

const value3=<MissingPanel><>{// @ts-expect-error
coveredLast}</></MissingPanel>;

const value4=<MissingPanel><>{// @ts-expect-error
coveredLast}
</>
</MissingPanel>;
