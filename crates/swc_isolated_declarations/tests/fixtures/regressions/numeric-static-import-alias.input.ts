class C {
  static 1e400 = 1;
}
namespace C {
  export interface Infinity {
    own: true;
  }
}
import Alias = C.Infinity;
const runtime = Alias;
const typed: Alias = { own: true };
export const queried: typeof Alias = 1;
