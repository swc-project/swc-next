export default interface I {
  value: string;
}
namespace I {
  export interface Item {}
}
type A = number;
const A = 1;
namespace Test {
  import A = I;
  export let item: A.Item;
  export let value: A;
  export let scalar: typeof A;
}
