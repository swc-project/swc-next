const Alias = 0;
namespace N {
  function C() {}
  C[123] = 1;
  namespace C {
    export interface x {}
  }
  import Alias = C.x;
  export const result = Alias;
}
