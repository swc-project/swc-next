function F() {}
namespace F {
  export interface Marker {}
}
function write() {
  F.value = 1;
}
import Alias = F.value;
type T = Alias;
