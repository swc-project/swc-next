export {};
declare namespace N {
  class Original {
    static x: number;
  }
  export namespace X {
    interface x {
      own: true;
    }
  }
  export { Original as X };
}
const A = 1;
namespace Use {
  import A = N.X.x;
  export const typed: A = { own: true };
  export const runtime = A;
}
