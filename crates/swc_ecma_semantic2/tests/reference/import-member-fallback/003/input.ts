namespace N {
  export class C {
    static x = 1;
  }
}
import Owner = N.C;
namespace Owner {
  export interface x {}
}
const A = 1;
namespace Test {
  import A = Owner.x;
  export let text: A;
  export const value = A;
}
