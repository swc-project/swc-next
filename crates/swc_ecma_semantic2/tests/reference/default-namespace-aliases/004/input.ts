export default function f() {}
namespace f {
  export interface Item {}
}
namespace Test {
  import A = f;
  const A = 1;
  export const value = A;
}
