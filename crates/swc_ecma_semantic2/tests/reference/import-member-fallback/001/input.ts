namespace N {
  export class C {
    static x = 1;
  }
}
import Owner = N.C;
namespace Owner {
  export interface Item {}
}
type A = string;
namespace Test {
  import A = Owner.x;
  export let text: A;
  export const value = A;
}
