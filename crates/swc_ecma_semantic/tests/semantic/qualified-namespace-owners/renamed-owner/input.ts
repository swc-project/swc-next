declare namespace N {
  namespace Source {
    export interface x {}
  }
  export class X {
    static x: number;
  }
  export { Source as X };
  export { X as Y };
}
type A = number;
namespace Use {
  import A = N.Y.x;
  const value: A = {};
}
