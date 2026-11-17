namespace Owner {
  export interface Item {}
}
import Owner = N.C;
namespace N {
  export class C {
    static x = 1;
  }
}
import Mid = Owner;
type A = string;
namespace Test {
  import A = Mid.x;
  export let text: A;
  export const value = A;
}
