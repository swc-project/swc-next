function F() {}
namespace F {
  export interface Marker {}
}
const object = { value: (F.value = 1) };
import Alias = F.value;
type T = Alias;
