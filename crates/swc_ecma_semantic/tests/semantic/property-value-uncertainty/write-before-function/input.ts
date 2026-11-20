export {};
C.x = 1;
function C() {}
namespace C {
  export interface T {}
}
type A = number;
namespace A {
  export interface T {}
}
namespace N {
  import A = C.x;
  export const typed: A = 1;
  export const runtime = A;
  export type Queried = typeof A;
  export type Nested = A.T;
}
