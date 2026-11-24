declare const state: number;
switch (state) // @ts-expect-error
{ case missing: break;
}
