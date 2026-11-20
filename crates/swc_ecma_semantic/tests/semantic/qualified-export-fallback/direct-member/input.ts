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
type A = number;
namespace Use {
  import A = N.X.x;
  export const typed: A = 1;
  export const runtime = A;
  export type Q = typeof A;
}
import B = N.X.T;
const b: B = { own: true };
