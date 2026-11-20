export {};
class C {
  static x = 1;
}
namespace C {
  export interface T {}
}
namespace Holder {
  export import Value = C.x;
}
type A = number;
namespace A {
  export interface T {}
}
namespace N {
  import A = Holder.Value;
  export const typed: A = 1;
  export const runtime = A;
  export type Queried = typeof A;
  export type Nested = A.T;
}
