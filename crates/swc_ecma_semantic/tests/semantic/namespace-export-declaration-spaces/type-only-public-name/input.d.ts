declare namespace N {
  interface T {}
  export type { T as x };
}
declare namespace M {
  export import y = N.x;
}
