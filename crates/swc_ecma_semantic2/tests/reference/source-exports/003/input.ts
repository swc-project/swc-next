export default interface I {
  value: string;
}
export const I = 1;
namespace I {
  export interface Item {}
}
type A = string;
const A = 1;
namespace Test {
  import A = I;
  export let item: A;
  export const scalar: typeof A = A;
}
