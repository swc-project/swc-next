export {};
const key = "x";
class C {
  static [key] = 1;
}
namespace C {
  export interface x {
    own: true;
  }
}
import Alias = C.x;
const runtime = Alias;
const typed: Alias = { own: true };
const queried: typeof Alias = 1;
