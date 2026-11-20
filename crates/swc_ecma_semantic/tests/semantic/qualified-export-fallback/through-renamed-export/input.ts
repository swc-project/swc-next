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
  export { X as Y };
}
type A = number;
namespace Use {
  import A = N.Y.x;
  export const typed: A = 1;
  export const runtime = A;
  export type Q = typeof A;
}
import B = N.Y.T;
const b: B = { own: true };
