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
export declare namespace Use {
  import A = N.X.x;
  export const typed: A;
  export const runtime: number;
  export type Q = typeof A;
  export {};
}
import B = N.X.T;
export declare const b: B;
