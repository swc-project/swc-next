const Alias = 0;
namespace N {
  class C {
    static [123] = 1;
  }
  namespace C {
    export interface x {}
  }
  import Alias = C.x;
  export const result = Alias;
}
