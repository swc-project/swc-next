export default function f() {}
namespace f {
  export interface Item {}
}
const A = 1;
type A = string;
namespace Test {
  import A = f;
  export const fn: typeof A = A;
  export let text: A;
}
