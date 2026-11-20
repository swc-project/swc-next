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
type A = number;
namespace Use {
  import A = Mid.x;
  export const typed: A = 1;
  export const runtime = A;
  export type Q = typeof A;
}
import B = Mid.T;
const b: B = { own: true };
