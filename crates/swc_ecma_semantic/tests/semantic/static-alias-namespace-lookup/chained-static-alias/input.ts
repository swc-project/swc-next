export {};
class C {
  static x = 1;
}
namespace C {
  export interface T {}
}
namespace A {
  export interface T {
    outer: true;
  }
}
namespace N {
  import A = C.x;
  import B = A;
  export const typed: B.T = { outer: true };
  export const runtime = A;
}
