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
namespace Holder {
  export import Item = C.x;
}
import Alias = Holder.Item;
const runtime = Alias;
const typed: Alias = { own: true };
const queried: typeof Alias = 1;
