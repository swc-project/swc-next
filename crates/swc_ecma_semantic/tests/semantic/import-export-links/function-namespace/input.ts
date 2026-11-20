namespace Outer {
  export function F() {}
  namespace F {
    export interface I {}
  }
  type A = string;
  const A = 1;
  namespace Test {
    import A = F;
    export const value: typeof A = A;
    export let text: A;
  }
}
