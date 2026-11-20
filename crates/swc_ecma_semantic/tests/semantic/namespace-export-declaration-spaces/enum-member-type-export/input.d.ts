declare enum E {
  A,
}
declare namespace E {
  interface X {}
  export type { X as A };
}
