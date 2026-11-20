declare namespace N {
  namespace Source {
    export interface x {}
  }
  export class X {
    static x: number;
  }
  export { Source as X };
}
type A = number;
namespace Use {
  import A = N.X.x;
  const value: A = {};
}
