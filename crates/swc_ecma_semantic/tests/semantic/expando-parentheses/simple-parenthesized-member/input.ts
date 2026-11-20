type Alias = number;
function F() {}
namespace F {
  export interface Marker {}
}
(F.value) = 1;
namespace Use {
  import Alias = F.value;
  export type T = Alias;
  export const result = Alias;
}
