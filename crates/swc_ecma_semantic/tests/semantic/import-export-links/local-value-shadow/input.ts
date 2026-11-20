namespace Outer {
  export function F() {}
  namespace F {
    export interface I {}
  }
  namespace Test {
    import A = F;
    const A = 1;
    export const value = A;
  }
}
