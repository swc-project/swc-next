namespace N { export interface I<T> { x: T; } export interface J<T> { x: T; } }
// prettier-ignore
class Whole implements N . I<  number  > { x  =  7; }
class Entries implements N.I<number>,
  // prettier-ignore
  N . J<  number  > { x = 7; }
