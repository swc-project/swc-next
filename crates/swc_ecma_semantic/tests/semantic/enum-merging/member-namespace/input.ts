enum E {
  A,
}
namespace E {
  export namespace A {
    export interface X {}
  }
}
namespace E {
  export type T = A.X;
}
