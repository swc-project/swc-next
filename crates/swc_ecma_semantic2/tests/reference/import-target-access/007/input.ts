namespace M {
  export class X {}
}
import type N = M;
const A = 0;
namespace Use {
  import A = N.X;
  import B = M.X;
  type Instance = A;
  type Constructor = typeof A;
  const a = A;
  const b = B;
}
