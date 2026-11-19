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
export namespace Use {
  import A = N.X.x;
  export const typed: A = 1;
  export const runtime: number = A;
  export type Q = typeof A;
}
import B = N.X.T;
export const b: B = { own: true };
