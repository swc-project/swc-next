namespace Outer {
  export function F() {}
  namespace F {
    export interface I {}
  }
  namespace Test {
    import A = F.I;
    export let item: A;
  }
}
