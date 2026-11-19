export type f = number;
export default function f() {}
namespace f {
  export interface Item {}
}
const A = 1;
export namespace Test {
  import A = f;
  export const scalar = A;
}
