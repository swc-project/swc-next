const Alias = 0;
namespace N {
  declare class C {
    static 1e400(): void;
  }
  namespace C {
    export interface Infinity {}
  }
  import Alias = C.Infinity;
  export const result = Alias;
}
