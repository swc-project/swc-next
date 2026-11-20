export default class C {}
namespace C {
  export interface Item {}
}
namespace A {
  export interface Item {}
}
namespace Test {
  import A = C;
  export let value: A;
  export let ctor: typeof A;
  export let item: A.Item;
}
