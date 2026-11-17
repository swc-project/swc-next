export {};
const A = 0;
type A = string;
namespace Target {
  export interface T {}
}
enum E {
  A = 42,
  B = A,
}
declare namespace E {
  export import A = Target;
}
namespace E {
  const value: 0 = A;
  type ValueType = A;
  type Qualified = A.T;
}
