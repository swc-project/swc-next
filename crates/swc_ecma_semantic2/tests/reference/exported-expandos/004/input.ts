type F = {};
export function F() {}
export namespace F {
  export interface Marker {}
}
F.x = 1;
namespace N {
  export import Item = F.x;
}
import Alias = N.Item;
const value = Alias;
