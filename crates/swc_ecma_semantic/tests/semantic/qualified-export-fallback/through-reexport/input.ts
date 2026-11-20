export {};
declare namespace N {
  class Original {
    static x: number;
  }
  export namespace X {
    interface T {
      own: true;
    }
  }
  export { Original as X };
}
import Mid = N.X;
declare namespace Holder {
  export { Mid as Y };
}
type A = number;
namespace Use {
  import A = Holder.Y.x;
  export const typed: A = 1;
  export const runtime = A;
  export type Q = typeof A;
}
import B = Holder.Y.T;
const b: B = { own: true };
