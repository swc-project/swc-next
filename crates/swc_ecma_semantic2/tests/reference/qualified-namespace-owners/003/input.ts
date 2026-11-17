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
  import Owner = N.X;
  import A = Owner.x;
  const value: A = {};
}
