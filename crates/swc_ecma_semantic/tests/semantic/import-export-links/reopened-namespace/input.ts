namespace Outer {
  export function F() {}
}
namespace Outer {
  namespace F {
    export interface I {}
  }
  type A = string;
  const A = 1;
  namespace Test {
    import A = F;
    export const value: typeof A = A;
    export let item: A.I;
  }
}
