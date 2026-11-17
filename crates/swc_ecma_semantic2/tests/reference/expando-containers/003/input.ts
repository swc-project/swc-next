function F() {}
namespace F {
  export interface Marker {}
}
namespace Write {
  F.value = 1;
}
import Alias = F.value;
type T = Alias;
