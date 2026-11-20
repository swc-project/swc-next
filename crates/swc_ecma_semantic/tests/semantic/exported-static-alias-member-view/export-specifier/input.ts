export {};
class C {
  static x = 1;
}
namespace C {
  export interface T {}
}
const A = 2;
type A = number;
namespace A {
  export interface T {}
}
declare namespace N {
  import local = C.x;
  export { local as A };
}
namespace N {
  export const typed: A = 1;
  export const runtime = A;
  export type Queried = typeof A;
  export type Nested = A.T;
}
import Selected = N.A;
const selected = Selected;
