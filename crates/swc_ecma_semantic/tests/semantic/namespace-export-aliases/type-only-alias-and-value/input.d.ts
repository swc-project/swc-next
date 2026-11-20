declare namespace N {
  export const x: number;
}
declare namespace N {
  interface Y {}
  export type { Y as x };
}
