declare const x: number;
const z = [x // @ts-ignore
, /* c */ ] satisfies Missing;
