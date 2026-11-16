namespace N { export interface I<T> { x: T; } }
class C implements
  // prettier-ignore
  N . I<  number  > { x = 7; }
