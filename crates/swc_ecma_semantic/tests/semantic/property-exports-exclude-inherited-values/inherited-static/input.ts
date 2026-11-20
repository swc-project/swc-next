export {};
class Base {
  static x = 1;
}
class C extends Base {}
namespace C {
  export interface x {
    own: true;
  }
}
import Alias = C.x;
const typed: Alias = { own: true };
