declare namespace N {
  export interface x {}
}
declare namespace M {
  export import y = N.x;
  interface y {}
}
