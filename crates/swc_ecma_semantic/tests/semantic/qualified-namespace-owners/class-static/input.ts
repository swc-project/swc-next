declare namespace N {
  namespace Source {
    export interface x {}
  }
  export class X {
    static x: number;
  }
  export { Source as X };
}
const A = 0;
namespace Use {
  import A = N.X.x;
  const value = A;
}
