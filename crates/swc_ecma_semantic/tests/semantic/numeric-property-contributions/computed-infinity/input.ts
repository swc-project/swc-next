const Alias = 0;
namespace N {
  class C {
    static [1e400] = 1;
  }
  namespace C {
    export interface Infinity {}
  }
  import Alias = C.Infinity;
  export const result = Alias;
}
