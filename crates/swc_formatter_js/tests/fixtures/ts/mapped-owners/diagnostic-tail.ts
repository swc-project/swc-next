type M = { [K in "a"] // @ts-expect-error
: Missing };
