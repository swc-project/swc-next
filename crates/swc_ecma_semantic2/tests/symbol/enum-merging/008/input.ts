const A = 0;
type A = string;
enum E {
  A,
}
namespace E {
  export namespace A {
    export interface X {}
  }
}
namespace E {
  const value = A;
  type Type = A;
  type Qualified = A.X;
}
