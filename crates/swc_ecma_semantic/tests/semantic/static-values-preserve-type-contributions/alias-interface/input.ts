export {};
class C {
  static x = 1;
}
namespace C {
  export interface T {}
}
import Alias = C.x;
interface Alias {
  own: true;
}
const runtime = Alias;
const typed: Alias = { own: true };
const queried: typeof Alias = 1;
