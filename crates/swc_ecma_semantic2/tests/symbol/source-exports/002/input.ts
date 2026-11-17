export default function f() {}
export type f = number;
namespace f {
  export interface Item {}
}
type A = string;
const A = 1;
namespace Test {
  import A = f;
  export let item: A;
  export const scalar: typeof A = A;
}
