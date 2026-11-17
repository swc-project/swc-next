function F() {}
namespace F {
  export interface Marker {}
}
class C {
  value = (F.value = 1);
}
import Alias = F.value;
type T = Alias;
